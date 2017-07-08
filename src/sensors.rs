use observation::Observation;
pub use sensors::dht::DhtSensor;

pub trait Sensor {
    fn get_observation(&self) -> Result<Observation, &'static str>;
}

mod dht {
    use std::sync::{Arc, RwLock};
    use std::time;

    use dht22_pi;

    use observation::{DataErrorKind, DataValue, Observation};
    use sensors::poller;

    pub struct DhtSensor {
        pub observation: Arc<RwLock<Observation>>,
        pub pin: u8,
        _poller: poller::Poller,
    }

    impl DhtSensor {
        pub fn new(pin: u8) -> Self {
            let observation = Arc::new(RwLock::new(Observation::new()));

            let update_fn = UpdateClosure::new(observation.clone(), pin);
            let poller = poller::Poller::new(time::Duration::from_secs(10), update_fn);

            DhtSensor {
                observation: observation,
                pin: pin,
                _poller: poller,
            }
        }
    }

    struct UpdateClosure {
        observation: Arc<RwLock<Observation>>,
        pin: u8,
    }

    impl UpdateClosure {
        pub fn new(observation: Arc<RwLock<Observation>>, pin: u8) -> Self {
            UpdateClosure {
                observation: observation.clone(),
                pin: pin,
            }
        }

        fn inner_call(&self) {
            if let Result::Ok(mut observation_mut) = self.observation.write() {
                match dht22_pi::read(self.pin) {
                    Result::Ok(reading) => {
                        (*observation_mut).add_data(DataValue::from_reading(&reading));
                    }
                    Result::Err(error) => {
                        (*observation_mut).add_error(DataErrorKind::from_error(&error));
                    }
                }
            }
        }
    }

    impl FnOnce<()> for UpdateClosure {
        type Output = ();
        extern "rust-call" fn call_once(self, args: ()) {
            self.inner_call();
        }
    }

    impl FnMut<()> for UpdateClosure {
        extern "rust-call" fn call_mut(&mut self, args: ()) {
            self.inner_call();
        }
    }

    impl Fn<()> for UpdateClosure {
        extern "rust-call" fn call(&self, args: ()) {
            self.inner_call();
        }
    }
}

impl Sensor for DhtSensor {
    fn get_observation(&self) -> Result<Observation, &'static str> {
        match self.observation.read() {
            Ok(value) => Ok(value.clone()),
            Err(_) => Err("Sensor is poisoned"),
        }
    }
}

mod poller {
    use std::sync::{Mutex, mpsc};
    use std::time;
    use std::thread;

    enum Command {
        Stop,
    }

    pub struct Poller {
        handle: Option<thread::JoinHandle<()>>,
        handle_tx: Mutex<mpsc::Sender<Command>>,
    }

    impl Poller {
        pub fn new<F>(interval: time::Duration, f: F) -> Self
        where
            F: Fn() -> (),
            F: Send + 'static,
        {
            let (tx, rx) = mpsc::channel::<Command>();
            let handle = thread::spawn(move || Self::poll(interval, rx, f));
            Poller {
                handle: Some(handle),
                handle_tx: Mutex::new(tx),
            }
        }

        fn poll<F>(interval: time::Duration, control_channel: mpsc::Receiver<Command>, f: F)
        where
            F: Fn() -> (),
            F: Send + 'static,
        {
            loop {
                f();
                match control_channel.recv_timeout(interval) {
                    Result::Ok(Command::Stop) => return,
                    Result::Err(_) => (),
                }
            }
        }
    }

    impl Drop for Poller {
        fn drop(&mut self) {
            if let Some(join_handler) = self.handle.take() {
                if let Result::Ok(handle_tx) = self.handle_tx.lock() {
                    if (*handle_tx).send(Command::Stop).is_ok() {
                        let _ = join_handler.join();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use sensors::dht::DhtSensor;
    use Sensor;

    #[test]
    fn new_sensor_is_readable() {
        let pin = 0;
        let sensor = DhtSensor::new(pin);
        let observation = sensor.get_observation();
        assert!(observation.is_ok());
    }
}
