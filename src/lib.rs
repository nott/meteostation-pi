use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
pub struct DataValue {
    pub temperature: f32,
    pub humidity: f32
}

#[derive(Debug, Clone)]
pub enum DataErrorKind {
    Timeout,
    Checksum,
    IO
}

#[derive(Debug, Clone)]
pub struct DataError {
    pub last_error: DataErrorKind,
    pub error_count: usize
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub data_point: Option<DataValue>,
    pub data_error: Option<DataError>
}

impl Observation {
    fn new() -> Self {
        Observation { data_point: None, data_error: None }
    }
}

pub struct Sensor {
    observation: Arc<RwLock<Observation>>,
    pin: u8,
    handle: Option<JoinHandle<()>>
}

impl Sensor {
    fn new(pin: u8) -> Self {
        let observation = Arc::new(RwLock::new(Observation::new()));
        Sensor { observation: observation, pin: pin, handle: None }
    }

    pub fn get_observation(self) -> Result<Observation, & 'static str> {
        match self.observation.read() {
            Ok(value) => Ok(value.clone()),
            Err(_) => Err("Sensor is poisoned")
        }
    }
}

#[cfg(test)]
mod tests {
    use ::Sensor;

    #[test]
    fn new_sensor_is_readable() {
        let pin = 0;
        let sensor = Sensor::new(pin);
        let observation = sensor.get_observation();
        assert!(observation.is_ok());
    }
}
