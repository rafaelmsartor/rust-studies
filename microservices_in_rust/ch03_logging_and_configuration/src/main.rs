use hyper::{Body, Response, Server};
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use log::{debug, info, warn, trace};
use dotenv::dotenv;
use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, App};
use serde_derive::Deserialize;

use std::env;
use std::io::{self, Read};
use std::fs::File;
use std::net::SocketAddr;

#[derive(Deserialize)]
struct Config {
    address: SocketAddr,
}

fn main() {
    dotenv().ok();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("address")
            .short("a")
            .long("address")
            .value_name("ADDRESS")
            .help("Sets an address")
            .takes_value(true))
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .get_matches();

    pretty_env_logger::init();
    info!("Rand microservice - 0.1.0");
    trace!("Starting...");

    let config_file_name = matches.value_of("config")
        .map(|s| s.to_owned())
        .or_else(|| Some(String::from("microservice.toml")))
        .unwrap();

    let config = File::open(config_file_name)
        .and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            Ok(buffer)
        })
        .and_then(|buffer| {
            toml::from_str::<Config>(&buffer)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
        })
        .map_err(|err| {
            warn!("Can't read config file: {}", err);
        })
        .ok();


    let addr = matches.value_of("address")
        .map(|s| s.to_owned())
        .or(env::var("ADDRESS").ok())
        .and_then(|addr| addr.parse().ok())
        .or(config.map(|config| config.address))
        .or_else(|| Some(([127, 0, 0, 1], 8080).into()))
        .unwrap();

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
