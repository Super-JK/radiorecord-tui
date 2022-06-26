use crate::api;
use crate::config::get_app_config_path;
use asciifyer::{convert_to_ascii, Dimension};
use curl::easy::Easy;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};

const TEMPDIR: &str = "rricons/";

/**
Download all stations icons to a file int the tmp folder
 */
pub fn get_all_icons() {
    //get all stations
    let list = api::radio_list().unwrap();

    let mut path = std::env::temp_dir();
    path.push(TEMPDIR);

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    //fetch the icons for each station and write it in a tmp folder
    for station in list {
        let mut easy = Easy::new();
        let mut file = BufWriter::new(
            File::create(format!("{}/{}.png", path.display(), station.prefix)).unwrap(),
        );
        easy.url(station.icon_fill_white.as_str()).unwrap();
        easy.write_function(move |data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
        easy.perform().unwrap();
    }
}

/**
Convert the icons previously downloaded to ascii art and write it to a file
 */
pub fn to_ascii() {
    //get stations list
    let list = api::radio_list().unwrap();
    //create save file
    let mut path = get_app_config_path().unwrap();
    path.push("ascii.json");
    let mut file = BufWriter::new(File::create(path).unwrap());
    file.write_all(b"{").unwrap();

    for (i, station) in list.iter().enumerate() {
        //path of the icon
        let mut path = std::env::temp_dir();
        path.push(format!("{}{}.png", TEMPDIR, station.prefix));

        //convert icon in different ascii size
        let sizes = Vec::from([60u32, 30]);
        for (j, size) in sizes.iter().enumerate() {
            let dimensions = Dimension::new(*size, *size);
            let mut ascii = convert_to_ascii(&path, Some(dimensions));
            let opti = &format!("{}\n", " ".repeat(*size as usize));
            while ascii.find(opti) != None {
                ascii = ascii.replace(opti, "");
            }
            //write to file  in json format
            let mut form = format!("\"{}_{}\" : {:?},", station.prefix, size, ascii);
            if i == list.len() - 1 && j == sizes.len() - 1 {
                form = format!("\"{}_{}\" : {:?}", station.prefix, size, ascii)
            }

            file.write_all(form.as_bytes()).unwrap();
        }
    }

    file.write_all(b"}").unwrap();
}
