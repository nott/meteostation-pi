use core::Metrics;
use sensors::{DataPoint, Sensor};

pub struct UpdateClosure<S> {
    sensor: S,
    metrics: Metrics,
}

impl<S> UpdateClosure<S> where S: Sensor {
    pub fn new(sensor: S, metrics: Metrics) -> Self {
        UpdateClosure {
            sensor: sensor,
            metrics: metrics,
        }
    }

    fn inner_call(&self) {
        match self.sensor.read() {
            Result::Ok(DataPoint { temperature: t, humidity: h}) => {
                self.metrics.set_value(t, h);
            }
            Result::Err(_) => {
                self.metrics.set_error();
            }
        }
    }
}

impl<S> FnOnce<()> for UpdateClosure<S> where S: Sensor {
    type Output = ();
    #[allow(unused_variables)]
    extern "rust-call" fn call_once(self, args: ()) {
        self.inner_call();
    }
}

impl<S> FnMut<()> for UpdateClosure<S> where S: Sensor {
    #[allow(unused_variables)]
    extern "rust-call" fn call_mut(&mut self, args: ()) {
        self.inner_call();
    }
}

impl<S> Fn<()> for UpdateClosure<S> where S: Sensor {
    #[allow(unused_variables)]
    extern "rust-call" fn call(&self, args: ()) {
        self.inner_call();
    }
}

#[cfg(test)]
mod tests {
    use sensors::OkSensor;
    use closures::UpdateClosure;
    use core::Core;

    #[test]
    fn test_update_closure() {
        let temperature = 24.4;
        let humidity = 45.1;
        let ok_sensor = OkSensor::new(temperature, humidity);
        let core = Core::new();
        let metrics1 = core.get_metrics();
        let metrics2 = core.get_metrics();
        let update_fn = UpdateClosure::new(ok_sensor, metrics1);

        update_fn();

        assert_eq!(metrics2.get_temperature(), temperature);
        assert_eq!(metrics2.get_humidity(), humidity);
        assert_eq!(metrics2.get_ok_count(), 1.0);
        assert_eq!(metrics2.get_error_count(), 0.0);
    }
}
