use crate::api;
use crate::config::{get_app_config_path, Error};
use curl::easy::Easy;
use image::imageops::FilterType;
use image::GenericImageView;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::process::exit;
use std::{fs, io};

const TEMPDIR: &str = "rricons/";
const ICONFILE: &str = "art.msgpack";

/**
Download all stations icons to a file int the tmp folder
 */
pub fn get_all_icons() {
    //get all stations
    let list = api::stations_list().unwrap();

    let mut path = std::env::temp_dir();
    path.push(TEMPDIR);

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }

    //fetch the icons for each station and write it in a tmp folder
    for station in list {
        let path = format!("{}/{}.png", path.display(), station.prefix);
        if File::open(&path).is_ok() {
            continue;
        }
        let mut easy = Easy::new();
        let mut file = BufWriter::new(File::create(path).unwrap());
        easy.url(station.icon_fill_white.as_str()).unwrap();
        easy.write_function(move |data| {
            file.write_all(data).unwrap();
            Ok(data.len())
        })
        .unwrap();
        easy.perform().unwrap();
    }
}

pub type StationsArtList = HashMap<String, StationArt>;
#[derive(Debug, Serialize, Deserialize)]
pub struct StationArt {
    pub icon: Vec<(u32, u32)>,
    pub size_x: u32,
    pub size_y: u32,
}
/**
Convert the icons previously downloaded to ascii art and write it to a file
 */
pub fn save_station_art() {
    //get stations list
    let list = api::stations_list().unwrap();

    // Convert image to dot art
    let mut art_list = HashMap::new();
    for station in list.iter() {
        //path of the icon
        let mut path = std::env::temp_dir();
        path.push(format!("{}{}.png", TEMPDIR, station.prefix));

        //convert icon in different ascii size
        let size = 128;

        let mut img = image::open(path).expect("Can't find image file at");
        let (_, mut height) = img.dimensions();

        img = img.resize(size, height, FilterType::Lanczos3);
        let d = img.dimensions();
        let width = d.0;
        height = d.1;

        let mut ascii = Vec::new();
        let mut min_x = size;
        let mut min_y = size;
        let mut max_x = 0;
        let mut max_y = 0;
        for y in 0..height {
            for x in 0..width {
                let p = img.get_pixel(x, y);
                if ((p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0) > 128.0 {
                    ascii.push((x, y));
                    if x < min_x {
                        min_x = x
                    }
                    if x > max_x {
                        max_x = x
                    }
                    if y < min_y {
                        min_y = y
                    }
                    max_y = y
                }
            }
        }
        ascii = ascii.iter().map(|v| (v.0 - min_x, v.1 - min_y)).collect();

        //write to file in message pack format
        art_list.insert(
            station.prefix.to_string(),
            StationArt {
                icon: ascii,
                size_x: max_x,
                size_y: max_y - min_y,
            },
        );
    }
    let mut buf = Vec::new();
    art_list.serialize(&mut Serializer::new(&mut buf)).unwrap();

    //create and write save file
    let mut path = get_app_config_path().unwrap();
    path.push(ICONFILE);
    let mut file = BufWriter::new(File::create(path).unwrap());
    file.write_all(&buf).unwrap();
}

/**
Read the file containing ascii art for icons. If it doesnt exist generate it
 */
pub fn read_icons() -> Result<StationsArtList, Error> {
    let mut path = get_app_config_path()?;
    path.push(ICONFILE);

    if !path.exists() {
        println!("Downloading icons...");
        get_all_icons();
        println!("Converting icons...");
        save_station_art();
    }

    let content = fs::read(path)?;
    let parsed = rmp_serde::from_slice(&content)?;
    Ok(parsed)
}
/**
Wait for the user to press enter
 **/
pub fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press enter to exit...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
    exit(0);
}
