extern crate meteostation_pi;

use std::time;

use meteostation_pi::core::Core;
use meteostation_pi::closures::UpdateClosure;
use meteostation_pi::poller::Poller;
use meteostation_pi::sensors::GpioSensor;
use meteostation_pi::web;


fn main() {
    let core = Core::new();
    let sensor = GpioSensor::new(4);
    let fn_update = UpdateClosure::new(sensor, core.get_metrics());
    Poller::new(time::Duration::from_secs(10), fn_update);
    web::server(core);
}
