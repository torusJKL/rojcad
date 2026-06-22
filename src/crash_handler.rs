//! Crash diagnostics: signal handler, panic hook, and crash log file.
//!
//! On startup, opens `{temp_dir}/rojcad_crash_{pid}.log`. On clean exit,
//! the file is removed (via startup truncation). On crash:
//! - **Rust panics**: Panic hook writes message + backtrace to file + stderr
//! - **SIGSEGV/SIGABRT/SIGBUS/SIGILL**: Signal handler captures backtrace,
//!   forks a child process to write raw addresses to crash file,
//!   parent exits with 128 + signal number.
//!
//! Crash log location: `std::env::temp_dir()` (`/tmp` on Linux/macOS, `%TEMP%` on Windows)

use backtrace::Backtrace;
use std::any::Any;
use std::ffi::c_int;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static CRASH_FILE: OnceLock<Mutex<File>> = OnceLock::new();
static CRASH_PATH: OnceLock<PathBuf> = OnceLock::new();
#[cfg(unix)]
static CRASH_FD: OnceLock<c_int> = OnceLock::new();

pub fn init() {
    match create_crash_file() {
        Ok((file, path)) => {
            #[cfg(unix)]
            {
                use std::os::fd::AsRawFd;
                let fd = file.as_raw_fd();
                CRASH_FD.set(fd).ok();
            }
            CRASH_PATH.set(path).ok();
            CRASH_FILE.set(Mutex::new(file)).ok();
        }
        Err(e) => {
            eprintln!("rojcad: warning: could not create crash log: {e}");
        }
    }

    install_panic_hook();

    #[cfg(unix)]
    install_signal_handler();
}

fn crash_log_path() -> PathBuf {
    let pid = std::process::id();
    std::env::temp_dir().join(format!("rojcad_crash_{pid}.log"))
}

fn create_crash_file() -> std::io::Result<(File, PathBuf)> {
    let path = crash_log_path();
    let _ = std::fs::remove_file(&path);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    Ok((file, path))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| String::from("?"));

        let msg = payload_message(info.payload());
        let bt = Backtrace::new();

        if let Some(mtx) = CRASH_FILE.get() {
            if let Ok(mut file) = mtx.lock() {
                let _ = writeln!(file, "PANIC");
                let _ = writeln!(file, "  at {location}");
                let _ = writeln!(file, "  {msg}");
                let _ = writeln!(file, "Backtrace:\n{bt:?}");
                let _ = file.flush();
            }
        }

        eprintln!("thread 'main' panicked at {location}:");
        eprintln!("  {msg}");
        if let Some(path) = CRASH_PATH.get() {
            eprintln!("note: crash log written to {}", path.display());
        }
    }));
}

fn payload_message(payload: &dyn Any) -> String {
    payload
        .downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("(no message)")
        .to_string()
}

/// Remove crash log file on clean exit (called at end of main).
/// If the process has crashed, this won't be reached.
pub(crate) fn cleanup() {
    if let Some(path) = CRASH_PATH.get() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn panic_detail(panic: &Box<dyn Any + Send>) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "(unknown)".to_string())
}

#[cfg(unix)]
fn install_signal_handler() {
    use std::mem::MaybeUninit;

    let alt_stack_size: usize = 8192 * 16;
    let mut alt_stack = MaybeUninit::<libc::stack_t>::uninit();
    unsafe {
        let ss = alt_stack.as_mut_ptr();
        (*ss).ss_sp = libc::malloc(alt_stack_size);
        if (*ss).ss_sp.is_null() {
            eprintln!("rojcad: warning: could not allocate alternate signal stack");
            return;
        }
        (*ss).ss_size = alt_stack_size;
        (*ss).ss_flags = 0;
        if libc::sigaltstack(ss, std::ptr::null_mut()) != 0 {
            let _ = libc::free((*ss).ss_sp);
            eprintln!("rojcad: warning: could not set alternate signal stack");
            return;
        }
    }

    let signals = [libc::SIGSEGV, libc::SIGABRT, libc::SIGBUS, libc::SIGILL];
    for &sig in &signals {
        unsafe {
            let mut sa = MaybeUninit::<libc::sigaction>::uninit();
            let sa_ptr = sa.as_mut_ptr();
            (*sa_ptr).sa_sigaction = crash_signal_handler as *const () as libc::sighandler_t;
            (*sa_ptr).sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK | libc::SA_NODEFER;
            let _ = libc::sigemptyset(&mut (*sa_ptr).sa_mask);
            libc::sigaction(sig, sa_ptr, std::ptr::null_mut());
        }
    }
}

/// Signal handler. Must be async-signal-safe: no heap allocation, no lock acquisition.
/// Strategy: capture raw addresses to stack buffer, write immediate signal notification
/// to stderr, fork() a child to write the crash file (child has a clean address space
/// and can safely use the Rust allocator), then parent calls _exit().
#[cfg(unix)]
extern "C" fn crash_signal_handler(
    sig: c_int,
    _info: *mut libc::siginfo_t,
    _ucontext: *mut libc::c_void,
) {
    // Write immediate notification to stderr (async-signal-safe).
    // This is the ONLY stderr output guaranteed to appear before exit.
    let sig_tag_ptr = match sig {
        s if s == libc::SIGSEGV => b"SIGSEGV\n\0" as *const u8,
        s if s == libc::SIGABRT => b"SIGABRT\n\0" as *const u8,
        s if s == libc::SIGBUS => b"SIGBUS\n\0" as *const u8,
        s if s == libc::SIGILL => b"SIGILL\n\0" as *const u8,
        _ => b"UNKNOWN\n\0" as *const u8,
    };
    let prelude = b"rojcad: ";
    let _ = unsafe { libc::write(libc::STDERR_FILENO, prelude.as_ptr() as *const _, prelude.len()) };
    let tag_len = unsafe { libc::strlen(sig_tag_ptr as *const _) };
    let _ = unsafe { libc::write(libc::STDERR_FILENO, sig_tag_ptr as *const _, tag_len) };

    // Capture raw addresses to stack buffer (no allocation)
    let mut frames: [usize; 128] = [0; 128];
    let mut count = 0;

    backtrace::trace(|frame| {
        if count < frames.len() {
            frames[count] = frame.ip() as usize;
            count += 1;
        }
        true
    });

    // Fork a child to write crash report (fork is async-signal-safe)
    let pid = unsafe { libc::fork() };

    if pid == 0 {
        // CHILD: has own address space, safe to allocate
        let sig_name = match sig {
            s if s == libc::SIGSEGV => "SIGSEGV",
            s if s == libc::SIGABRT => "SIGABRT",
            s if s == libc::SIGBUS => "SIGBUS",
            s if s == libc::SIGILL => "SIGILL",
            _ => "UNKNOWN",
        };

        // Use the parent's crash path, not child's own PID
        if let Some(path) = CRASH_PATH.get() {
            if let Ok(mut f) = std::fs::File::create(path) {
                use std::io::Write;

                let pid = std::process::id();
                let secs = now_secs();
                let _ = writeln!(f, "rojcad crash report — pid={pid}  ({secs})");
                let _ = writeln!(f, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                let _ = writeln!(f, "Signal: {sig_name}");
                let _ = writeln!(f);
                let _ = writeln!(f, "Backtrace ({count} frames):");

                // Raw addresses only — backtrace::resolve allocates which can
                // fail after fork() from a signal handler with corrupted heap.
                // Resolve offline: addr2line -e rojcad <addr>
                let _ = writeln!(f, " (resolve: addr2line -e rojcad <addr>)");
                for (i, &addr) in frames[..count].iter().enumerate() {
                    let _ = writeln!(f, " {i:>2}: 0x{addr:016x}");
                    let _ = f.flush();
                }

                let _ = writeln!(f);
                let _ = writeln!(f, "End of crash report.");
                let _ = f.flush();
            }

            let _ = writeln!(
                std::io::stderr(),
                "rojcad: {sig_name} - crash report written to {}",
                path.display()
            );
        }

        unsafe { libc::_exit(1) };
    } else if pid > 0 {
        // PARENT: exit immediately
        unsafe { libc::_exit(128 + sig) };
    } else {
        // FORK FAILED: write raw addresses via pre-opened fd
        write_raw_backtrace(sig, &frames[..count]);
        unsafe { libc::_exit(128 + sig) };
    }
}

#[cfg(unix)]
fn write_raw_backtrace(sig: c_int, frames: &[usize]) {
    let sig_name = match sig {
        s if s == libc::SIGSEGV => "SIGSEGV",
        s if s == libc::SIGABRT => "SIGABRT",
        s if s == libc::SIGBUS => "SIGBUS",
        s if s == libc::SIGILL => "SIGILL",
        _ => "UNKNOWN",
    };

    let fd = match CRASH_FD.get() {
        Some(fd) => *fd,
        None => return,
    };

    let pid = std::process::id();
    let header = format!("rojcad raw crash — pid={pid} signal={sig_name}\n");
    let _ = unsafe { libc::write(fd, header.as_ptr() as *const _, header.len()) };

    for (i, &addr) in frames.iter().enumerate() {
        let line = format!(" {i:>2}: 0x{addr:016x}\n");
        let _ = unsafe { libc::write(fd, line.as_ptr() as *const _, line.len()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_file_path() {
        let path = crash_log_path();
        let s = path.to_string_lossy();
        assert!(s.contains("rojcad_crash_"));
        assert!(s.contains(&std::process::id().to_string()));
        assert!(s.ends_with(".log"));
    }

    #[test]
    fn test_create_crash_file() {
        let (file, path) = create_crash_file().expect("should create crash file");
        assert!(path.exists());
        let mut f = file;
        let r = writeln!(f, "test write");
        assert!(r.is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_payload_message_string() {
        let msg = "custom error".to_string();
        assert_eq!(payload_message(&msg as &dyn Any), "custom error");
    }

    #[test]
    fn test_payload_message_str() {
        let msg: &str = "static str error";
        assert_eq!(payload_message(&msg as &dyn Any), "static str error");
    }

    #[test]
    fn test_payload_message_other() {
        let val: i32 = 42;
        assert_eq!(payload_message(&val as &dyn Any), "(no message)");
    }

    #[test]
    fn test_panic_detail_string() {
        let panic: Box<dyn Any + Send> = Box::new("detail msg".to_string());
        assert_eq!(panic_detail(&panic), "detail msg");
    }

    #[test]
    fn test_panic_detail_str() {
        let panic: Box<dyn Any + Send> = Box::new("str msg");
        assert_eq!(panic_detail(&panic), "str msg");
    }

    #[test]
    fn test_panic_detail_unknown() {
        let panic: Box<dyn Any + Send> = Box::new(42);
        assert_eq!(panic_detail(&panic), "(unknown)");
    }
}
