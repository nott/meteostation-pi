use dht22_pi;

#[derive(Debug, Clone)]
pub struct DataValue {
    pub temperature: f32,
    pub humidity: f32,
}

impl DataValue {
    pub fn new(temperature: f32, humidity: f32) -> Self {
        DataValue {
            temperature: temperature,
            humidity: humidity,
        }
    }

    pub fn from_reading(value: &dht22_pi::Reading) -> Self {
        let dht22_pi::Reading {
            temperature: temp,
            humidity: hum,
        } = *value;
        DataValue {
            temperature: temp,
            humidity: hum,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataErrorKind {
    Timeout,
    Integrity,
    IO,
    Runtime,
}

impl DataErrorKind {
    pub fn from_error(error: &dht22_pi::ReadingError) -> Self {
        match *error {
            dht22_pi::ReadingError::Timeout => DataErrorKind::Timeout,
            dht22_pi::ReadingError::Checksum => DataErrorKind::Integrity,
            dht22_pi::ReadingError::Gpio(_) => DataErrorKind::IO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataError {
    pub last_error: DataErrorKind,
    pub error_count: usize,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub data_point: Option<DataValue>,
    pub data_error: Option<DataError>,
}

impl Observation {
    pub fn new() -> Self {
        Observation {
            data_point: None,
            data_error: None,
        }
    }

    pub fn add_data(&mut self, value: DataValue) {
        self.data_point = Some(value);
        self.data_error = None;
    }

    pub fn add_error(&mut self, kind: DataErrorKind) {
        let data_error = match self.data_error.take() {
            Some(mut data_error) => {
                data_error.last_error = kind;
                data_error.error_count += 1;
                data_error
            }
            None => DataError {
                last_error: kind,
                error_count: 1,
            },
        };

        if data_error.error_count > 10 {
            self.data_point = None;
        }
        self.data_error = Some(data_error);
    }
}

#[cfg(test)]
mod tests {
    use {dht22_pi, rppal};

    use observation::{DataErrorKind, DataValue, Observation};

    #[test]
    fn value_from_dht22() {
        // given DHT22 Reading
        let temperature = 10.0;
        let humidity = 11.1;
        let reading = dht22_pi::Reading {
            temperature: temperature,
            humidity: humidity,
        };

        // when DataValue is created from reading
        let data_value = DataValue::from_reading(&reading);

        // then data and error are missing
        assert_eq!(data_value.temperature, temperature);
        assert_eq!(data_value.humidity, humidity);
    }

    #[test]
    fn errorkind_from_dht22() {
        assert_eq!(
            DataErrorKind::from_error(&dht22_pi::ReadingError::Timeout),
            DataErrorKind::Timeout
        );
        assert_eq!(
            DataErrorKind::from_error(&dht22_pi::ReadingError::Checksum),
            DataErrorKind::Integrity
        );
        assert_eq!(
            DataErrorKind::from_error(&dht22_pi::ReadingError::Gpio(
                rppal::gpio::Error::DevMemNotFound,
            )),
            DataErrorKind::IO
        );
    }

    #[test]
    fn no_data_by_default() {
        // given an observation that's just been created
        let observation = Observation::new();
        // then data and error are missing
        assert!(observation.data_point.is_none());
        assert!(observation.data_error.is_none());
    }

    #[test]
    fn one_data_point() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();
        let temperature = 10.0;
        let humidity = 11.1;

        // when a data point is added
        observation.add_data(DataValue::new(temperature, humidity));

        // then data is updated
        assert!(observation.data_point.is_some());
        let data_value = observation.data_point.unwrap();
        assert_eq!(data_value.temperature, temperature);
        assert_eq!(data_value.humidity, humidity);
        // and error is not set
        assert!(observation.data_error.is_none());
    }

    #[test]
    fn one_error() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();
        let kind = DataErrorKind::Integrity;

        // when a data point is added
        observation.add_error(kind.clone());

        // then last error is updated
        let data_error = observation.data_error.unwrap();
        assert_eq!(data_error.last_error, kind);
        // and error counter is initialized
        assert_eq!(data_error.error_count, 1);
        // and data is missing
        assert!(observation.data_point.is_none());
    }

    #[test]
    fn error_counter() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();

        // when several errors are added in a row
        observation.add_error(DataErrorKind::Timeout);
        observation.add_error(DataErrorKind::Integrity);
        observation.add_error(DataErrorKind::Runtime);
        observation.add_error(DataErrorKind::IO);

        // then error counter is properly updated
        let data_error = observation.data_error.unwrap();
        assert_eq!(data_error.error_count, 4);
        // and last error wins
        assert_eq!(data_error.last_error, DataErrorKind::IO);
    }

    #[test]
    fn error_reset_on_success() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();
        let kind = DataErrorKind::Integrity;
        let temperature = 10.0;
        let humidity = 11.1;

        // when a data point is added after an error
        observation.add_error(kind.clone());
        observation.add_data(DataValue::new(temperature, humidity));

        // then data is updated
        assert!(observation.data_point.is_some());
        let data_value = observation.data_point.unwrap();
        assert_eq!(data_value.temperature, temperature);
        assert_eq!(data_value.humidity, humidity);
        // and error is reset
        assert!(observation.data_error.is_none());
    }

    #[test]
    fn result_cached_on_several_errors() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();
        let temperature = 10.0;
        let humidity = 11.1;

        // when a data point is added successfully and then followed by several errors
        observation.add_data(DataValue::new(temperature, humidity));
        observation.add_error(DataErrorKind::Timeout);
        observation.add_error(DataErrorKind::Integrity);
        observation.add_error(DataErrorKind::IO);
        observation.add_error(DataErrorKind::Runtime);

        // then data is cached
        assert!(observation.data_point.is_some());
        let data_value = observation.data_point.unwrap();
        assert_eq!(data_value.temperature, temperature);
        assert_eq!(data_value.humidity, humidity);
        // and error is set
        assert!(observation.data_error.is_some());
    }

    #[test]
    fn result_not_cached_on_too_many_errors() {
        // given a mutable observation that's just been created
        let mut observation = Observation::new();
        let temperature = 10.0;
        let humidity = 11.1;

        // when a data point is added successfully and then followed by too many errors
        observation.add_data(DataValue::new(temperature, humidity));
        for _ in 1..100 {
            observation.add_error(DataErrorKind::Timeout);
        }

        // then data is not cached
        assert!(observation.data_point.is_none());
        // and error is set
        assert!(observation.data_error.is_some());
    }
}
