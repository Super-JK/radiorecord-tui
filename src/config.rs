use crate::api::Station;
use crate::config::Error::ReadConfig;
use std::path::PathBuf;
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadFav(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseFav(#[from] serde_json::Error),
    #[error("error parsing the DB file: {0}")]
    ParseIcon(#[from] rmp_serde::decode::Error),
    #[error("error reading config dir")]
    ReadConfig(),
}
/**
Add or delete a favorite from the favorite file
 */
pub fn toggle_to_favorite(station: &Station) -> Result<Vec<Station>, Error> {
    let mut path = get_app_config_path()?;
    path.push("favorite.json");

    let mut parsed: Vec<Station> = read_favorite()?;

    if !parsed.contains(station) {
        parsed.push(station.clone());
        fs::write(path, serde_json::to_vec(&parsed)?)?;
    } else {
        let index = parsed.iter().position(|x| x == station).unwrap();
        parsed.remove(index);
        fs::write(path, serde_json::to_vec(&parsed)?)?;
    }
    Ok(parsed)
}
/**
Read favorite station file or return an empty list
 */
pub fn read_favorite() -> Result<Vec<Station>, Error> {
    let mut path = get_app_config_path()?;
    path.push("favorite.json");

    if !path.exists() {
        Ok(Vec::new())
    } else {
        let content = fs::read_to_string(path)?;
        let parsed: Vec<Station> = serde_json::from_str(&content)?;
        Ok(parsed)
    }
}

pub fn get_app_config_path() -> Result<PathBuf, Error> {
    let mut path = dirs_next::config_dir().ok_or(ReadConfig())?;
    path.push("radiorecord-tui");

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}
