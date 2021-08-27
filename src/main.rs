mod player;
mod api;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    ui::start_ui()
}
