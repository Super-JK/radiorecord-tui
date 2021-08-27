use std::fs::File;
use std::io::{BufReader};
use rodio::{Decoder, OutputStream, Source};

use curl::easy::{Easy, WriteError};
use std::io::{Write};
use std::thread;
use std::time::Duration;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};

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
            let mut file = File::create("/tmp/rrsound").unwrap();
            easy.url(url.as_str()).unwrap();
            easy.write_function(move |data| {
                file.write_all(data).unwrap();
                if playing.load(Ordering::Acquire) {
                    Ok(data.len())
                } else {
                    Err(WriteError::Pause)
                }
            }).unwrap();
            easy.perform().unwrap();

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
                std::thread::sleep(Duration::from_millis(200));

                let (_stream, handle) = OutputStream::try_default().expect("no output found");


                let source = loop {
                    match Decoder::new(BufReader::new(File::open("/tmp/rrsound").expect("file not found"))) {
                        Ok(source) => break source,
                        Err(_) => {},
                    };
                    if !playing.load(Ordering::Acquire) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(500));
                };


                let res = handle.play_raw(source.convert_samples());
                loop {
                    if let Err(_) = res {
                        println!("play error");
                        break;
                    }
                    if !playing.load(Ordering::Acquire) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
                /*
                let sink = Sink::try_new(&handle).unwrap();
                sink.append(source);
                sink.sleep_until_end();
                self.sink=Some(sink);
                 */
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


