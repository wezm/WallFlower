use weather::Observation;

pub fn latest_observation(observations: Vec<Observation>) -> Option<Observation> {
    observations.into_iter().nth(0)
}

pub fn format_observation(observation: &Option<Observation>) -> String {
    if let Some(o) = observation {
        format!(
            "{}°C feels like {}°C   Rain since 9am: {}mm   {}% humidity",
            o.air_temp, o.apparent_t, o.rain_trace, o.rel_hum
        )
    } else {
        let default = "--";
        format!(
            "{}°C feels like {}°C   Rain since 9am: {}mm   {}% humidity",
            default, default, default, default
        )
    }
}

