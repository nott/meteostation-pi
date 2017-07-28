use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Server, Request, Response};
use prometheus::{self, Encoder};

use core;


lazy_static! {
    static ref PROMETHEUS_MIME: Mime = {
        let encoder = prometheus::TextEncoder::new();
        let mime_str = encoder.format_type();
        let mime: Mime = match mime_str.parse() {
            Result::Ok(mime) => mime,
            Result::Err(_) => Mime(TopLevel::Text, SubLevel::Plain, vec![])
        };
        mime
    };
}


pub fn server(core: core::Core) {
    let addr = "127.0.0.1:9898";
    info!("Listening {}", &addr);
    Server::http(addr)
        .unwrap()
        .handle(move |request: Request, mut res: Response| {
            let metrics = core.to_string();
            res.headers_mut().set(ContentType(PROMETHEUS_MIME.clone()));
            res.send(&metrics.as_bytes()).unwrap();
            info!("HTTP {} {}", &request.method, &request.uri);
        })
        .unwrap();
}


#[cfg(test)]
mod tests {
    use web::PROMETHEUS_MIME;

    #[test]
    fn prometheus_mime() {
        let expected = "text/plain; version=0.0.4";
        let actual = PROMETHEUS_MIME.clone().to_string();
        assert_eq!(actual, expected);
    }
}
