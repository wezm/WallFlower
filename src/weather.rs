extern crate ftp;
extern crate reqwest;

struct Client;

struct Observations {
    data: Vec<Observation>
};

struct Observation {
//     "sort_order": 0,
//     "wmo": 95936,
//     "name": "Melbourne (Olympic Park)",
//     "history_product": "IDV60901",
//     "local_date_time": "11/01:30pm",
//     "local_date_time_full": "20180811133000",
//     "aifstime_utc": "20180811033000",
//     "lat": -37.8,
//     "lon": 145.0,
//     "apparent_t": 11.7,
//     "cloud": "-",
//     "cloud_base_m": null,
//     "cloud_oktas": null,
//     "cloud_type_id": null,
//     "cloud_type": "-",
//     "delta_t": 3.1,
//     "gust_kmh": 15,
//     "gust_kt": 8,
//     "air_temp": 13.5,
//     "dewpt": 7.1,
//     "press": 1006.1,
//     "press_qnh": 1006.1,
//     "press_msl": 1006.1,
//     "press_tend": "-",
//     "rain_trace": "1.0",
//     "rel_hum": 65,
//     "sea_state": "-",
//     "swell_dir_worded": "-",
//     "swell_height": null,
//     "swell_period": null,
//     "vis_km": "10",
//     "weather": "-",
//     "wind_dir": "WSW",
//     "wind_spd_kmh": 6,
//     "wind_spd_kt": 3
}

struct Forecast;

#[derive(Fail, Debug)]
pub enum WeatherError {
    #[fail(display = "I/O error")]
    IoError(io::Error),
    #[fail(display = "HTTP error")]
    HttpError(reqwest::Error),
    #[fail(display = "UTF-8 parse error")]
    ParseError(str::Utf8Error),
    #[fail(display = "JSON error")]
    JsonError(serde_json::Error),
}

type WeatherResult = Result<T, WeatherError>;

// impl From<str::Utf8Error> for WeatherError {
//     fn from(err: str::Utf8Error) -> Self {
//         WeatherError::ParseError(err)
//     }
// }


//ftp://ftp.bom.gov.au/anon/gen/fwo/IDV10450.xml
//http://reg.bom.gov.au/fwo/IDV60901/IDV60901.95936.json

impl Client {
    pub fn forecast(&self) -> WeatherResult<Forecast> {
        unimplemented!()
    }

    pub fn observations(&self) -> WeatherResult<Observations> {
        let obs: ObservationsRaw = reqwest::get("http://reg.bom.gov.au/fwo/IDV60901/IDV60901.95936.json").send()?.json()?;
    }
}


#[test]
fn test_deesire() {
    let bom = Client::new();

    let obs = bom.observations();
    let forecast = bom.forecast();
}
