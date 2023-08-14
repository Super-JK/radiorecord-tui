use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[cfg(feature = "libmpv_player")]
use libmpv::{FileState, Mpv};

#[cfg(feature = "rodio_player")]
use {
    curl::easy::Easy,
    rodio::{Decoder, OutputStream, Source},
    std::fs::File,
    std::io::{BufReader, BufWriter, Write},
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[cfg(feature = "rodio_player")]
const TEMPFILE: &str = "rrsound";

enum PlayerCommand {
    Play(String),
    Stop,
    NowPlaying,
}

enum PlayerResponse {
    NowPlaying(String),
}

pub struct Player {
    playing: bool,
    url: String,
    sender: Sender<PlayerCommand>,
    receiver: Receiver<PlayerResponse>,
}

/**
Player used to control the station playback
 */
impl Player {
    pub fn new(url: String) -> Self {
        let (sender_player, receiver_player) = channel::bounded(1);
        let (sender_interface, receiver_interface) = channel::bounded(1);

        #[cfg(feature = "libmpv_player")]
        thread::spawn(move || {
            let mpv = Mpv::new().unwrap();
            mpv.set_property("volume", 85).unwrap();
            mpv.set_property("vo", "null").unwrap();

            loop {
                if receiver_player.is_empty() {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
                match receiver_player.recv().unwrap() {
                    PlayerCommand::Play(url) => {
                        mpv.playlist_load_files(&[(&url, FileState::Replace, None)])
                            .unwrap();
                        mpv.unpause().unwrap();
                    }
                    PlayerCommand::Stop => {
                        mpv.playlist_clear().unwrap();
                        mpv.pause().unwrap()
                    }
                    PlayerCommand::NowPlaying => {
                        let mut title = "Loading...".to_string();
                        if let Ok(title_) = mpv.get_property::<String>("media-title") {
                            title = title_;
                        }
                        sender_interface
                            .send(PlayerResponse::NowPlaying(title))
                            .unwrap();
                    }
                };
            }
        });

        #[cfg(feature = "rodio_player")]
        thread::spawn(move || {
            let mut path = std::env::temp_dir();
            path.push(TEMPFILE);
            let playing = Arc::new(AtomicBool::new(false));

            loop {
                if receiver_player.is_empty() {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
                match receiver_player.recv().unwrap() {
                    PlayerCommand::Play(url) => {
                        // write to tempfile
                        let mut file = BufWriter::new(File::create(&path).unwrap());
                        let mut easy = Easy::new();
                        easy.write_function(move |data| {
                            file.write_all(data).unwrap();
                            Ok(data.len())
                        })
                        .unwrap();

                        let playing_ = playing.clone();
                        easy.progress_function(move |_, _, _, _| playing_.load(Ordering::Acquire))
                            .unwrap();
                        easy.progress(true).unwrap();
                        easy.url(url.as_str()).unwrap();
                        playing.store(true, Ordering::Release);

                        thread::spawn(move || easy.perform());

                        // read from tempfile
                        let source = loop {
                            if let Ok(source) = Decoder::new(BufReader::new(
                                File::open(&path).expect("file not found"),
                            )) {
                                break source;
                            };
                        };

                        let playing_ = playing.clone();
                        thread::spawn(move || {
                            let (_stream, handle) =
                                OutputStream::try_default().expect("no output found");
                            let _res = handle.play_raw(source.convert_samples());

                            loop {
                                if !playing_.load(Ordering::Acquire) {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(200));
                            }
                        });
                    }
                    PlayerCommand::Stop => {
                        playing.store(false, Ordering::Relaxed);
                    }
                    PlayerCommand::NowPlaying => {
                        sender_interface
                            .send(PlayerResponse::NowPlaying("Not implemented".to_string()))
                            .unwrap();
                    }
                }
            }
        });

        Self {
            playing: false,
            url,
            sender: sender_player,
            receiver: receiver_interface,
        }
    }

    pub fn resume(&mut self) {
        if !self.is_playing() {
            self.play(&self.url.clone());
        }
    }

    pub fn toggle_play(&mut self) {
        if self.is_playing() {
            self.stop();
        } else {
            self.play(&self.url.clone());
        }
    }

    /**
    Stop the player
     */
    pub fn stop(&mut self) {
        self.sender.send(PlayerCommand::Stop).unwrap();
        self.playing = false;
    }

    /**
    Player is playing
     */
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /**
    Force play the url even if player is not paused
    **/
    pub fn force_play(&mut self, url: &str) -> bool {
        if self.is_playing() {
            self.stop();
            self.play(url)
        } else {
            self.play(url)
        }
    }

    /**
    Play the station from the specified url
     */
    pub fn play(&mut self, url: &str) -> bool {
        if !self.playing {
            self.url = url.to_string();
            self.sender
                .send(PlayerCommand::Play(url.to_string()))
                .unwrap();
            self.playing = true;

            return true;
        }
        false
    }

    /// The current playing title (author and title name)
    pub fn now_playing(&self) -> Option<String> {
        self.sender.send(PlayerCommand::NowPlaying).unwrap();
        let res = self.receiver.recv().unwrap();

        if let PlayerResponse::NowPlaying(title) = res {
            return Some(title);
        }

        None
    }
}
