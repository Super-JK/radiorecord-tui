use crossbeam::channel::Sender;
use std::collections::HashMap;
use zbus::zvariant::Value;
use zbus::{dbus_interface, ConnectionBuilder};

pub enum Command {
    PlayPause,
    Play,
    Pause,
    Next,
    Previous,
}

pub struct MediaPlayerInterface {
    pub tx: Sender<Command>,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
#[allow(non_snake_case)]
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
    fn Metadata(&self) -> HashMap<&str, Value> {
        let mut map = HashMap::new();
        map.insert("mpris:trackid", Value::from("f"));
        map.insert("xesam:title", Value::from("Salut"));
        map
    }

    // Can be `async` as well.
    fn Next(&mut self) {
        self.tx.send(Command::Next).expect("Could not send")
    }
    fn Previous(&mut self) {
        self.tx.send(Command::Previous).expect("Could not send")
    }

    fn Play(&mut self) {
        self.tx.send(Command::Play).expect("Could not send")
    }
    fn Pause(&mut self) {
        self.tx.send(Command::Pause).expect("Could not send");
    }
    fn PlayPause(&mut self) {
        self.tx.send(Command::PlayPause).expect("Could not send");
    }
}

pub async fn launch_mpris_server(tx: Sender<Command>) -> Result<(), Box<dyn std::error::Error>> {
    let player = MediaPlayerInterface { tx };
    let _ = ConnectionBuilder::session()?
        .name("org.mpris.MediaPlayer2.rrt")?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build()
        .await?;
    Ok(())
}
