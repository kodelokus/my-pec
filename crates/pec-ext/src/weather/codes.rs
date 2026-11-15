//! WMO weather codes as the weather page's conditions, and temperatures in the person's units (weather R3).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    Snow,
    Thunderstorm,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Units {
    #[default]
    Celsius,
    Fahrenheit,
}

impl WeatherCondition {
    /// The condition for a WMO code; none for a code the table doesn't know.
    pub fn of(code: u32) -> Option<Self> {
        use WeatherCondition::*;
        Some(match code {
            0 | 1 => Clear,
            2 => PartlyCloudy,
            3 => Cloudy,
            45 | 48 => Fog,
            51 | 53 | 55 | 56 | 57 => Drizzle,
            61 | 63 | 66 | 80 | 81 => Rain,
            65 | 67 | 82 => HeavyRain,
            71 | 73 | 75 | 77 | 85 | 86 => Snow,
            95 | 96 | 99 => Thunderstorm,
            _ => return None,
        })
    }

    /// The words the companion is told (weather data 5).
    pub fn words(self) -> &'static str {
        use WeatherCondition::*;
        match self {
            Clear => "clear",
            PartlyCloudy => "partly cloudy",
            Cloudy => "cloudy",
            Fog => "fog",
            Drizzle => "drizzle",
            Rain => "rain",
            HeavyRain => "heavy rain",
            Snow => "snow",
            Thunderstorm => "thunderstorm",
        }
    }
}

impl Units {
    /// A Celsius reading in these units, rounded to a whole degree.
    pub fn of(self, celsius: f64) -> i32 {
        let value = match self {
            Units::Celsius => celsius,
            Units::Fahrenheit => celsius * 9.0 / 5.0 + 32.0,
        };
        value.round() as i32
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Units::Celsius => "°C",
            Units::Fahrenheit => "°F",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_code_in_the_table_has_its_condition_and_others_have_none() {
        use WeatherCondition::*;
        for (codes, condition) in [
            (&[0, 1][..], Clear),
            (&[2], PartlyCloudy),
            (&[3], Cloudy),
            (&[45, 48], Fog),
            (&[51, 53, 55, 56, 57], Drizzle),
            (&[61, 63, 66, 80, 81], Rain),
            (&[65, 67, 82], HeavyRain),
            (&[71, 73, 75, 77, 85, 86], Snow),
            (&[95, 96, 99], Thunderstorm),
        ] {
            for code in codes {
                assert_eq!(WeatherCondition::of(*code), Some(condition), "{code}");
            }
        }
        for unknown in [4, 50, 60, 100] {
            assert_eq!(WeatherCondition::of(unknown), None, "{unknown}");
        }
    }

    #[test]
    fn temperatures_round_in_either_unit() {
        assert_eq!(Units::Celsius.of(26.4), 26);
        assert_eq!(Units::Celsius.of(-0.6), -1);
        assert_eq!(Units::Fahrenheit.of(26.3), 79);
        assert_eq!(Units::Fahrenheit.of(-40.0), -40);
    }
}
