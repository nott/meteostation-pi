extern crate env_logger;
extern crate meteostation_pi;

use std::time;
use std::env;

use meteostation_pi::core::Core;
use meteostation_pi::closures::UpdateClosure;
use meteostation_pi::poller::Poller;
use meteostation_pi::sensors::GpioSensor;
use meteostation_pi::web;


fn init_logging() {
    const RUST_LOG: &str = "RUST_LOG";
    if env::var_os(RUST_LOG).is_none() {
        env::set_var(RUST_LOG, "info");
    }

    env_logger::init().unwrap();
}


fn main() {
    init_logging();
    let core = Core::new();
    let sensor = GpioSensor::new(4);
    let fn_update = UpdateClosure::new(sensor, core.get_metrics());
    let _poller = Poller::new(time::Duration::from_secs(10), fn_update);
    web::server(core);
}
