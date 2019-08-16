use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use log::{debug, info, trace};
use dotenv::dotenv;

use std::env;


fn main() {
    dotenv().ok();

    pretty_env_logger::init();
    info!("Rand microservice - 0.1.0");
    trace!("Starting...");

    let addr = env::var("ADDRESS")
        .unwrap_or_else(|_|"127.0.0.1:8080".into())
        .parse()
        .expect("Can't parse ADDRESS variable");

    debug!("Trying to bind to server address {}", addr);
    let builder = Server::bind(&addr);

    trace!("Creating the service handler");
    let server = builder.serve(|| {
        service_fn_ok(|req|{
            trace!("Incoming request is: {:?}", req);
            let random_byte = rand::random::<u8>();
            debug!("Generate value is: {}", random_byte);
            Response::new(Body::from(random_byte.to_string()))
        })
    });
    info!("Used address: {}", server.local_addr());
    let server = server.map_err(drop);
    debug!("Run!");
    hyper::rt::run(server);
}
