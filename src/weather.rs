extern crate ftp;
extern crate reqwest;
extern crate serde_json;

pub struct Client;

#[derive(Debug, Deserialize)]
struct ObservationsRaw {
    observations: Observations,
}

#[derive(Debug, Deserialize)]
struct Observations {
    data: Vec<Observation>,
}

#[derive(Debug, Deserialize)]
pub enum WindDirection {
    CALM,
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    NNE,
    ENE,
    ESE,
    SSE,
    SSW,
    WSW,
    WNW,
    NNW,
}

#[derive(Debug, Deserialize)]
pub struct Observation {
    pub sort_order: u32,
    pub name: String,
    pub history_product: String,
    pub local_date_time: String,      //"11/01:30pm",
    pub local_date_time_full: String, // "20180811133000",
    pub aifstime_utc: String,         // "20180811033000",
    pub lat: f64,
    pub lon: f64,
    pub apparent_t: f64,
    pub delta_t: f64,
    pub gust_kmh: u32,
    pub gust_kt: u32,
    pub air_temp: f64,
    pub dewpt: f64,
    pub press: f64,
    pub press_qnh: f64,
    pub press_msl: f64,
    pub press_tend: String,
    pub rain_trace: String, // Rain since 9am, not sure why this is a string
    pub rel_hum: u32,
    pub wind_dir: WindDirection,
    pub wind_spd_kmh: u32,
    pub wind_spd_kt: u32,
}

pub struct Forecast;

#[derive(Fail, Debug)]
pub enum WeatherError {
    // #[fail(display = "I/O error")]
    // IoError(io::Error),
    #[fail(display = "HTTP error")] HttpError(reqwest::Error),
    // #[fail(display = "UTF-8 parse error")]
    // ParseError(str::Utf8Error),
    #[fail(display = "JSON error")] JsonError(serde_json::Error),
}

type WeatherResult<T> = Result<T, WeatherError>;

impl From<serde_json::Error> for WeatherError {
    fn from(err: serde_json::Error) -> Self {
        WeatherError::JsonError(err)
    }
}

impl From<reqwest::Error> for WeatherError {
    fn from(err: reqwest::Error) -> Self {
        WeatherError::HttpError(err)
    }
}

//ftp://ftp.bom.gov.au/anon/gen/fwo/IDV10450.xml
//http://reg.bom.gov.au/fwo/IDV60901/IDV60901.95936.json

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub fn forecast(&self) -> WeatherResult<Forecast> {
        unimplemented!()
    }

    pub fn observations(&self) -> WeatherResult<Vec<Observation>> {
        let obs: ObservationsRaw =
            reqwest::get("http://reg.bom.gov.au/fwo/IDV60901/IDV60901.95936.json")?.json()?;

        Ok(obs.observations.data)
    }
}
