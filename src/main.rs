mod api;
mod app;
mod config;
mod mpris;
mod player;
mod tools;
mod ui;

use crate::api::stations_list;
use crate::mpris::{launch_mpris_server, Response};
use crate::tools::pause;
use clap::{Arg, Command};
use crossbeam::channel;
use rand::random;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
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
                        s.push_str(&format!("{}, ", station.prefix));
                    } else {
                        s.push_str(&format!("{}\n", station.prefix));
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
                    } else {
                        panic!("Station not found")
                    }
                } else {
                    // play random station
                    let random = random::<usize>() % list.len();
                    let station = &list[random];
                    println!("Now playing : {}", station.title);
                    player.play(&station.stream_320);
                }
                // launch and handle mpris interface
                let (mpris_tx, mpris_rx) = channel::bounded(1);
                let (tx, rx) = channel::bounded(1);

                let _conn = launch_mpris_server(mpris_tx, rx).await?;

                thread::spawn(move || loop {
                    if mpris_rx.is_empty() {
                        thread::sleep(Duration::from_millis(200));
                        continue;
                    }
                    if let app::Event::Mpris(event) = mpris_rx.recv().unwrap() {
                        match event {
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
                        mpris::Command::NowPlaying => {
                            #[cfg(feature = "libmpv_player")]
                            {
                                if let Some(title) = player.now_playing() {
                                    tx.send(Response::NowPlaying(title)).unwrap();
                                }
                            }
                        }
                        mpris::Command::Status => {
                            let status = if player.is_playing() {
                                "Playing"
                            } else {
                                "Stopped"
                            };
                            tx.send(Response::Status(status.to_string())).unwrap();
                        }
                    }
                    };
                });
                pause();
            }
            &_ => {
                eprint!("Command not found")
            }
        }
        Ok(())
    } else {
        // launch the tui app
        app::App::new().start().await
    }
}
