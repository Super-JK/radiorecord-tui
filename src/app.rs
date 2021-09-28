use tui::{
    backend::CrosstermBackend,
    widgets::ListState,
    Terminal,
};

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde_json::Value;

use std::{
    io,
    sync::mpsc::{self,Sender},
    thread,
    time::{Duration, Instant},
    process::exit,
};
use crate::{
    api::{
        Station,
        now_playing,
        radio_list
    },
    player::Player,
};
use crate::config::{
    read_favorite, toggle_to_favorite, read_icons
};
use crate::ui::{
    render_stations, render_help
};

pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub enum Context {
    Stations,
    Help,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Favorite(bool),
    Standard(bool),
}

pub struct App{
    pub stations_list_std:Vec<Station>,
    pub stations_list_fav:Vec<Station>,
    pub stations_list:Vec<Station>,
    player:Player,
    pub icon_list:Value,
    active_context: Context,
    pub music_title:String,
    pub stations_list_state:ListState,
    pub playing_station:Station,
    pub active_menu_item: MenuItem,

}

impl App{
    pub fn new() -> App {
        //try to get the stations list. Exit the program if impossible
        let stations_list_std = match radio_list(){
            Ok(list)=>list,
            Err(_)=> {
                eprintln!("No connection available !");
                exit(1) },
        };

        //Use standard stations list by default and try to fetch favorite list. If it exist, it will be used as default
        let mut stations_list = stations_list_std.clone();
        let mut active_menu_item = MenuItem::Standard(true);

        let stations_list_fav = match read_favorite() {
            Ok(list)=>{
                if list.len() > 0 {
                    stations_list = list.clone();
                    active_menu_item = MenuItem::Favorite(true)
                }
                list
            },
            Err(_)=> Vec::new(),
        };

        //initiate the active list
        let mut stations_list_state = ListState::default();
        stations_list_state.select(Some(0));
        let playing_station = stations_list[0].clone();

        App{
            stations_list_std,
            stations_list_fav,
            stations_list,
            player : Player::new(),
            icon_list: read_icons().expect("could not retrieve icons"),
            active_context: Context::Stations,
            music_title: String::from("Press n to show current song"),
            stations_list_state,
            playing_station,
            active_menu_item
        }
    }

    pub fn get_stations_list(&self) -> &Vec<Station>{
        match self.active_menu_item {
            MenuItem::Favorite(_) => &self.stations_list_fav,
            _ => &self.stations_list_std,
        }
    }

    pub fn get_status(&self)->Vec<&str>{
        let playing = match self.player.is_playing() {
            true=>"Playing",
            false=>"Paused",
        };

        vec![self.playing_station.title.as_str(),playing]
    }


pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>{

    //prepare the terminal to be used
    enable_raw_mode().expect("can not run in raw mode");
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //setup event emitter and receiver
    let (tx, rx) = mpsc::channel();
    event_sender(tx);

    loop {
        //draw the corresponding context each tick
        terminal.draw(|rect| {
            match self.active_context {
                Context::Help => render_help(rect, self),
                Context::Stations =>render_stations(rect, self)
            }
        })?;

        //wait for a tick or keyPress before continuing
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') | KeyCode::Char('?')=> self.active_context = Context::Help,
                KeyCode::Char('f') => {
                    if let Some(selected) = self.stations_list_state.selected() {
                        self.stations_list_fav = toggle_to_favorite(self.stations_list[selected].clone()).expect("can add to fav");
                    }
                }
                KeyCode::Char('n') => {
                    if self.player.is_playing() {
                        self.music_title = now_playing(self.playing_station.id).unwrap().to_string();
                    }
                }
                KeyCode::Char('N') => {
                    if let Some(selected) = self.stations_list_state.selected() {
                        let id = self.stations_list[selected].id;
                        self.music_title = now_playing(id).unwrap().to_string();
                    };
                }
                KeyCode::Char(' ') => {
                    self.toggle_playing()
                }
                KeyCode::Enter =>{
                    if let Some(selected) = self.stations_list_state.selected() {
                        let same = self.playing_station == self.stations_list[selected];

                        if !same {
                            self.playing_station = self.stations_list[selected].clone();

                            self.player.stop();

                            thread::sleep(Duration::from_millis(200));

                            let url = self.playing_station.clone().stream_320;
                            self.player.play(url.clone());
                        } else {
                            self.toggle_playing()
                        }
                    }
                }
                KeyCode::Down => {
                    match self.active_menu_item {
                        MenuItem::Favorite(true) | MenuItem::Standard(true) => {
                            if let Some(selected) = self.stations_list_state.selected() {
                                let amount_stations = match self.active_menu_item {
                                    MenuItem::Favorite(true) => { self.stations_list_fav.len() },
                                    _ => self.stations_list_std.len(),
                                };
                                if selected >= amount_stations - 1 {
                                    self.stations_list_state.select(Some(0));
                                } else {
                                    self.stations_list_state.select(Some(selected + 1));
                                }
                            }
                        },
                        _=>{
                            self.active_menu_item = MenuItem::Standard(true);
                            self.stations_list= self.stations_list_std.clone();
                            self.stations_list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Up => {
                    match self.active_menu_item {

                        MenuItem::Favorite(true) | MenuItem::Standard(true)=>{
                            if let Some(selected) = self.stations_list_state.selected() {
                                let amount_stations = match self.active_menu_item {
                                    MenuItem::Favorite(true) =>{ self.stations_list_fav.len()},
                                    _=>self.stations_list_std.len(),
                                };
                                if selected > 0 {
                                    self.stations_list_state.select(Some(selected - 1));
                                } else {
                                    self.stations_list_state.select(Some(amount_stations - 1));
                                }
                            }
                        }
                        _=>{
                            if self.stations_list_fav.len() > 0 {
                                self.active_menu_item = MenuItem::Favorite(true);
                                self.stations_list= self.stations_list_fav.clone();
                                self.stations_list_state.select(Some(0));
                            };
                        },
                    }
                }
                KeyCode::Esc=>{
                    match self.active_context {
                        Context::Help => self.active_context = Context::Stations,
                        Context::Stations =>self.active_menu_item = match self.active_menu_item {
                            MenuItem::Favorite(true) => {MenuItem::Favorite(false)}
                            MenuItem::Standard(true) => {MenuItem::Standard(false)}
                            _ => {MenuItem::Standard(false)}
                        },
                    }
                }
                _ => {}
            },

            Event::Tick => {}
        }
    }
    Ok(())

}
    //start or stop the current radio
    fn toggle_playing(&mut self){
        let playing = self.player.is_playing();
        if playing {
            self.player.stop()
        } else {
            let url = self.playing_station.clone().stream_320;
            self.player.play(url.clone());
        }
    }
}

/**
Capture and resend key press as well as sending tick for refresh
 */
fn event_sender(tx:Sender<Event<KeyEvent>>){
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });
}
