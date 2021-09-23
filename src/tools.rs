use crate::api;
use curl::easy::Easy;
use std::fs::File;
use std::io::Write;
use asciifyer::{Dimension, convert_to_ascii};
use std::path::PathBuf;
use std::fs;
use crate::config::get_app_config_path;

const TEMPICONPATH: &str = "/tmp/rricons/";

pub fn get_all_icons(){
    let list = api::radio_list().unwrap();

    let path = PathBuf::from(TEMPICONPATH);

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    for station in list {
        let mut easy = Easy::new();
        let mut file = File::create(format!("{}/{}.png",TEMPICONPATH,station.prefix)).unwrap();
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
    let mut path = get_app_config_path().unwrap();
    path.push("ascii.json");
    let mut file = File::create(path).unwrap();
    file.write_all(b"{").unwrap();

    for (i, station) in list.iter().enumerate() {
        let path = format!("{}/{}.png",TEMPICONPATH,station.prefix);
        let sizes = Vec::from([60u32,30]);
        for (j, size) in sizes.iter().enumerate() {
            let dimensions = Dimension::new(*size, *size);
            let mut ascii = convert_to_ascii(&path, Some(dimensions));
            let opti = &format!("{}\n"," ".repeat(*size as usize)) ;
            while ascii.find(opti) != None {
                ascii = ascii.replace(opti,"");
            };

            let mut form = format!("\"{}_{}\" : {:?},",station.prefix,size,ascii);
            if i == list.len()-1 && j == sizes.len()-1  {form = format!("\"{}_{}\" : {:?}",station.prefix,size,ascii) }

            file.write_all(form.as_bytes()).unwrap()
        }

    };

    file.write_all(b"}").unwrap();
}