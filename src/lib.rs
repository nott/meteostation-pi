extern crate dht22_pi;

use std::sync::{Arc, RwLock};
use std::time;
use std::thread;

#[derive(Debug, Clone)]
pub struct DataValue {
    pub temperature: f32,
    pub humidity: f32,
}

#[derive(Debug, Clone)]
pub enum DataErrorKind {
    Timeout,
    Checksum,
    IO,
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
    fn new() -> Self {
        Observation {
            data_point: None,
            data_error: None,
        }
    }
}

trait Sensor {
    fn get_observation(self) -> Result<Observation, &'static str>;
}

pub struct AsyncDhtSensor {
    observation: Arc<RwLock<Observation>>,
    pin: u8,
    handle: Option<thread::JoinHandle<()>>,
}

impl AsyncDhtSensor {
    fn new(pin: u8) -> Self {
        let observation = Arc::new(RwLock::new(Observation::new()));
        let handle = thread::spawn(move || {
            thread::sleep(time::Duration::from_secs(10));
            dht22_pi::read(pin).unwrap();
        });

        AsyncDhtSensor {
            observation: observation,
            pin: pin,
            handle: Some(handle),
        }
    }
}

impl Sensor for AsyncDhtSensor {
    fn get_observation(self) -> Result<Observation, &'static str> {
        match self.observation.read() {
            Ok(value) => Ok(value.clone()),
            Err(_) => Err("Sensor is poisoned"),
        }
    }
}

#[cfg(test)]
mod tests {
    use AsyncDhtSensor;
    use Sensor;

    #[test]
    fn new_sensor_is_readable() {
        let pin = 0;
        let sensor = AsyncDhtSensor::new(pin);
        let observation = sensor.get_observation();
        assert!(observation.is_ok());
    }
}
