mod player;
mod api;
mod ui;
mod config;
mod tools;
mod app;

use crate::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("rrt is loading...");
    App::new().start()
}
