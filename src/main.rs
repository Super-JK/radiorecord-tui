mod player;
mod api;
mod ui;
mod config;
mod tools;
mod app;

use clap::{App, Arg};

fn main() -> Result<(), Box<dyn std::error::Error>>{

    let matches = App::new("Radio Record tui")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        //.usage("Press `h` while running the app to see keybindings")
        .subcommand(App::new("list")
            .about("List available stations")
            .arg(
                Arg::with_name("line")
                    .help("Display the list in one line")
                    .value_name("line")
                    .long("line")
                    .short("l")
                    .required(false)
                    .takes_value(false)
            )
        )
        .subcommand(App::new("play")
            .about("Play stream from chosen station")
            .arg(
                Arg::with_name("station")
                    .help("Station to play")
                    .value_name("station")
                    .long("station")
                    .short("s")
                    .takes_value(true)
                    .required(true)
            )
        )
        .get_matches();

    if let Some(cmd) = matches.subcommand_name() {
        // Save, because we checked if the subcommand is present at runtime
        let m = matches.subcommand_matches(cmd).unwrap();
        let list = api::radio_list().unwrap();
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
                println!("{}",s);
            },
            "play" => {
                if let Some(st) = m.value_of("station"){
                    if let Some(station_found) = list.iter().find(|station| station.prefix == st) {
                        let mut player = player::Player::new();
                        player.play(station_found.stream_320.clone());
                        loop {
                        }
                    } else {
                        panic!("Station not found")
                    }
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
