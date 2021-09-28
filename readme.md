# RadioRecord-tui

A simple terminal interface for listening to radio record web stations written in Rust.\
You can listen to all available record stations and mark some as favorite for ease.
![ui](/https://gitlab.com/vandechat96/radiorecord-tui/-/raw/master/screenshots/ui.png)
Exact color depend on your terminal theme

*This project is still in development and there is a lot of things to do*
## Build and run
You first have to install [Rust](https://www.rust-lang.org/tools/install) ( usage of `rustup` is recommended )

Run without optimizations :
```bash
cargo run
```
You can also install it with :
```bash
cargo install --path .
```

#### Audio Player
By default, the player use libmpv (better audio) but you can specify to use rodio with `--features rodio_player --no-default-features`

## List of known issues/thing to do
- Sometimes the radio won't start and if you try a second time it will play it two time
- ~~Program crash when ascii.json is not found (Create auto gen with tools.rs)~~
- Better handling of the various exception (Especially within the "api")
- Rewrite the ui part to make it more clear and clean. (Partially done)
- ~~Find where come the thread that doesn't die~~
- ~~Find/make a better audio backend/player due to poor audio quality~~
- Write helpful documentation
- Remove the need to write the stream to a tempfile.