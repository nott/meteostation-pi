use core::Metrics;
use sensors::{DataPoint, Sensor};

pub struct UpdateClosure<S> {
    sensor: S,
    metrics: Metrics,
}

impl<S> UpdateClosure<S>
where
    S: Sensor,
{
    pub fn new(sensor: S, metrics: Metrics) -> Self {
        UpdateClosure {
            sensor: sensor,
            metrics: metrics,
        }
    }

    fn inner_call(&self) {
        match self.sensor.read() {
            Result::Ok(DataPoint {
                           temperature: t,
                           humidity: h,
                       }) => {
                info!("Sensor: t={}, humidity={}", t, h);
                self.metrics.set_value(t, h);
            }
            Result::Err(err) => {
                warn!("Sensor: error={}", err);
                self.metrics.set_error();
            }
        }
    }
}

impl<S> FnOnce<()> for UpdateClosure<S>
where
    S: Sensor,
{
    type Output = ();
    #[allow(unused_variables)]
    extern "rust-call" fn call_once(self, args: ()) {
        self.inner_call();
    }
}

impl<S> FnMut<()> for UpdateClosure<S>
where
    S: Sensor,
{
    #[allow(unused_variables)]
    extern "rust-call" fn call_mut(&mut self, args: ()) {
        self.inner_call();
    }
}

impl<S> Fn<()> for UpdateClosure<S>
where
    S: Sensor,
{
    #[allow(unused_variables)]
    extern "rust-call" fn call(&self, args: ()) {
        self.inner_call();
    }
}

#[cfg(test)]
mod tests {
    use sensors::{DataPoint, ErrorKind, ErrSensor, OkSensor, SequenceSensor};
    use closures::UpdateClosure;
    use core::Core;

    const TEST_TEMPERATURE: f64 = 24.0;
    const TEST_HUMIDITY: f64 = 55.1;

    #[test]
    fn test_update_closure_ok() {
        // given a sensor that always return data without errors
        let ok_sensor = OkSensor::new(TEST_TEMPERATURE, TEST_HUMIDITY);
        // and a core instance
        let core = Core::new();
        let metrics1 = core.get_metrics();
        let metrics2 = core.get_metrics();
        // and an update closure with the sensor above
        let update_fn = UpdateClosure::new(ok_sensor, metrics1);

        // when update function is called several times in a row
        let iterations = 5;
        for _ in 0..iterations {
            update_fn();
        }

        // then temperature and humidity are properly recorded
        assert_eq!(metrics2.get_temperature(), TEST_TEMPERATURE);
        assert_eq!(metrics2.get_humidity(), TEST_HUMIDITY);
        // and ok count equals to the number of iterations
        assert_eq!(metrics2.get_ok_count(), iterations as f64);
        // and there were no errors registered
        assert_eq!(metrics2.get_error_count(), 0.0);
    }

    #[test]
    fn test_update_closure_err() {
        // given a sensor that always returns errors
        let err_sensor = ErrSensor::new(ErrorKind::Integrity);
        // and a core instance
        let core = Core::new();
        let metrics1 = core.get_metrics();
        let metrics2 = core.get_metrics();
        // and an update closure with the sensor above
        let update_fn = UpdateClosure::new(err_sensor, metrics1);

        // when update function is called several times in a row
        let iterations = 5;
        for _ in 0..iterations {
            update_fn();
        }

        // then temperature and humidity have default value 0.0
        assert_eq!(metrics2.get_temperature(), 0.0);
        assert_eq!(metrics2.get_humidity(), 0.0);
        // and no successful observations were registered
        assert_eq!(metrics2.get_ok_count(), 0.0);
        // and error count equals to the number of iterations
        assert_eq!(metrics2.get_error_count(), iterations as f64);
    }

    #[test]
    fn test_update_closure_mixed() {
        // given a sensor with predefined results sequence
        let mut results = Vec::new();
        results.push(Result::Err(ErrorKind::IO));
        results.push(Result::Ok(
            DataPoint::new(TEST_TEMPERATURE - 1.0, TEST_HUMIDITY - 1.0),
        ));
        results.push(Result::Ok(DataPoint::new(TEST_TEMPERATURE, TEST_HUMIDITY)));
        results.push(Result::Err(ErrorKind::Integrity));
        let sensor = SequenceSensor::new(results);
        // and a core instance
        let core = Core::new();
        let metrics1 = core.get_metrics();
        let metrics2 = core.get_metrics();
        // and an update closure with the sensor above
        let update_fn = UpdateClosure::new(sensor, metrics1);

        // when update function is called 3 times in a row
        for _ in 0..3 {
            update_fn();
        }
        // then temperature and humidity are taken from the 3rd value
        assert_eq!(metrics2.get_temperature(), TEST_TEMPERATURE);
        assert_eq!(metrics2.get_humidity(), TEST_HUMIDITY);
        // and ok count equals to 2 because we had only 2 non-error data points in a row
        assert_eq!(metrics2.get_ok_count(), 2.0);
        // and error count is reset because the previous result was successful
        assert_eq!(metrics2.get_error_count(), 0.0);

        // and when update function is called one more time
        update_fn();
        // then temperature and humidity are taken from the last successful execution
        assert_eq!(metrics2.get_temperature(), TEST_TEMPERATURE);
        assert_eq!(metrics2.get_humidity(), TEST_HUMIDITY);
        // and ok count is reset on error
        assert_eq!(metrics2.get_ok_count(), 0.0);
        // and error count is increased
        assert_eq!(metrics2.get_error_count(), 1.0);
    }
}
