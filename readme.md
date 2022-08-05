# RadioRecord-tui

A simple terminal interface for listening to radio record web stations written in Rust.\
You can listen to all available record stations and mark some as favorite for ease.
![ui](https://gitlab.com/vandechat96/radiorecord-tui/-/raw/master/screenshots/ui.png)

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
