use tui::{
    layout::{
        Alignment, Constraint, Direction, Layout, Rect
    },
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Frame,
    backend::Backend
};

use serde_json::Value;

use crate::{
    api::Station,
    app::{App, MenuItem},
};

//const ACCENT_COLOR:Color = Color::Rgb(255,96,0);
const ACCENT_COLOR:Color = Color::Yellow;

/**
Display the help menu on the terminal
 */
pub fn render_help<B>(rect: &mut Frame<B>, _app: &App)
    where
        B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(2)
        .split(rect.size());

    rect.render_widget(help_paragraph(), chunks[0])
}
/**
Display the main menu on the terminal

It used corresponding function  to generate each part and split the terminal into different zones
 */
pub fn render_stations<B>(rect: &mut Frame<B>, app: &mut App) where B: Backend,
{
    //get base layout
    let chunks = base_chunk(rect.size());

    //add the status bar
    let bar = status_bar(&app.get_status());
    rect.render_widget(bar, chunks[0]);

    //split the rect
    let stations_chunks = split_horizontal_chunk(chunks[1]);
    let stations_list_chunks = split_vertical_chunk(stations_chunks[0]);

    //generate the stations lists
    let list_std = make_std_stations_list(&app.stations_list_std,  &app.active_menu_item);
    let list_fav = make_fav_stations_list(&app.stations_list_fav, &app.active_menu_item);

    //add the stations list. Only the active list is navigable
    match app.active_menu_item {
        MenuItem::Favorite(true)=>{
            rect.render_stateful_widget(list_fav, stations_list_chunks[0], &mut app.stations_list_state);
            rect.render_widget(list_std, stations_list_chunks[1]);
        },
        MenuItem::Standard(true)=>{
            rect.render_widget(list_fav, stations_list_chunks[0]);
            rect.render_stateful_widget(list_std, stations_list_chunks[1], &mut app.stations_list_state);
        },
        _=>{
            rect.render_widget(list_fav, stations_list_chunks[0]);
            rect.render_widget(list_std, stations_list_chunks[1]);
        }
    }

    //add the detail, icons and footer
    let detail_chunks = split_detail_chunk(stations_chunks[1]);

    let detail = station_detail(&app.stations_list_state, app.get_stations_list());
    rect.render_widget(detail, detail_chunks[0]);

    let icon = make_icon(&app.icon_list, &app.stations_list_state, app.get_stations_list(), detail_chunks[1].height);
    rect.render_widget(icon, detail_chunks[1]);

    let footer = footer(&app.music_title);
    rect.render_widget(footer, chunks[2]);
}
/**
Split a Rect into two Rect horizontally (20% - 80%)
 */
fn split_horizontal_chunk(chunk:Rect)->Vec<Rect>{
    split_chunk(chunk,Direction::Horizontal)
}
/**
Split a Rect into two Rect vertically (20% - 80%)
 */
fn split_vertical_chunk(chunk:Rect)->Vec<Rect>{
    split_chunk(chunk,Direction::Vertical)
}
/**
Split a Rect into two Rect in a given direction (20% - 80%)
 */
fn split_chunk(chunk:Rect,dir:Direction)->Vec<Rect>{
    Layout::default()
        .direction(dir)
        .constraints(
            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
        )
        .split(chunk)
}
/**
Split a rect to maximize the size of icons
*/
fn split_detail_chunk(chunk:Rect) ->Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Min(2)
            ].as_ref(),
        )
        .split(chunk)
}
/**
Split a Rect into 3 pieces (Default Layout)
 */
fn base_chunk(size:Rect)->Vec<Rect>{
    Layout::default()
        .direction(Direction::Vertical)
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
/**
Paragraph displaying currently playing song
 */
fn footer(title: &String) ->Paragraph {
    Paragraph::new(title.as_str())
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Reset))
                .title("Now Playing"),
        )
}
/**
Tab indicating the status of the player
 */
fn status_bar<'a>(status:&Vec<&'a str>) ->Tabs<'a>{
    let status_vec = status
        .iter()
        .map(|t| {
            Spans::from(vec![
                Span::styled(*t, Style::default()),
            ])
        })
        .collect();

    Tabs::new(status_vec)
        .block(
            Block::default().title("Status")
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
        )
        .style(Style::default())
        .divider(Span::raw("|"))
}
/**
Paragraph for the help menu
 */
fn help_paragraph<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::styled(format!("{:50}{:40}", "Description", "Key"),Style::default().fg(Color::Red),)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Change station list", "<Esc>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Go to Help", "h"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Move up", "<Up Arrow key>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Move down", "<Down Arrow Key>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Change station", "<Enter>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Play/pause station", "<Space>"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Add/remove from favorite", "f"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Get current playing song", "n"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Get current playing song on the selected station", "N"))]),
        Spans::from(vec![Span::raw(format!("{:50}{:40}", "Quit program", "q"))]),
    ])
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default())
                .title("Help (<Esc> to quit)"),
        );
    home
}
/**
She standard station list as a List with the correct style to be displayed
 */
fn make_std_stations_list<'a>(stations_list:&Vec<Station>,menu_item:&MenuItem)-> List<'a>{
    let style = match menu_item{
        MenuItem::Standard(_) => Style::default().fg(ACCENT_COLOR),
        _ => Style::default(),
    };
    make_stations_list(stations_list,"Stations",style)
}
/**
She favorite station list as a List with the correct style to be displayed
 */
fn make_fav_stations_list<'a>(stations_list:&Vec<Station>,menu_item:&MenuItem)-> List<'a>{
    let style = match menu_item{
        MenuItem::Favorite(_) => Style::default().fg(ACCENT_COLOR),
        _ => Style::default(),
    };
    make_stations_list(stations_list,"Favorites",style)
}

/**
Generate the stations list based on the stations names
 */
fn make_stations_list<'a>(stations_list:&Vec<Station>, title:&'a str, style:Style) -> List<'a> {

    let stations = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(style);

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
            .bg(ACCENT_COLOR)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
}
/**
Paragraph with the stations icon, size depends on available space
 */
fn make_icon<'a>(icon_list: &Value, stations_list_state: &ListState, stations_list:&Vec<Station>, mut size:u16) -> Paragraph<'a> {
    let selected_station = stations_list
        .get(
            stations_list_state
                .selected()
                .expect("there is always a selected station"),
        )
        .expect("exists")
        .clone();

    if size >= 30 {
        size = 60
    } else { size = 30 }

    let name = format!("{}_{}",&selected_station.prefix,size);

    let icon = match icon_list[name].as_str() {
        None => "no_icon",
        Some(icon)=>icon
    };
    Paragraph::new(format!("{}",icon))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default())
        )
}
/**
Details about the stations as a Table
 */
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
                "Description",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Detail")
                .border_type(BorderType::Rounded),
        )
        .widths(&[
            Constraint::Percentage(5),
            Constraint::Percentage(15),
            Constraint::Percentage(80),
        ])
}



