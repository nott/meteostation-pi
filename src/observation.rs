use dht22_pi;

#[derive(Debug, Clone)]
pub struct DataValue {
    pub temperature: f32,
    pub humidity: f32,
}

impl DataValue {
    pub fn from_reading(value: &dht22_pi::Reading) -> Self {
        match *value {
            dht22_pi::Reading {
                temperature: temp,
                humidity: hum,
            } => {
                DataValue {
                    temperature: temp,
                    humidity: hum,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataErrorKind {
    Timeout,
    Integrity,
    IO,
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
