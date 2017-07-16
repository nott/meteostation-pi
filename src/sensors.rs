use std::panic;
use std::cell::Cell;
use std::vec::Vec;

use dht22_pi;

#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    pub temperature: f64,
    pub humidity: f64,
}

impl DataPoint {
    pub fn new(temperature: f64, humidity: f64) -> Self {
        DataPoint {
            temperature: temperature,
            humidity: humidity,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Timeout,
    Integrity,
    IO,
    Runtime,
}

/// A trait for measuring temperature and humididty
///
/// There is only one real world implementation (for GpioSensor), but
/// we keep this trait separate to make the code more testable.
pub trait Sensor {
    /// Read temperature and humidity from sensor.
    fn read(&self) -> Result<DataPoint, ErrorKind>;
}

/// DHT22 temperature and humidity sensor
///
/// The only real life struct implementing Sensor trait.
pub struct GpioSensor {
    pin: u8,
}

impl GpioSensor {
    /// Create a new DHT22 sensor instance.
    pub fn new(pin: u8) -> Self {
        GpioSensor { pin: pin }
    }
}

impl Sensor for GpioSensor {
    fn read(&self) -> Result<DataPoint, ErrorKind> {
        match panic::catch_unwind(|| dht22_pi::read(self.pin)) {
            Result::Ok(dht_result) => {
                match dht_result {
                    Result::Ok(dht22_pi::Reading {
                                   temperature: t,
                                   humidity: h,
                               }) => Result::Ok(DataPoint::new(t as f64, h as f64)),
                    Result::Err(dht22_pi::ReadingError::Timeout) => Result::Err(ErrorKind::Timeout),
                    Result::Err(dht22_pi::ReadingError::Checksum) => Result::Err(
                        ErrorKind::Integrity,
                    ),
                    Result::Err(dht22_pi::ReadingError::Gpio(_)) => Result::Err(ErrorKind::IO),
                }
            }
            Result::Err(_) => Result::Err(ErrorKind::Runtime),
        }
    }
}

/// Sensor that always returns a predefined value.
///
/// Only used for testing.
pub struct OkSensor {
    value: DataPoint,
}

impl OkSensor {
    /// Create a new testing sensor instance.
    pub fn new(temperature: f64, humidity: f64) -> Self {
        OkSensor { value: DataPoint::new(temperature, humidity) }
    }
}

impl Sensor for OkSensor {
    fn read(&self) -> Result<DataPoint, ErrorKind> {
        Result::Ok(self.value.clone())
    }
}

/// Sensor that always returns a predefined error.
///
/// Only used for testing.
pub struct ErrSensor {
    error: ErrorKind,
}

impl ErrSensor {
    /// Create a new testing sensor instance.
    pub fn new(error: ErrorKind) -> Self {
        ErrSensor { error: error }
    }
}

impl Sensor for ErrSensor {
    fn read(&self) -> Result<DataPoint, ErrorKind> {
        Result::Err(self.error.clone())
    }
}

/// Sensor that always returns a predefined sequence of values.
///
/// Only used for testing.
pub struct SequenceSensor {
    sequence: Vec<Result<DataPoint, ErrorKind>>,
    position: Cell<usize>,
}

impl SequenceSensor {
    /// Create a new testing sensor instance.
    pub fn new(sequence: Vec<Result<DataPoint, ErrorKind>>) -> Self {
        SequenceSensor {
            sequence: sequence,
            position: Cell::new(0),
        }
    }
}

impl Sensor for SequenceSensor {
    fn read(&self) -> Result<DataPoint, ErrorKind> {
        let index = self.position.get();
        match self.sequence.get(index) {
            Some(value) => {
                self.position.set(index + 1);
                value.clone()
            }
            None => Result::Err(ErrorKind::Runtime),
        }
    }
}

#[cfg(test)]
mod tests {
    use sensors::{DataPoint, ErrorKind, ErrSensor, GpioSensor, OkSensor, Sensor, SequenceSensor};

    const TEST_TEMPERATURE: f64 = 24.0;
    const TEST_HUMIDITY: f64 = 55.1;

    #[test]
    fn gpiosensor_smoke() {
        let test_pin = 255;
        let sensor = GpioSensor::new(test_pin);
        assert!(sensor.read().is_err());
    }

    #[test]
    fn oksensor_smoke() {
        let sensor = OkSensor::new(TEST_TEMPERATURE, TEST_HUMIDITY);
        assert!(sensor.read().is_ok());
    }

    #[test]
    fn errsensor_smoke() {
        let error = ErrorKind::Integrity;
        let sensor = ErrSensor::new(error.clone());
        assert_eq!(sensor.read(), Result::Err(error));
    }

    #[test]
    fn sequencesensor_smoke() {
        let mut results = Vec::new();
        let value0 = Result::Err(ErrorKind::IO);
        results.push(value0.clone());
        let value1 = Result::Ok(DataPoint::new(TEST_TEMPERATURE, TEST_HUMIDITY));
        results.push(value1.clone());

        let sensor = SequenceSensor::new(results);

        assert_eq!(sensor.read(), value0);
        assert_eq!(sensor.read(), value1);
        assert_eq!(sensor.read(), Result::Err(ErrorKind::Runtime));
        assert_eq!(sensor.read(), Result::Err(ErrorKind::Runtime));
    }
}
