# RadioRecord-tui
⚠️**This project is in very early stage and contains a lot of bugs (more on that later)**

A simple terminal interface for listening to radio record web station.

## Build and run
You first have to install [Rust](https://www.rust-lang.org/tools/install) ( usage of `rustup` is recommended )
```bash
cargo run
```
You can also install it with ;
```bash
cargo install
```
However, the program will crash for now if it doesn't find the ascii.json file at the right location.

## Bugs and enhancements

Like said previously, there is a lot to do in this project. It this poorly written and has a lot if bugs.

### List of known issues/thing to do
- Sometimes the radio won't start and if you try a second time it will play it two time
- Program crash when ascii.json is not found (Create auto gen with tools.rs)
- Better handling of the various exception (Especially within the "api")
- Rewrite the ui part to make it more clear and clean
- Find where come the thread that doesn't dye
- Find/make a better audio backend/player due to poor audio quality
- Write documentation
- Remove the need to write the stream to a tempfile.