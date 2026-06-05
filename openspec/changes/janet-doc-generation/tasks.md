## 1. Rust — `--eval` CLI flag

- [x] 1.1 Add `janet_cstringv` extern declaration to `src/bridge.rs`
- [x] 1.2 Parse `--eval <expr>` args in `src/main.rs`, join multiple into one string
- [x] 1.3 Append `--eval` expression to boot.janet source for evaluation at startup

## 2. boot.janet — eval support

- [x] 2.1 `--eval` expressions are appended as raw code at end of boot.janet
- [x] 2.2 eval runs in normal boot.janet context (all helpers available)

## 3. boot.janet — `dump-docs` function

- [x] 3.1 Implement `(dump-docs &opt path)` — iterate `(group)`, call `(doc 'fn)` per function, write Markdown file to `doc/janet-api.md`
- [x] 3.2 Implement HTML generation — same iteration, emit single-file HTML with inline CSS and JS to `doc/janet-api.html`
- [x] 3.3 Implement Janet syntax tokenizer — parse example code strings into `<span class="tok-*">` tagged HTML (comments gray, strings red, keywords blue, numbers green, special forms brown)
- [x] 3.4 Implement search JS (Ctrl+K listener, input filter of `<article>` elements)
- [x] 3.5 Implement sidebar nav HTML (collapsible category list, anchor links)
- [x] 3.6 Add error handling for file write failures

## 4. justfile — doc generation recipe

- [x] 4.1 Add `doc-janet` recipe that runs `cargo run -- --headless --eval '(do (dump-docs "doc") (os/exit 0))'`
- [x] 4.2 `just doc-janet` produces `doc/janet-api.md` and `doc/janet-api.html`

## 5. Verification

- [x] 5.1 Build and run `just doc-janet`, both output files generated
- [x] 5.2 Markdown structure verified (8 categories, 30 functions, code blocks with janet tag)
- [x] 5.3 HTML verified: sidebar navigation, search input, syntax highlighting (44 token spans), fixed layout with only main scrolling, back-to-top button
