<img src="https://demille.github.io/encrusted/src/img/name.svg" alt="encrusted" width="200px" height="78px" align="left" />

<p align="right">
  <img src="https://img.shields.io/crates/v/encrusted.svg" alt="Crates.io" align="right" />
  <br/>
  <a href="https://travis-ci.org/DeMille/encrusted">
    <img src="https://travis-ci.org/DeMille/encrusted.svg?branch=master" alt="Built Status" align="right" />
  </a>
</p>
<br/>

---

#### A z-machine (interpreter) for Infocom-era text adventure games like Zork

Runs in a web interface or directly in a terminal.
Built with Rust and WebAssembly (`wasm32-unknown-unknown`).

🎮 &nbsp;[Launch the web player][web]

<br/>

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
cargo install encrusted --bin encrusted
```

Run a file with `encrusted <FILE>`.
Use `$undo` and `$redo` to step through your move history.
Use `save` and `restore` to save your progress.


### Build
WebAssembly/React web version (requires node & rust nightly):

```sh
# If you haven't added nightly or the wasm32 target:
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly

# Runs webpack dev server on port 8000
npm run dev

# Build .wasm module with rust nightly, debug mode
npm run build:debug

# Or build all in release mode & bundle JS into the ./build directory
npm run release
```


### Tests

Run z-machine tests ([czech](https://inform-fiction.org/zmachine/standards/z1point1/appc.html) & [praxix](https://inform-fiction.org/zmachine/standards/z1point1/appc.html)) through [regtest](https://eblong.com/zarf/plotex/regtest.html):
```
npm run test
```


### Notes
- Currently only supports v3 zcode files
- Saves games in the Quetzal format


### License
MIT
