mod player;
mod api;
mod ui;
mod config;
mod tools;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("rrt is loading...");
    ui::start_ui()
}
