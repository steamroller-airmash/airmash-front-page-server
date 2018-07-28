extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate serde;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod spec;

use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::{http, middleware, server, App, Error, HttpResponse};
use futures::future::join_all;
use futures::{Future, Stream};

use hyper::{Body, Client};
use hyper_tls::HttpsConnector;

use std::str;

use spec::*;

const CONFIG_FILE: &'static str = include_str!("../config.json");

lazy_static! {
	static ref CONFIG: GameSpec = serde_json::from_str(CONFIG_FILE).unwrap();
}

fn main() {
	std::env::set_var("RUST_LOG", "info");
	std::env::set_var("RUST_BACKTRACE", "1");
	env_logger::init();

	server::new(move || {
		App::new()
			.middleware(middleware::Logger::default())
			.resource("/", move |r| {
				r.method(http::Method::GET).f(
					move |req| -> Box<Future<Item = HttpResponse, Error = Error>> {
						info!("{:?}", req);

						let https = HttpsConnector::new(4).unwrap();
						let client: Client<_, Body> = Client::builder()
							.build(https);

						let mut regions = vec![];
						let config = CONFIG.clone();

						for region in config.data.iter() {
							let mut requests = vec![];

							for server in region.games.iter() {
								info!("{:?}", server);
								requests.push(
									client
										.get(server.url.parse().unwrap())
										.and_then(|response| {
											response.into_body().fold(
												vec![],
												|mut acc, chunk| -> Result<Vec<_>, hyper::Error> {
													acc.extend_from_slice(&*chunk);
													Ok(acc)
												},
											)
										})
										.map(|v: Vec<u8>| {
											println!("{:?}", v);
											println!("Incoming text: {}", str::from_utf8(&v).unwrap());
											serde_json::from_slice(&v).unwrap()
										})
										.map(|v: ServerResponse| v.players)
										.map_err(|e| {
											error!("Error: {:?}", e);
											InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR)
										}),
								);
							}

							let region = region.clone();

							regions.push(join_all(requests).map(move |counts| {
								let games = region
									.games
									.into_iter()
									.zip(counts.into_iter())
									.map(|(game, count)| ServerSpec {
										players: count,
										..game
									})
									.collect();

								RegionSpec { games, ..region }
							}));
						}

						Box::new(
							join_all(regions)
								.map(|regions| GameSpec {
									protocol: 5,
									country: "XX".to_owned(),
									data: regions,
								})
								.map(|spec| serde_json::to_string(&spec).unwrap())
								.map(|resp| HttpResponse::Ok().body(resp))
								.map_err(|e| e.into()),
						)
					},
				)
			})
	}).bind("0.0.0.0:9000")
		.unwrap()
		.shutdown_timeout(1)
		.run();
}
