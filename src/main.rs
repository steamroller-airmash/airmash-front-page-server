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
mod games;

use actix_web::{http, middleware, server, App, HttpRequest, HttpResponse};
use http::Method;

use sentry::{Hub, Level};
use sentry_actix::{ActixWebHubExt, SentryMiddleware};

use std::str;

use spec::*;
use games::games;

const CONFIG_FILE: &'static str = include_str!("../config.json");

lazy_static! {
	static ref CONFIG: GameSpec = serde_json::from_str(CONFIG_FILE).unwrap();
}

/// Log the client error to sentry for investigation
/// later. If a sentry dsn is not provided in the
/// SENTRY_DSN this is a no-op
fn clienterror(req: &HttpRequest) -> HttpResponse {
	let hub = Hub::from_request(req);
	hub.capture_message("A client error occurred", Level::Error);

	HttpResponse::Ok().body("")
}

/// Not sure what this should do yet. One option
/// would be to forward to the official servers
fn login(_: &HttpRequest) -> HttpResponse {
	HttpResponse::NotImplemented().body("")
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
			.resource("/games", |r| r.method(Method::GET).f(games))
			.resource("/clienterror", |r| r.method(Method::POST).f(clienterror))
			.resource("/login", |r| r.method(Method::POST).f(login))
	}).bind("0.0.0.0:9000")
		.unwrap()
		.shutdown_timeout(1)
		.run();
}
