use std::fs::File;

use curl::easy::Easy;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(feature = "libmpv_player")]
use libmpv::{FileState, Mpv};

#[cfg(feature = "libmpv_player")]
macro_rules! play {
    ($_playing:ident) => {
        let mut path = std::env::temp_dir();
        path.push(TEMPFILE);

        let mpv = Mpv::new().unwrap();
        mpv.set_property("volume", 85).unwrap();
        mpv.set_property("vo", "null").unwrap();
        mpv.playlist_load_files(&[(&path.to_str().unwrap(), FileState::AppendPlay, None)])
            .unwrap();
    };
}

#[cfg(feature = "rodio_player")]
use {
    rodio::{Decoder, OutputStream, Source},
    std::io::BufReader,
};

#[cfg(feature = "rodio_player")]
macro_rules! play {
    ($playing:ident) => {
        let (_stream, handle) = OutputStream::try_default().expect("no output found");
        let mut path = std::env::temp_dir();
        path.push(TEMPFILE);

        let source = loop {
            match Decoder::new(BufReader::new(File::open(&path).expect("file not found"))) {
                Ok(source) => break source,
                Err(_) => {}
            };
            if !$playing.load(Ordering::Acquire) {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        };

        let _res = handle.play_raw(source.convert_samples());
    };
}

const TEMPFILE: &str = "rrsound";

pub struct Player {
    playing: Arc<AtomicBool>,
    url: String,
    current: Arc<AtomicBool>,
}
/**
Player used to control the station playback
 */
impl Player {
    pub fn new(url:String) -> Self {
        Self {
            playing: Arc::new(AtomicBool::new(false)),
            current: Arc::new(AtomicBool::new(true)),
            url,
        }
    }
    /**
    Fetch and write the stream to a tempfile
    */
    fn fetch(&self, url: String) {
        let playing = self.current.clone();
        thread::spawn(move || {
            let mut easy = Easy::new();
            let mut path = std::env::temp_dir();
            path.push(TEMPFILE);
            let mut file = BufWriter::new(File::create(&path).unwrap());
            easy.url(url.as_str()).unwrap();
            easy.write_function(move |data| {
                file.write_all(data).unwrap();
                Ok(data.len())
            })
            .unwrap();
            easy.progress_function(move |_, _, _, _| playing.load(Ordering::Acquire))
                .unwrap();
            easy.progress(true).unwrap();
            easy.perform()
        });
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
    Stop the player ( reading and writing)
     */
    pub fn stop(&mut self) {
        self.current.store(false, Ordering::Release);
    }

    /**
    Return if the player is playing
     */
    pub fn is_playing(&self) -> bool {
        self.playing.load(Ordering::Acquire)
    }

    /**
    Force play the url even if player is not paused
    **/
    pub fn force_play(&mut self, url: &str) -> bool {
        if self.is_playing() {
            self.stop();
            thread::sleep(Duration::from_millis(210));
            self.play(url)
        } else {
            self.play(url)
        }
    }

    /**
    Read and play the sound from the tempfile
    */
    pub fn play(&mut self, url: &str) -> bool {
        if self
            .playing
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            self.url = url.to_string();
            self.fetch(url.to_string());

            let playing = self.playing.clone();
            let current = self.current.clone();

            thread::spawn(move || {
                thread::sleep(Duration::from_millis(1500));
                play!(playing);

                loop {
                    if !current.load(Ordering::Acquire) {
                        playing.store(false, Ordering::Release);
                        thread::sleep(Duration::from_millis(100));
                        current.store(true, Ordering::Release);
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            });
            return true;
        }
        false
    }
}
