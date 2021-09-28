use curl::easy::Easy;
use serde_json::Value;
use serde::{Deserialize, Serialize};
/**
Represent a song (title and artist)
*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Title {
    pub song: String,
    pub artist: String,
}

impl Title{
    pub fn to_string(&self)->String{
        format!("{} - {}",self.song,self.artist)
    }
}
/**
Represent a station with useful info
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Station {
    pub id: u16,
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
    genre:Value,
    stations:Vec<Station>,
}

#[derive(Serialize, Deserialize)]
struct Res{
    result : Part,
}

#[derive(Serialize, Deserialize)]
struct PartHistory {
    history:Vec<Title>,
}

#[derive(Serialize, Deserialize)]
struct ResHistory{
    result : PartHistory,
}
#[derive(Debug)]
pub enum ApiError {
    NoConnection,
}

/**
Fetch the list of stations and some information about them
 */
pub fn radio_list()->Result<Vec<Station>,ApiError> {
    let data= read("https://www.radiorecord.ru/api/stations/")?;

    let str_ = std::str::from_utf8(&data).unwrap();
    let json:Res = serde_json::from_str(str_).unwrap();

    Ok(json.result.stations)

}
/**
Fetch a list the song and artist in the history of a station
*/
pub fn now_playing(id:u16)->Result<Title,ApiError>{
    let data = read(format!("https://www.radiorecord.ru/api/station/history/?id={}",id).as_str())?;

    let str_ = std::str::from_utf8(&data).unwrap();
    let json:ResHistory = serde_json::from_str(str_).unwrap();

    Ok(json.result.history[0].clone())
}


fn read(url:&str) -> Result<Vec<u8>,ApiError>{
    let mut data= Vec::new();
    let mut handle = Easy::new();
    handle.url(url).unwrap();
    let res;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        res = transfer.perform();

    }
    match res{
        Ok(_)=>Ok(data),
        Err(_)=>Err(ApiError::NoConnection)
    }
}
