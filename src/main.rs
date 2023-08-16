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
use clap::{Parser, Subcommand};
use crossbeam::channel;
use rand::random;
use std::process::exit;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
      
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List available stations
    List{
        #[arg(short, long, default_value_t = false)]
        line: bool, 
    },
    /// Play the specified station (random if none)
    Play{
        #[arg(short, long)]
        station:Option<String>,
    },
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    
    if let Some(cmd) = cli.command {
        let list = stations_list().unwrap();
        match cmd {
           Commands::List{line} => {
                let mut s = String::new();
                for station in list {
                    if line {
                        s.push_str(&format!("{}, ", station.prefix));
                    } else {
                        s.push_str(&format!("{} : {}\n", station.title, station.prefix));
                    }
                }
                println!("{}", s);
            }
            Commands::Play{station} => {
                // background player in cli

                let mut player = player::Player::new(list[0].stream_320.clone());
                if let Some(station) = station {
                    // if a station is selected play it
                    if let Some(station_found) = list.iter().find(|s| s.prefix == station) {
                        player.play(&station_found.stream_320);
                    } else {
                        eprintln!("Station not found");
                        exit(1);
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
        }
        Ok(())
    } else {
        // launch the tui app
        app::App::new().start().await
    }
}
