use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Points},
        Block, BorderType, Borders, List, ListItem, Paragraph,
    },
    Frame,
};

use crate::app::Status;
use crate::tools::StationsArtList;
use crate::{
    api::Station,
    app::{App, MenuItem},
};

//const ACCENT_COLOR:Color = Color::Rgb(255,96,0);
const ACCENT_COLOR: Color = Color::Yellow;

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
pub fn render_stations<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    //get base layout
    let chunks = base_chunk(rect.size());

    //add the status bar
    let bar = info_bar(app);
    rect.render_widget(bar, chunks[0]);

    //split the rect
    let stations_chunks = split_horizontal_chunk(chunks[1]);
    let stations_list_chunks = split_chunk(stations_chunks[0], Direction::Vertical, 30, 70);

    //generate the stations lists
    let list_std = make_std_stations_list(&app.get_stations_list_std(), &app.active_menu_item);
    let list_fav = make_fav_stations_list(&app.get_stations_list_fav(), &app.active_menu_item);

    //add the stations list. Only the active list is navigable
    match app.active_menu_item {
        MenuItem::Favorite(true) => {
            rect.render_stateful_widget(
                list_fav,
                stations_list_chunks[0],
                &mut app.stations_list_state,
            );
            rect.render_widget(list_std, stations_list_chunks[1]);
        }
        MenuItem::Standard(true) => {
            rect.render_widget(list_fav, stations_list_chunks[0]);
            rect.render_stateful_widget(
                list_std,
                stations_list_chunks[1],
                &mut app.stations_list_state,
            );
        }
        _ => {
            rect.render_widget(list_fav, stations_list_chunks[0]);
            rect.render_widget(list_std, stations_list_chunks[1]);
        }
    }

    //add the icons and footer
    make_icon(
        rect,
        &stations_chunks[1],
        &app.icon_list,
        &app.get_selected_station().unwrap_or_default(),
    );

    let footer = status_bar(app.get_status(), &app.music_title);
    rect.render_widget(footer, chunks[2]);
}
/**
Split a Rect into two Rect horizontally (20% - 80%)
 */
fn split_horizontal_chunk(chunk: Rect) -> Vec<Rect> {
    split_chunk(chunk, Direction::Horizontal, 20, 80)
}
/**
Split a Rect into two Rect vertically (20% - 80%)
 */
/*fn split_vertical_chunk(chunk: Rect) -> Vec<Rect> {
    split_chunk(chunk, Direction::Vertical, 20, 80)
}*/
/**
Split a Rect into two Rect in a given direction and percentage of the parts
 */
fn split_chunk(chunk: Rect, dir: Direction, left: u16, right: u16) -> Vec<Rect> {
    Layout::default()
        .direction(dir)
        .constraints([Constraint::Percentage(left), Constraint::Percentage(right)].as_ref())
        .split(chunk)
        .to_vec()
}
/**
Split a Rect into 3 pieces (Default Layout)
 */
fn base_chunk(size: Rect) -> Vec<Rect> {
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
        .to_vec()
}
/**
Paragraph displaying currently playing song
 */
fn status_bar(status: Status, title: &str) -> Paragraph {
    Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::RAPID_BLINK),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Reset))
                .title(status.to_string()),
        )
}
/**
Paragraph displaying information about current station
 */
fn info_bar<'a>(app: &App) -> Paragraph<'a> {
    let station = if app.filtering {
        let mut s = Station::default();
        s.title = "Search".to_string();
        s.tooltip = app.filter.value().to_string();
        s
    } else {
        app.get_selected_station().unwrap_or_default()
    };

    Paragraph::new(station.tooltip.to_string())
        .style(match app.filtering {
            true => Style::default().fg(Color::Yellow),
            false => Style::default(),
        })
        .alignment(match app.filtering {
            true => Alignment::Left,
            false => Alignment::Center,
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(match app.filtering {
                    true => Style::default().fg(Color::Yellow),
                    false => Style::default(),
                })
                .title(station.title.to_string()),
        )
}
/**
Paragraph for the help menu
 */
fn help_paragraph<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            format!("{:50}{:40}", "Description", "Key"),
            Style::default().fg(Color::Red),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Change station list", "<Esc>"
        ))]),
        Line::from(vec![Span::raw(format!("{:50}{:40}", "Go to Help", "h"))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Move up", "<Up Arrow key>"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Move down", "<Down Arrow Key>"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Change station", "<Enter>"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Play/pause station", "<Space>"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Add/remove from favorite", "f"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Enter search mode", "/"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Get current playing song", "n"
        ))]),
        Line::from(vec![Span::raw(format!(
            "{:50}{:40}",
            "Get current playing song on the selected station", "N"
        ))]),
        Line::from(vec![Span::raw(format!("{:50}{:40}", "Quit program", "q"))]),
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
fn make_std_stations_list<'a>(stations_list: &[&Station], menu_item: &MenuItem) -> List<'a> {
    let style = match menu_item {
        MenuItem::Standard(_) => Style::default().fg(ACCENT_COLOR),
        _ => Style::default(),
    };
    make_stations_list(stations_list, "Stations", style)
}
/**
She favorite station list as a List with the correct style to be displayed
 */
fn make_fav_stations_list<'a>(stations_list: &[&Station], menu_item: &MenuItem) -> List<'a> {
    let style = match menu_item {
        MenuItem::Favorite(_) => Style::default().fg(ACCENT_COLOR),
        _ => Style::default(),
    };
    make_stations_list(stations_list, "Favorites", style)
}

/**
Generate the stations list based on the stations names
 */
fn make_stations_list<'a>(stations_list: &[&Station], title: &'a str, style: Style) -> List<'a> {
    let stations = Block::default()
        .borders(Borders::ALL)
        .style(Style::default())
        .title(title)
        .border_type(BorderType::Rounded)
        .border_style(style);

    let items: Vec<_> = stations_list
        .iter()
        .map(|station| {
            ListItem::new(Line::from(vec![Span::styled(
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
Canvas with the stations icon
 */
fn make_icon<B>(
    rect: &mut Frame<B>,
    stations_chunks: &Rect,
    icon_list: &StationsArtList,
    selected_station: &Station,
) where
    B: Backend,
{
    let double = stations_chunks.width / 2 > stations_chunks.height;
    let canvas_size = 200.0;
    let icon_canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .paint(|ctx| {
            let name = selected_station.prefix.to_string();

            match icon_list.get(&name) {
                None => ctx.print(-8.0, 5.0, "no_icon"),
                Some(art) => {
                    let mut shape = Vec::new();
                    let icon = &art.icon;

                    let offset_y = -canvas_size / 2.0 + art.size_y as f64;
                    let offset_x = canvas_size / 2.0 - (art.size_x / 2) as f64;

                    icon.iter().for_each(|(x, y)| {
                        let x = *x as f64 + offset_x;

                        if double {
                            let y = -(*y as f64) + offset_y / 2.0;
                            shape.push((x, y * 2.0));
                            shape.push((x, y * 2.0 + 1.0));
                        } else {
                            let y = -(*y as f64) + offset_y * 2.0;
                            shape.push((x, y));
                        }
                    });

                    ctx.draw(&Points {
                        coords: &shape,
                        color: Color::Reset,
                    });
                }
            };
        })
        .marker(symbols::Marker::Braille)
        .x_bounds([0.0, canvas_size * 0.9])
        .y_bounds([-canvas_size, 0.0]);

    rect.render_widget(icon_canvas, *stations_chunks);
}
