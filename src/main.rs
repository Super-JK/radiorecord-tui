mod api;
mod app;
mod config;
mod player;
mod tools;
mod ui;

use clap::{Arg, Command};
use rand::random;
use crate::api::stations_list;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Radio Record tui")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        //.usage("Press `h` while running the app to see keybindings")
        .subcommand(
            Command::new("list").about("List available stations").arg(
                Arg::new("line")
                    .help("Display the list in one line")
                    .value_name("line")
                    .long("line")
                    .short('l')
                    .required(false)
                    .takes_value(false),
            ),
        )
        .subcommand(
            Command::new("play")
                .about("Play stream from chosen station")
                .arg(
                    Arg::new("station")
                        .help("Station to play")
                        .value_name("station")
                        .long("station")
                        .short('s')
                        .takes_value(true)
                        .required(false),
                ),
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_name() {
        // Save, because we checked if the subcommand is present at runtime
        let m = matches.subcommand_matches(cmd).unwrap();
        let list = stations_list().unwrap();
        match cmd {
            "list" => {
                let mut s = String::new();
                let line = m.is_present("line");
                for station in list {
                    if line {
                        s.push_str(&*format!("{}, ", station.prefix));
                    } else {
                        s.push_str(&*format!("{}\n", station.prefix));
                    }
                }
                println!("{}", s);
            }
            "play" => {
                if let Some(st) = m.value_of("station") {
                    if let Some(station_found) = list.iter().find(|station| station.prefix == st) {
                        let mut player = player::Player::new();
                        player.play(&station_found.stream_320);
                        pause()
                    } else {
                        panic!("Station not found")
                    }
                } else {
                    let mut player = player::Player::new();
                    let random= random::<usize>()  % list.len();
                    let station = &list[random];
                    println!("Now playing : {}",station.title);
                    player.play(&station.stream_320);
                    pause()
                }
            }
            &_ => {
                panic!("Command not found")
            }
        }
        Ok(())
    } else {
        app::App::new().start()
    }
}

use std::io;
use std::io::prelude::*;

fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}
