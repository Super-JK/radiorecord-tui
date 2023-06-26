use std::thread;
use std::time::Duration;
use crossbeam::channel;
use crossbeam::channel::{Sender};

#[cfg(feature = "libmpv_player")]
use libmpv::{FileState, Mpv};

#[cfg(feature = "rodio_player")]
use {
    rodio::{Decoder, OutputStream, Source},
    std::io::{BufReader, BufWriter, Write},
    curl::easy::Easy,
    std::fs::File,
    std::sync::{Arc,atomic::{AtomicBool, Ordering}},
};


#[cfg(feature = "rodio_player")]
const TEMPFILE: &str = "rrsound";

enum PlayerCommand {
    Play(String),
    Stop,
}

pub struct Player {
    playing: bool,
    url: String,
    sender: Sender<PlayerCommand>,
}

/**
Player used to control the station playback
 */
impl Player {
    pub fn new(url: String) -> Self {
        let (sender, receiver) = channel::bounded(1);

        #[cfg(feature = "libmpv_player")]
        thread::spawn(move || {
            let mpv = Mpv::new().unwrap();
            mpv.set_property("volume", 85).unwrap();
            mpv.set_property("vo", "null").unwrap();

            loop {
                if receiver.is_empty() {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
                match receiver.recv().unwrap() {
                    PlayerCommand::Play(url) => {
                        mpv.playlist_load_files(&[(&url, FileState::Replace, None)]).unwrap();
                        mpv.unpause().unwrap();
                    }
                    PlayerCommand::Stop => {
                        mpv.playlist_clear().unwrap();
                        mpv.pause().unwrap()
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
                if receiver.is_empty() {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
                match receiver.recv().unwrap() {
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
                        easy.progress_function(move |_, _, _, _| {  playing_.load(Ordering::Acquire)})
                            .unwrap();
                        easy.progress(true).unwrap();
                        easy.url(url.as_str()).unwrap();
                        playing.store(true,Ordering::Release);

                        thread::spawn(move || {
                            easy.perform()
                        });

                        // read from tempfile
                        let source = loop {
                            match Decoder::new(BufReader::new(File::open(&path).expect("file not found"))) {
                                Ok(source) => break source,
                                Err(_) => {}
                            };
                        };

                        let playing_ = playing.clone();
                        thread::spawn(move || {

                            let (_stream, handle) = OutputStream::try_default().expect("no output found");
                            let _res = handle.play_raw(source.convert_samples());

                            loop {
                                if !playing_.load(Ordering::Acquire) {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(200));
                            }
                        });
                    },
                    PlayerCommand::Stop => { playing.store(false,Ordering::Relaxed);

                    },
                }
            }
        });

        Self {
            playing: false,
            url,
            sender,
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
        if !self.playing
        {
            self.url = url.to_string();
            self.sender.send(PlayerCommand::Play(url.to_string())).unwrap();
            self.playing = true;

            return true;
        }
        false
    }
}
