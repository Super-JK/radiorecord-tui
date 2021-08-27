use crate::api;
use curl::easy::Easy;
use std::fs::File;
use std::io::Write;
use asciifyer::{Dimension, convert_to_ascii};

pub fn get_all_icons(){
    let list = api::radio_list().unwrap();

    for station in list {
        let mut easy = Easy::new();
        let mut file = File::create(format!("./img/{}.png",station.prefix)).unwrap();
        easy.url(station.icon_fill_white.as_str()).unwrap();
        easy.write_function(move |data| {
            file.write_all(data).unwrap();
            Ok(data.len())

        }).unwrap();
        easy.perform().unwrap();
    }
}

pub fn to_ascii(){
    let list = api::radio_list().unwrap();
    let mut file = File::create("./data/ascii.json").unwrap();
    file.write_all(b"{").unwrap();

    for station in list {
        let path = format!("./img/{}.png",station.prefix);
        let dimensions = Dimension::new(60, 60);
        let ascii = convert_to_ascii(&path, Some(dimensions));


        file.write_all(format!("\"{}\" : {:?},",station.prefix,ascii).as_bytes()).unwrap();
    };

    file.write_all(b"}").unwrap();
}