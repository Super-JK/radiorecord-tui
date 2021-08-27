use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint,Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,Tabs,
    },
    Terminal,
};

use crate::api::{Station, now_playing};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::{io, fs};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use crate::{api, player};
use tui::layout::Rect;
use thiserror::Error;
use serde_json::Value;
use std::process::exit;

const FAV_PATH: &str = "./data/fav.json";
const ACCENT_COLOR:Color = Color::Rgb(255,96,0);

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Stations,
    Help,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Stations => 0,
            MenuItem::Help => 1,
        }
    }
}
#[derive(Copy, Clone, Debug)]
enum Context {
    Change,
    Favorite,
    Standard,
}

#[derive(Clone, Debug)]
struct Status{
    station:String,
    playing:bool,
}

impl Status{
    pub fn new(station:String,playing:bool)->Status{
        Status{
            station,
            playing,
        }
    }
    pub fn to_vec(&self)->Vec<&str>{
        let playing = match self.playing {
            true=>"Playing",
            false=>"Paused",
        };

        vec![self.station.as_str(),playing]
    }

    pub fn set_station(&mut self, station:String){
        self.station=station;
    }
    pub fn set_playing(&mut self, playing:bool){
        self.playing=playing;
    }


}

pub fn start_ui() -> Result<(), Box<dyn std::error::Error>>{
    let stations_list_std =match  api::radio_list(){
        Ok(list)=>list,
        Err(_)=> {
            eprintln!("No connection available !");
            exit(1) },
    };

    let stations_len = stations_list_std.len();
    let mut stations_list_fav = match read_favorite(){
        Ok(list)=>list,
        Err(_)=> Vec::new(),
    };

    let mut stations_list = stations_list_fav.clone();
    let mut player = player::Player::new();

    let icon_list:Value = serde_json::from_str( fs::read_to_string("./data/ascii.json").unwrap().as_str()).unwrap();


    enable_raw_mode().expect("can not run in raw mode");

    let (tx, rx) = mpsc::channel();
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

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;


    let mut active_menu_item = MenuItem::Stations;
    let mut active_context = Context::Favorite;
    let mut stations_list_state = ListState::default();
    let selected = 0;
    stations_list_state.select(Some(selected));

    let mut selected_station = stations_list[selected].clone();
    let mut title = String::from("Nothing Playing");
    let mut status = Status::new(selected_station.title.to_string(), player.is_playing());

    loop {
        terminal.draw(|rect| {
            let size = rect.size();

            let chunks = base_chunk(size);

            let footer = footer(&title);

            let status_menu = generate_status(&status.to_vec());

            let bar = status_bar(status_menu);

            rect.render_widget(bar, chunks[0]);

            match active_menu_item {
                MenuItem::Help => rect.render_widget(render_help(), chunks[1]),
                MenuItem::Stations => {
                    let stations_chunk = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let stations_chunk_test = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(stations_chunk[0]);
                    let list = render_stations_list(&stations_list_std,false);
                    let detail = station_detail(&stations_list_state, &stations_list);
                    let list_fav = render_stations_list( &stations_list_fav,true);

                    match active_context {
                        Context::Favorite=>{
                            rect.render_stateful_widget(list_fav, stations_chunk_test[0], &mut stations_list_state);
                            rect.render_widget(list, stations_chunk_test[1]);
                        },
                        Context::Standard=>{
                            rect.render_widget(list_fav, stations_chunk_test[0]);
                            rect.render_stateful_widget(list, stations_chunk_test[1], &mut stations_list_state);
                        },
                        Context::Change=>{
                            rect.render_widget(list_fav, stations_chunk_test[0]);
                            rect.render_widget(list, stations_chunk_test[1]);
                        }
                    }
                    //rect.render_widget(list_fav, stations_chunk_test[0]);
                    //rect.render_stateful_widget(list, stations_chunk_test[1], &mut stations_list_state);
                    let detail_chunk = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(stations_chunk[1]);
                    rect.render_widget(detail, detail_chunk[0]);
                    let icon = render_icon(&icon_list,&stations_list_state, &stations_list);
                    rect.render_widget(icon, detail_chunk[1]);

                }
            }

            rect.render_widget(footer, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Help,
                KeyCode::Char('s') => active_menu_item = MenuItem::Stations,
                KeyCode::Char('f') => {
                    if let Some(selected) = stations_list_state.selected() {
                        stations_list_fav = toggle_to_favorite(stations_list[selected].clone()).expect("can add to fav");
                    }

                }
                KeyCode::Char('n') => {
                    title = now_playing(selected_station.id).unwrap()[0].to_string();

                }
                KeyCode::Char('N') => {
                    if let Some(selected) = stations_list_state.selected() {
                        let id = stations_list[selected].id;
                        title = now_playing(id).unwrap()[0].to_string();
                    };

                }
                KeyCode::Char(' ') => {
                    if let Some(selected) = stations_list_state.selected() {
                        let same = selected_station == stations_list[selected];
                        selected_station = stations_list[selected].clone();

                        let playing = player.is_playing();
                        if (playing && same) || !same {
                            player.stop()
                        }
                        thread::sleep(Duration::from_millis(200));
                        if (same && !playing ) || !same {
                            let url = selected_station.clone().stream_320;
                            player.play(url.clone());
                        }
                        status.set_station(selected_station.title.to_string());
                        status.set_playing(player.is_playing())
                    }
                }
                KeyCode::Down => {
                    match active_context{
                        Context::Change=>{
                            active_context=Context::Standard;
                            stations_list= stations_list_std.clone();
                            stations_list_state.select(Some(0));
                        },
                        _=> {
                            if let Some(selected) = stations_list_state.selected() {
                                let amount_stations = match active_context {
                                    Context::Favorite => { stations_list_fav.len() },
                                    _ => stations_len,
                                };
                                if selected >= amount_stations - 1 {
                                    stations_list_state.select(Some(0));
                                } else {
                                    stations_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                    }

                }
                KeyCode::Up => {
                    match active_context{
                        Context::Change=>{
                            active_context=Context::Favorite;
                            stations_list= stations_list_fav.clone();
                            stations_list_state.select(Some(0));
                        },
                        _=>{
                            if let Some(selected) = stations_list_state.selected() {
                                let amount_stations = match active_context{
                                    Context::Favorite=>{ stations_list_fav.len()},
                                    _=>stations_len,
                                };
                                if selected > 0 {
                                    stations_list_state.select(Some(selected - 1));
                                } else {
                                    stations_list_state.select(Some(amount_stations - 1));
                                }
                            }
                        }
                    }

                }
                KeyCode::Esc=>{
                    active_context=Context::Change;
                }
                _ => {}
            },

            Event::Tick => {}
        }
    }
    Ok(())

}

fn base_chunk(size:Rect)->Vec<Rect>{
    Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
                .as_ref(),
        )
        .split(size)
}

fn footer(title: &String) ->Paragraph {
    Paragraph::new(title.as_str())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Now Playing")
                .border_type(BorderType::Plain),
        )
}

fn generate_status<'a>(status:&Vec<&'a str>) ->Vec<Spans<'a>>{
    status
        .iter()
        .map(|t| {
            //let (first, rest) = t.split_at(1);
            Spans::from(vec![
                /*Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::UNDERLINED),
                ),

             */
                Span::styled(*t, Style::default().fg(Color::Gray)),
            ])
        })
        .collect()
}

fn status_bar(status:Vec<Spans>) ->Tabs{
    Tabs::new(status)
        //.select(active_menu_item.into())
        .block(Block::default().title("Status").borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray))
        //.highlight_style(Style::default().fg(Color::Red))
        .divider(Span::raw("|"))
}

pub fn render_help<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::styled(format!("{:50}{:40}", "Description", "Key"),Style::default().fg(Color::Red),)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Go to Stations", "s"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Go to Help", "h"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Move up", "<Up Arrow key>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Move down", "<Down Arrow Key>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Play/pause station", "<Space>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Add/remove from favorite", "f"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Change station list", "<Esc>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Get current playing song", "n"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Get current playing song on the selected station", "N"))]),
    ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray))
                .title("Help")
                .border_type(BorderType::Plain),
        );
    home
}


pub fn render_stations_list<'a>(stations_list:&Vec<Station>,fav:bool) -> List<'a> {
    let title = match fav {
        true=>"Favorite",
        false=>"Stations"
    };
    let stations = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray))
        .title(title)
        .border_type(BorderType::Plain);

    let items: Vec<_> = stations_list
        .iter()
        .map(|station| {
            ListItem::new(Spans::from(vec![Span::styled(
                station.title.clone(),
                Style::default(),
            )]))
        })
        .collect();


     List::new(items).block(stations).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}

fn render_icon<'a>(icon_list: &Value, stations_list_state: &ListState, stations_list:&Vec<Station>) -> Paragraph<'a> {
    let selected_station = stations_list
        .get(
            stations_list_state
                .selected()
                .expect("there is always a selected station"),
        )
        .expect("exists")
        .clone();
    Paragraph::new(format!("{}",icon_list[selected_station.prefix].as_str().unwrap()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray))
                //.title("")
                .border_type(BorderType::Plain),
        )
}

fn station_detail<'a>(stations_list_state: &ListState, stations_list:&Vec<Station>)-> Table<'a>{

    let selected_station = stations_list
        .get(
            stations_list_state
                .selected()
                .expect("there is always a selected station"),
        )
        .expect("exists")
        .clone();
    Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_station.id.to_string())),
        Cell::from(Span::raw(selected_station.title)),
        Cell::from(Span::raw(selected_station.tooltip)),
        //Cell::from(Span::raw(selected_station.created_at.to_string())),
    ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "ID",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Title",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Tooltip",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            /*
           Cell::from(Span::styled(
               "Icon",
               Style::default().add_modifier(Modifier::BOLD),
           )),

           Cell::from(Span::styled(
               "Created At",
               Style::default().add_modifier(Modifier::BOLD),
           ))*/
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Detail")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(5),
            Constraint::Percentage(15),
            Constraint::Percentage(80),
            //Constraint::Percentage(50),
            //Constraint::Percentage(20),
        ])
}

fn toggle_to_favorite(station:Station) -> Result<Vec<Station>, Error> {
    let db_content = fs::read_to_string(FAV_PATH)?;
    let mut parsed: Vec<Station> = serde_json::from_str(&db_content)?;

    if  !parsed.contains(&station){
        parsed.push(station);
        fs::write(FAV_PATH, &serde_json::to_vec(&parsed)?)?;
    } else {
        let index = parsed.iter().position(|x| *x == station).unwrap();
        parsed.remove(index);
        fs::write(FAV_PATH, &serde_json::to_vec(&parsed)?)?;
    }
    Ok(parsed)

}

fn read_favorite() -> Result<Vec<Station>, Error> {
    let db_content = fs::read_to_string(FAV_PATH)?;
    let parsed: Vec<Station> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

