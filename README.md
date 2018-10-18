<img src="https://demille.github.io/encrusted/src/img/name.svg" alt="encrusted" width="200px;"/>

---

#### A z-machine (interpreter) for Infocom-era text adventure games like Zork

Runs in a web interface or directly in a terminal.  
Built with Rust and WebAssembly (`wasm32-unknown-unknown`).

🎮 [Load the web version][web]

**Features**
- [x] Live mapping to keep track of where you are
- [x] Undo / Redo support
- [x] Narration / Dictation using the [web speech APIs][APIs]
- [x] Object tree inspector

[web]: https://sterlingdemille.com/encrusted
[APIs]: https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API


### Install
Terminal version:

```sh
cargo install encrusted
```

Run a file with `encrusted <FILE>`.  
Use `$undo` and `$redo` to step through your move history.  
Use `save` and `restore` to save your progress.


### Build
WebAssembly/React web version (requires node & rust nightly):

```sh
# Runs webpack dev server on port 8000
npm run dev

# Build .wasm module with rust nightly, debug mode
npm run build:debug

# Or build all in release mode & bundle JS into the ./build directory
npm run release
```


### Tests
[![Build Status](https://travis-ci.org/DeMille/encrusted.svg?branch=master)](https://travis-ci.org/DeMille/encrusted)

Crude testing by running the [czech unit tests](https://inform-fiction.org/zmachine/standards/z1point1/appc.html):
```
npm run test
```


### Notes
- Currently only supports v3 zcode files
- Saves games in the Quetzal format


### License
MIT
