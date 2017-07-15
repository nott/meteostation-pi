use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::server::{Server, Request, Response};

use core;


pub fn server(core: core::Core) {
    let addr = "127.0.0.1:9898";
    Server::http(addr)
        .unwrap()
        .handle(move |_: Request, mut res: Response| {
            let metrics = core.to_string();
            // TODO: take mime-type from encoder
            let content_type = ContentType(Mime(TopLevel::Text, SubLevel::Plain, vec![]));
            res.headers_mut().set(content_type);
            res.send(&metrics.as_bytes()).unwrap();
        })
        .unwrap();
}
