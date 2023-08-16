use crate::config::{read_favorite, toggle_to_favorite};
use crate::mpris::{self, launch_mpris_server, Command, Response};
use crate::tools::{read_icons, StationsArtList};
use crate::ui::{render_help, render_stations};
use crate::{
    api::{now_playing, stations_list, Station},
    player::Player,
};
use crossbeam::channel;
use crossbeam::channel::Sender;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::random;
use std::fmt::{Display, Formatter};
use std::{
    io,
    process::exit,
    thread,
    time::{Duration, Instant},
};
use tui::{backend::CrosstermBackend, widgets::ListState, Terminal};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub enum Event {
    Input(KeyEvent),
    Tick,
    NowPlaying,
    Mpris(mpris::Command),
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

pub const TICK_RATE: Duration = Duration::from_millis(200);

pub struct Status {
    pub station: Station,
    pub playing: bool,
}

impl Status {
    pub fn mpris_playing(&self) -> String {
        if self.playing {
            "Playing".to_string()
        } else {
            "Stopped".to_string()
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let np = match self.playing {
            true => "Now playing",
            false => "Paused",
        };
        write!(f, "{} : {} ", np, self.station.title)
    }
}

pub struct App {
    stations_list_std: Vec<Station>,
    stations_list_fav: Vec<Station>,
    player: Player,
    pub icon_list: StationsArtList,
    active_context: Context,
    pub filtering: bool,
    pub music_title: String,
    pub stations_list_state: ListState,
    pub playing_station: Station,
    pub active_menu_item: MenuItem,
    pub filter: Input,
    last_selected: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        //try to get the stations list. Exit the program if impossible
        let stations_list_std = match stations_list() {
            Ok(list) => list,
            Err(_) => {
                eprintln!("No connection available !");
                exit(1)
            }
        };

        //Use standard stations list by default and try to fetch favorite list. If it exist, it will be used as default
        let mut active_menu_item = MenuItem::Standard(true);

        let stations_list_fav = match read_favorite() {
            Ok(list) => {
                if !list.is_empty() {
                    active_menu_item = MenuItem::Favorite(true)
                }
                list
            }
            Err(_) => Vec::new(),
        };

        //initiate the active list
        let mut stations_list_state = ListState::default();
        stations_list_state.select(Some(0));
        let playing_station = match active_menu_item {
            MenuItem::Favorite(_) => &stations_list_fav,
            MenuItem::Standard(_) => &stations_list_std,
        }[0]
        .clone();

        let input = Input::default();

        App {
            stations_list_std,
            stations_list_fav,
            player: Player::new(playing_station.stream_320.to_string()),
            icon_list: read_icons().expect("could not retrieve icons"),
            active_context: Context::Stations,
            music_title: String::from("Press n to show current song"),
            stations_list_state,
            playing_station,
            active_menu_item,
            filtering: false,
            filter: input,
            last_selected: None,
        }
    }

    pub fn get_stations_list(&self) -> Vec<&Station> {
        match self.active_menu_item {
            MenuItem::Favorite(_) => self.get_stations_list_fav(),
            _ => self.get_stations_list_std(),
        }
    }

    fn get_filtered_station<'a>(&'a self, stations: &'a [Station]) -> Vec<&Station> {
        let value = self.filter.value().to_lowercase();
        stations
            .iter()
            .filter(|s| {
                s.title.to_lowercase().contains(&value) || s.tooltip.to_lowercase().contains(&value)
            })
            .collect()
    }

    pub fn get_stations_list_std(&self) -> Vec<&Station> {
        self.get_filtered_station(&self.stations_list_std)
    }

    pub fn get_stations_list_fav(&self) -> Vec<&Station> {
        self.get_filtered_station(&self.stations_list_fav)
    }

    pub fn get_status(&self) -> Status {
        Status {
            station: self.playing_station.clone(),
            playing: self.player.is_playing(),
        }
    }

    pub fn get_selected_station(&self) -> Option<Station> {
        if let Some(selected) = self.stations_list_state.selected() {
            self.get_stations_list().get(selected).cloned().cloned()
        } else {
            None
        }
    }

    fn next(&mut self) {
        if let Some(selected) = self.stations_list_state.selected() {
            let amount_stations = self.get_stations_list().len();

            if selected >= amount_stations - 1 {
                // wrap to start
                self.stations_list_state.select(Some(0));
            } else {
                self.stations_list_state.select(Some(selected + 1));
            }
        }
    }

    fn previous(&mut self) {
        if let Some(selected) = self.stations_list_state.selected() {
            let amount_stations = self.get_stations_list().len();

            if selected > 0 {
                self.stations_list_state.select(Some(selected - 1));
            } else {
                // wrap to end
                self.stations_list_state.select(Some(amount_stations - 1));
            }
        }
    }
    fn toggle_context(&mut self) {
        self.active_menu_item = match self.active_menu_item {
            MenuItem::Favorite(b) => MenuItem::Favorite(!b),
            MenuItem::Standard(b) => MenuItem::Standard(!b),
        };
    }
    fn update_now_playing(&mut self) {
        #[cfg(feature = "libmpv_player")]
        {
            if self.get_status().playing {
                if let Some(title) = self.player.now_playing() {
                    self.music_title = title;
                }
            }
        }
        #[cfg(feature = "rodio_player")]
        {
            self.music_title = now_playing(self.playing_station.id).unwrap().to_string();
        }
    }

    pub async fn start(&mut self) -> color_eyre::Result<()> {
        println!("rrt is loading...");

        //prepare the terminal to be used
        enable_raw_mode().expect("can not run in raw mode");
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        //setup event emitter and receiver
        let (tx, rx) = channel::bounded(1);
        event_sender(tx.clone());

        let (player_tx, player_rx) = channel::bounded(1);

        let _conn = launch_mpris_server(tx, player_rx).await?;

        loop {
            //draw the corresponding context each tick
            terminal.draw(|rect| match self.active_context {
                Context::Help => render_help(rect, self),
                Context::Stations => render_stations(rect, self),
            })?;

            //wait for a tick or keyPress before continuing
            match rx.recv()? {
                Event::Input(event) => {
                    if self.filtering {
                        match event.code {
                            KeyCode::Esc => {
                                self.filter.reset();
                                if let Some(last_id) = self.last_selected {
                                    let pos = self
                                        .get_stations_list()
                                        .iter()
                                        .position(|s| s.id == last_id);
                                    self.stations_list_state.select(pos);
                                }
                                self.toggle_context();
                            }
                            KeyCode::Enter => {}
                            _ => {
                                self.filter.handle_event(&CEvent::Key(event));
                                continue;
                            }
                        }
                        self.filtering = !self.filtering;
                        continue;
                    }
                    match event.code {
                        KeyCode::Char('q') => {
                            let mut stdout = io::stdout();
                            stdout.execute(LeaveAlternateScreen)?;
                            disable_raw_mode()?;
                            terminal.show_cursor()?;
                            break;
                        }
                        KeyCode::Char('h') | KeyCode::Char('?') => {
                            self.active_context = Context::Help
                        }
                        KeyCode::Char('f') => {
                            if let Some(selected_station) = self.get_selected_station() {
                                self.stations_list_fav =
                                    toggle_to_favorite(&selected_station).expect("can add to fav");

                                if self.stations_list_fav.is_empty() {
                                    self.active_menu_item = MenuItem::Standard(true)
                                } else if let Some(selected) = self.stations_list_state.selected() {
                                    // if last move to previous
                                    if selected == self.get_stations_list().len() {
                                        self.stations_list_state.select(Some(selected - 1))
                                    }
                                }
                            }
                        }
                        KeyCode::Char('n') => self.update_now_playing(),
                        KeyCode::Char('N') => {
                            if let Some(selected_station) = self.get_selected_station() {
                                let id = selected_station.id;
                                self.music_title = now_playing(id).unwrap().to_string();
                            }
                        }
                        KeyCode::Char('r') => {
                            let random = random::<usize>() % self.get_stations_list().len();

                            let temp = self.get_stations_list()[random].clone();
                            let url = &temp.stream_320;

                            if self.player.force_play(url) {
                                self.playing_station = self.get_stations_list()[random].clone();
                                self.stations_list_state.select(Some(random));
                            }
                        }
                        KeyCode::Char(' ') => self.player.toggle_play(),
                        KeyCode::Enter => {
                            if let Some(selected_station) = self.get_selected_station() {
                                let same = self.playing_station == selected_station;

                                if !same {
                                    let url = &selected_station.stream_320;
                                    if self.player.force_play(url) {
                                        self.playing_station = selected_station.clone();
                                    }
                                } else {
                                    self.player.toggle_play()
                                }
                            }
                        }
                        KeyCode::Down => match self.active_menu_item {
                            MenuItem::Favorite(true) | MenuItem::Standard(true) => {
                                self.next();
                            }
                            _ => {
                                if !self.get_stations_list_std().is_empty() {
                                    self.active_menu_item = MenuItem::Standard(true);
                                    self.stations_list_state.select(Some(0));
                                }
                            }
                        },
                        KeyCode::Up => match self.active_menu_item {
                            MenuItem::Favorite(true) | MenuItem::Standard(true) => {
                                self.previous();
                            }
                            _ => {
                                if !self.get_stations_list_fav().is_empty() {
                                    self.active_menu_item = MenuItem::Favorite(true);
                                    self.stations_list_state.select(Some(0));
                                };
                            }
                        },
                        KeyCode::Esc => match self.active_context {
                            Context::Help => self.active_context = Context::Stations,
                            Context::Stations => {
                                self.toggle_context();
                            }
                        },
                        KeyCode::Char('/') => {
                            if let Context::Stations = self.active_context {
                                self.filtering = !self.filtering;
                                self.toggle_context();
                                self.last_selected = self.get_selected_station().map(|s| s.id);
                            }
                        }

                        _ => {}
                    }
                }

                Event::Tick => {}
                Event::NowPlaying => self.update_now_playing(),
                Event::Mpris(event) => {
                    match event {
                        Command::PlayPause => self.player.toggle_play(),
                        Command::Stop => self.player.stop(),
                        Command::Play => self.player.resume(),
                        Command::Next => {
                            self.next();
                            let station = self.get_selected_station().unwrap();
                            if self.player.force_play(&station.stream_320) {
                                self.playing_station = station;
                            }
                        }
                        Command::Previous => {
                            self.previous();
                            let station = self.get_selected_station().unwrap();
                            if self.player.force_play(&station.stream_320) {
                                self.playing_station = station;
                            }
                        }
                        Command::NowPlaying => {
                            player_tx
                                .send(Response::NowPlaying(self.music_title.to_string()))
                                .unwrap();
                        }
                        Command::Status => {
                            player_tx
                                .send(Response::Status(self.get_status().mpris_playing()))
                                .unwrap();
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/**
Capture and resend key press as well as sending tick for refresh
 */
fn event_sender(tx: Sender<Event>) {
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        #[cfg(feature = "libmpv_player")]
        let mut tick_to_playing = 20;
        loop {
            let timeout = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= TICK_RATE && tx.send(Event::Tick).is_ok() {
                #[cfg(feature = "libmpv_player")]
                {
                    tick_to_playing -= 1;
                    if tick_to_playing <= 0 {
                        tick_to_playing = 4;
                        tx.send(Event::NowPlaying).expect("could not send");
                    }
                }

                last_tick = Instant::now();
            }
        }
    });
}
