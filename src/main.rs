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
extern crate sentry;
extern crate sentry_actix;

mod spec;

use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::{http, middleware, server, App, Error, HttpResponse};
use futures::future::join_all;
use futures::{Future, Stream};

use sentry::{Hub, Level};
use sentry_actix::{SentryMiddleware, ActixWebHubExt};

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
    sentry::init(());

	server::new(move || {
		App::new()
			.middleware(middleware::Logger::default())
			.middleware(SentryMiddleware::new())
			.resource("/games", move |r| {
				r.method(http::Method::GET).f(
					move |req| -> Box<Future<Item = HttpResponse, Error = Error>> {
						let https = HttpsConnector::new(4).unwrap();
						let client: Client<_, Body> = Client::builder()
							.build(https);

						let headers = req.request().headers();
						let country = headers.get("CF-IPCountry")
							.map(|x| x.to_str().unwrap_or("XX"))
							.unwrap_or("XX")
							.to_owned();

						let mut regions = vec![];
						let config = CONFIG.clone();

						for region in config.data.iter() {
							let mut requests = vec![];

							for server in region.games.iter() {
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
									country: country,
									data: regions,
								})
								.map(|spec| serde_json::to_string(&spec).unwrap())
								.map(|resp| {
									HttpResponse::Ok()
										.header(http::header::CONTENT_TYPE, "application/json; charset=utf-8")
										.body(resp)
								})
								.map_err(|e| e.into()),
						)
					},
				)
			})
			.resource("/clienterror", move |r| {
				r.method(http::Method::POST).f(
					move |req| -> HttpResponse {
						let hub = Hub::from_request(req);
						hub.capture_message("A client error occurred", Level::Error);

						HttpResponse::Ok()
							.body("")
					},
				)
			})
	}).bind("0.0.0.0:9000")
		.unwrap()
		.shutdown_timeout(1)
		.run();
}
