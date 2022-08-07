mod api;
mod app;
mod config;
mod mpris;
mod player;
mod tools;
mod ui;

use crate::api::stations_list;
use crate::mpris::launch_mpris_server;
use crate::tools::pause;
use clap::{Arg, Command};
use crossbeam::channel;
use rand::random;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        // Safe, because we checked if the subcommand is present at runtime
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
                // background player in cli

                let mut player = player::Player::new(list[0].stream_320.clone());
                if let Some(st) = m.value_of("station") {
                    // if a station is selected play it
                    if let Some(station_found) = list.iter().find(|station| station.prefix == st) {
                        player.play(&station_found.stream_320);
                        pause()
                    } else {
                        panic!("Station not found")
                    }
                } else {
                    // play random station
                    let random = random::<usize>() % list.len();
                    let station = &list[random];
                    println!("Now playing : {}", station.title);
                    player.play(&station.stream_320);
                    pause()
                }
                // launch and handle mpris interface
                let (tx, rx) = channel::bounded(1);
                launch_mpris_server(tx).await?;
                thread::spawn(move || loop {
                    if rx.is_empty() {
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                    match rx.recv().unwrap() {
                        mpris::Command::PlayPause => player.toggle_play(),
                        mpris::Command::Stop => player.stop(),
                        mpris::Command::Play => player.resume(),
                        mpris::Command::Next => {
                            let random = random::<usize>() % list.len();
                            let station = &list[random];
                            println!("Now playing : {}", station.title);
                            player.force_play(&station.stream_320);
                        }
                        mpris::Command::Previous => {}
                    };
                });
            }
            &_ => {
                panic!("Command not found")
            }
        }
        Ok(())
    } else {
        // launch the tui app
        app::App::new().start().await
    }
}
