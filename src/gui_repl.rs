use std::ffi::{CStr, CString, c_char};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};

static REQ_TX: OnceLock<mpsc::Sender<String>> = OnceLock::new();
static REQ_RX: OnceLock<Mutex<mpsc::Receiver<String>>> = OnceLock::new();
static RESP_TX: OnceLock<mpsc::Sender<String>> = OnceLock::new();

/// Create the channels and return the ends the viewer thread needs:
/// (request_sender, response_receiver)
pub fn init() -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
    let (req_tx, req_rx) = mpsc::channel::<String>();
    let (resp_tx, resp_rx) = mpsc::channel::<String>();
    REQ_TX.set(req_tx).expect("gui_repl::init called twice");
    REQ_RX
        .set(Mutex::new(req_rx))
        .expect("gui_repl::init called twice");
    RESP_TX.set(resp_tx).expect("gui_repl::init called twice");
    (REQ_TX.get().unwrap().clone(), resp_rx)
}

/// FFI: Called from Janet (REPL thread). Returns next pending request string,
/// or nil if no request is pending. Caller must free the returned string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gui_repl_poll_request() -> *mut c_char {
    let rx = match REQ_RX.get() {
        Some(rx) => rx,
        None => return std::ptr::null_mut(),
    };
    let rx = match rx.lock() {
        Ok(rx) => rx,
        Err(_) => return std::ptr::null_mut(),
    };
    match rx.try_recv() {
        Ok(msg) => {
            let c_msg = CString::new(msg).unwrap();
            c_msg.into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

/// FFI: Called from Janet (REPL thread). Sends a response JSON string
/// back to the viewer thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gui_repl_send_response(response: *const c_char) {
    let tx = match RESP_TX.get() {
        Some(tx) => tx,
        None => return,
    };
    let s = match unsafe { CStr::from_ptr(response) }.to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let _ = tx.send(s.to_string());
}
