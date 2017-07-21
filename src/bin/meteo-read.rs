extern crate meteostation_pi;

use meteostation_pi::sensors::{DataPoint, ErrorKind, GpioSensor, Sensor};


fn main() {
    let sensor = GpioSensor::new(4);
    let observation = sensor.read();

    let output = match observation {
        Result::Ok(DataPoint {
                       temperature: t,
                       humidity: h,
                   }) => format!("Temperature:\t{}\nHumidity:\t{}", t, h),
        Result::Err(ErrorKind::Timeout) => "Error: timeout".into(),
        Result::Err(ErrorKind::Integrity) => "Error: integrity".into(),
        Result::Err(ErrorKind::IO) => "Error: IO".into(),
        Result::Err(ErrorKind::Runtime) => "Error: runtime".into(),
    };
    println!("{}\n", output);
}
