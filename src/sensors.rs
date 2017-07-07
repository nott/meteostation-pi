use std::sync::{Arc, RwLock, Mutex, mpsc};
use std::time;
use std::thread;

use dht22_pi;

use observation::{DataErrorKind, DataValue, Observation};

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
        let handle = thread::spawn(move || Self::read_loop(observation_bg, pin, rx));

        AsyncDhtSensor {
            observation: observation,
            pin: pin,
            handle: Some(handle),
            handle_tx: Mutex::new(tx),
        }
    }

    fn read_loop(
        observation: Arc<RwLock<Observation>>,
        pin: u8,
        control_channel: mpsc::Receiver<AsyncDhtSensorCommand>,
    ) {
        loop {
            if let Result::Ok(mut observation_mut) = observation.write() {
                match dht22_pi::read(pin) {
                    Result::Ok(reading) => {
                        (*observation_mut).add_data(DataValue::from_reading(&reading));
                    }
                    Result::Err(error) => {
                        (*observation_mut).add_error(DataErrorKind::from_error(&error));
                    }
                }
            }
            match control_channel.recv_timeout(time::Duration::from_secs(10)) {
                Result::Ok(AsyncDhtSensorCommand::Stop) => return,
                Result::Err(_) => (),
            }
        }
    }
}

impl Drop for AsyncDhtSensor {
    fn drop(&mut self) {
        if let Some(join_handler) = self.handle.take() {
            if let Result::Ok(handle_tx) = self.handle_tx.lock() {
                if (*handle_tx).send(AsyncDhtSensorCommand::Stop).is_ok() {
                    let _ = join_handler.join();
                }
            }
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
