extern crate dht22_pi;

use std::sync::{Arc, RwLock, Mutex, mpsc};
use std::time;
use std::thread;

#[derive(Debug, Clone)]
pub struct DataValue {
    pub temperature: f32,
    pub humidity: f32,
}

impl DataValue {
    fn from_reading(value: &dht22_pi::Reading) -> Self {
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
    fn from_error(error: &dht22_pi::ReadingError) -> Self {
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
    fn new() -> Self {
        Observation {
            data_point: None,
            data_error: None,
        }
    }

    fn add_data(&mut self, value: DataValue) {
        self.data_point = Some(value);
        self.data_error = None;
    }

    fn add_error(&mut self, kind: DataErrorKind) {
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

pub trait Sensor {
    fn get_observation(&self) -> Result<Observation, &'static str>;
}

enum AsyncDhtSensorCommand {
    Stop,
}

pub struct AsyncDhtSensor {
    observation: Arc<RwLock<Observation>>,
    pin: u8,
    handle: Option<thread::JoinHandle<()>>,
    handle_tx: Mutex<mpsc::Sender<AsyncDhtSensorCommand>>,
}

impl AsyncDhtSensor {
    pub fn new(pin: u8) -> Self {
        let observation = Arc::new(RwLock::new(Observation::new()));

        let observation_bg = observation.clone();
        let (tx, rx) = mpsc::channel::<AsyncDhtSensorCommand>();
        let handle = thread::spawn(move || loop {
            let mut observation_mut = observation_bg.write().unwrap();
            match dht22_pi::read(pin) {
                Result::Ok(reading) => {
                    (*observation_mut).add_data(DataValue::from_reading(&reading));
                }
                Result::Err(error) => {
                    (*observation_mut).add_error(DataErrorKind::from_error(&error));
                }
            }
            match rx.recv_timeout(time::Duration::from_secs(10)) {
                Result::Ok(_) => return,
                Result::Err(_) => (),
            }
        });

        AsyncDhtSensor {
            observation: observation,
            pin: pin,
            handle: Some(handle),
            handle_tx: Mutex::new(tx),
        }
    }
}

impl Drop for AsyncDhtSensor {
    fn drop(&mut self) {
        let handle = self.handle.take();
        match handle {
            Some(join_handler) => {
                let handle_tx = self.handle_tx.lock().unwrap();
                match (*handle_tx).send(AsyncDhtSensorCommand::Stop) {
                    Result::Ok(()) => {join_handler.join().unwrap();}
                    Result::Err(_) => (),
                }
            }
            None => (),
        }
    }
}

impl Sensor for AsyncDhtSensor {
    fn get_observation(&self) -> Result<Observation, &'static str> {
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
