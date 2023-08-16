use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;
use zbus::zvariant::Value;
use zbus::{dbus_interface, Connection, ConnectionBuilder};

use crate::app::Event;

pub enum Command {
    PlayPause,
    Play,
    Stop,
    Next,
    Previous,
    NowPlaying,
    Status,
}

pub enum Response {
    NowPlaying(String),
    Status(String),
}

pub struct MediaPlayerInterface {
    pub tx: Sender<Event>,
    pub rx: Receiver<Response>,
}

#[cfg(debug_assertions)]
const INAME: &str = "org.mpris.MediaPlayer2.rrt_test";
#[cfg(not(debug_assertions))]
const INAME: &str = "org.mpris.MediaPlayer2.rrt";

#[allow(non_snake_case)]
#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl MediaPlayerInterface {
    #[dbus_interface(property, name = "CanControl")]
    fn CanControl(&self) -> bool {
        true
    }
    #[dbus_interface(property, name = "CanPlay")]
    fn CanPlay(&self) -> bool {
        true
    }
    #[dbus_interface(property, name = "CanStop")]
    fn CanStop(&self) -> bool {
        true
    }
    #[dbus_interface(property, name = "CanGoNext")]
    fn CanGoNext(&self) -> bool {
        true
    }
    #[dbus_interface(property, name = "CanGoPrevious")]
    fn CanGoPrevious(&self) -> bool {
        true
    }
    #[dbus_interface(property, name = "Metadata")]
    async fn Metadata(&self) -> HashMap<&str, Value> {
        self.tx.send(Event::Mpris(Command::NowPlaying)).expect("Could not send");
        let mut map = HashMap::new();
        if let Response::NowPlaying(title) = self.rx.recv().unwrap() {
            map.insert("xesam:title", Value::from(title));
            return map;
        }
        map
    }
    #[dbus_interface(property, name = "PlaybackStatus")]
    async fn PlaybackStatus(&self) -> String {
        self.tx.send(Event::Mpris(Command::Status)).expect("Could not send");
        if let Ok(Response::Status(status)) = self.rx.recv() {
            status
        } else {
            "Unknown".to_string()
        }
    }

    // Can be `async` as well.
    async fn Next(&mut self) {
        self.tx.send(Event::Mpris(Command::Next)).expect("Could not send")
    }
    async fn Previous(&mut self) {
        self.tx.send(Event::Mpris(Command::Previous)).expect("Could not send")
    }

    async fn Play(&mut self) {
        self.tx.send(Event::Mpris(Command::Play)).expect("Could not send")
    }
    async fn Stop(&mut self) {
        self.tx.send(Event::Mpris(Command::Stop)).expect("Could not send");
    }
    async fn PlayPause(&mut self) {
        self.tx.send(Event::Mpris(Command::PlayPause)).expect("Could not send");
    }
}

pub async fn launch_mpris_server(
    tx: Sender<Event>,
    rx: Receiver<Response>,
) -> color_eyre::Result<Connection> {
    let player = MediaPlayerInterface { tx, rx };
    let conn = ConnectionBuilder::session()?
        .name(INAME)?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build().await?;

    Ok(conn)
}
