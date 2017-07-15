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
