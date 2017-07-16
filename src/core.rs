use std::string::ToString;

use prometheus::{self, Encoder};


#[derive(Clone)]
pub struct Metrics {
    temperature: prometheus::Gauge,
    humidity: prometheus::Gauge,
    ok_count: prometheus::Gauge,
    error_count: prometheus::Gauge,
}


impl Metrics {
    pub fn new(registry: &prometheus::Registry) -> Self {
        let temperature_opts =
            prometheus::Opts::new("temperature", "Current temperature (Celsius)");
        let temperature = prometheus::Gauge::with_opts(temperature_opts).unwrap();
        registry.register(Box::new(temperature.clone())).unwrap();

        let humidity_opts = prometheus::Opts::new("humidity", "Current humidity (percent)");
        let humidity = prometheus::Gauge::with_opts(humidity_opts).unwrap();
        registry.register(Box::new(humidity.clone())).unwrap();

        let ok_count_opts = prometheus::Opts::new("ok_count", "Number of successful sensor reads");
        let ok_count = prometheus::Gauge::with_opts(ok_count_opts).unwrap();
        registry.register(Box::new(ok_count.clone())).unwrap();

        let error_count_opts =
            prometheus::Opts::new("error_count", "Number of unsuccessful sensor reads");
        let error_count = prometheus::Gauge::with_opts(error_count_opts).unwrap();
        registry.register(Box::new(error_count.clone())).unwrap();

        Metrics {
            temperature: temperature,
            humidity: humidity,
            ok_count: ok_count,
            error_count: error_count,
        }
    }

    pub fn set_value(&self, temperature: f64, humidity: f64) {
        self.temperature.set(temperature);
        self.humidity.set(humidity);
        self.ok_count.inc();
        self.error_count.set(0.0);
    }

    pub fn set_error(&self) {
        self.error_count.inc();
        self.ok_count.set(0.0);
    }

    pub fn get_temperature(&self) -> f64 {
        self.temperature.get()
    }

    pub fn get_humidity(&self) -> f64 {
        self.humidity.get()
    }

    pub fn get_ok_count(&self) -> f64 {
        self.ok_count.get()
    }

    pub fn get_error_count(&self) -> f64 {
        self.error_count.get()
    }
}

pub struct Core {
    registry: prometheus::Registry,
    metrics: Metrics,
}

impl Core {
    pub fn new() -> Self {
        let registry = prometheus::Registry::new();
        let metrics = Metrics::new(&registry);
        Core {
            registry: registry,
            metrics: metrics,
        }
    }

    pub fn get_metrics(&self) -> Metrics {
        self.metrics.clone()
    }
}

impl ToString for Core {
    fn to_string(&self) -> String {
        let mut buffer = Vec::<u8>::new();
        let encoder = prometheus::TextEncoder::new();

        let metric_familys = self.registry.gather();
        encoder.encode(&metric_familys, &mut buffer).unwrap();

        String::from_utf8(buffer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use core::Core;

    #[test]
    fn test_empty() {
        let core = Core::new();
        let expected_metrics = "# HELP error_count Number of unsuccessful sensor reads\n\
             # TYPE error_count gauge\n\
             error_count 0\n\
             # HELP humidity Current humidity (percent)\n\
             # TYPE humidity gauge\n\
             humidity 0\n\
             # HELP ok_count Number of successful sensor reads\n\
             # TYPE ok_count gauge\n\
             ok_count 0\n\
             # HELP temperature Current temperature (Celsius)\n\
             # TYPE temperature gauge\n\
             temperature 0\n";
        assert_eq!(core.to_string(), expected_metrics);
    }
}
