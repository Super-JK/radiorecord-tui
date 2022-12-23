use curl::easy::Easy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
/**
Represent a song (title and artist)
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Title {
    pub song: String,
    pub artist: String,
}

impl Display for Title {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.song, self.artist)
    }
}
/**
Represent a station with useful info
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Station {
    pub id: usize,
    pub title: String,
    pub prefix: String,
    pub tooltip: String,
    short_title: String,
    pub icon_fill_white: String,
    pub stream_320: String,
}
impl PartialEq for Station {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Station {}

#[derive(Serialize, Deserialize)]
struct Part {
    genre: Value,
    stations: Vec<Station>,
}

#[derive(Serialize, Deserialize)]
struct Res {
    result: Part,
}

#[derive(Serialize, Deserialize)]
struct PartHistory {
    history: Vec<Title>,
}

#[derive(Serialize, Deserialize)]
struct ResHistory {
    result: PartHistory,
}

#[derive(Serialize, Deserialize)]
struct PartNowPlaying {
    id: usize,
    track: Title,
}

#[derive(Serialize, Deserialize)]
struct ResNowPlaying {
    result: Vec<PartNowPlaying>,
}
#[derive(Debug)]
pub enum ApiError {
    NoConnection,
    ServerError,
}

/**
Fetch the list of stations and some information about them
 */
pub fn stations_list() -> Result<Vec<Station>, ApiError> {
    let data = read("https://www.radiorecord.ru/api/stations/")?;

    let str_ = std::str::from_utf8(&data).unwrap();
    let json: Res = serde_json::from_str(str_).unwrap();

    Ok(json.result.stations)
}
/**
Fetch a list of the song and artist in the history of a station
*/
pub fn history(id: usize) -> Result<Vec<Title>, ApiError> {
    let data = read(format!("https://www.radiorecord.ru/api/station/history/?id={}", id).as_str())?;

    let str_ = std::str::from_utf8(&data).unwrap();
    let json: ResHistory = match serde_json::from_str::<ResHistory>(str_) {
        Ok(res) => res,
        Err(_) => return Err(ApiError::ServerError),
    };

    Ok(json.result.history)
}

/**
Fetch the current playing song
*/
pub fn now_playing(id: usize) -> Result<Title, ApiError> {
    match history(id) {
        Ok(mut vec) => Ok(vec.remove(0)),
        Err(ApiError::ServerError) => now_playing_back(id),
        Err(error) => Err(error),
    }
}
/**
Fetch the current playing song from a different endpoints if th other fail
*/
fn now_playing_back(id: usize) -> Result<Title, ApiError> {
    let data = read("https://www.radiorecord.ru/api/stations/now/")?;

    let str_ = std::str::from_utf8(&data).unwrap();
    let json: ResNowPlaying = serde_json::from_str(str_).unwrap();

    let station = json.result.into_iter().nth(id).unwrap();

    Ok(station.track)
}

/**
Read from url
**/
fn read(url: &str) -> Result<Vec<u8>, ApiError> {
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(url).unwrap();
    let res;
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        res = transfer.perform();
    }
    match res {
        Ok(_) => Ok(data),
        Err(_) => Err(ApiError::NoConnection),
    }
}
