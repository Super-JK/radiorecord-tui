use std::fs::File;

use curl::easy::{Easy};
use std::io::{BufWriter, Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "libmpv_player")]
use libmpv::{Mpv, FileState};

#[cfg(feature = "libmpv_player")]
macro_rules! play {
    ($_playing:ident) => {
        let mut path = std::env::temp_dir();
        path.push(TEMPFILE);

        let mpv = Mpv::new().unwrap();
        mpv.set_property("volume", 85).unwrap();
        mpv.set_property("vo", "null").unwrap();
        mpv.playlist_load_files(&[(&path.to_str().unwrap(), FileState::AppendPlay, None)]).unwrap();
    }
}

#[cfg(feature = "rodio_player")]
use {
    std::io::{BufReader},
    rodio::{Decoder, OutputStream, Source}
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
                Err(_) => {},
            };
            if !$playing.load(Ordering::Acquire) {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        };

        let _res = handle.play_raw(source.convert_samples());
    }
}

const TEMPFILE: &str = "rrsound";

pub struct Player {
    playing:Arc<AtomicBool>,
}
/**
Player used to control the station playback
 */
impl Player{
    pub fn new() -> Player {
        Player{
            playing: Arc::new(AtomicBool::new(false)),
        }
    }
    /**
    Fetch and write the stream to a tempfile
    */
    fn fetch(&self, url:String){
        let playing = self.playing.clone();
        thread::spawn(move || {
            let mut easy = Easy::new();
            let mut path = std::env::temp_dir();
            path.push(TEMPFILE);
            let mut file = BufWriter::new(File::create(&path).unwrap());
            easy.url(url.as_str()).unwrap();
            easy.write_function(move |data| {
                file.write(data).unwrap();
                Ok(data.len())
            }).unwrap();
            easy.progress_function(move |_, _, _, _| {
                playing.load(Ordering::Acquire)
            }).unwrap();
            easy.progress(true).unwrap();
            match easy.perform() {
                _ => {}
            }
        });
    }
    /**
    Read and play the sound from the tempfile
    */
    pub fn  play(&mut self, url:String){
        if !self.playing.load(Ordering::Acquire) {
            self.playing.store(true, Ordering::Release);

            self.fetch(url);

            let playing = self.playing.clone();

            thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(1500));
                play!(playing);

                loop {
                    if !playing.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            });
        }
    }
    /**
    Stop the player ( reading and writing)
     */
    pub fn stop(&mut self){
        self.playing.store(false,Ordering::Release)
    }
    /**
    Return if the player is playing
    */
    pub fn is_playing(&self) -> bool{
        self.playing.load(Ordering::Acquire)
    }
}


