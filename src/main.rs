#![feature(never_type)]
// TODO: Fixup clienterror
#![allow(unused_imports)]

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
extern crate percent_encoding;
extern crate sentry;
extern crate sentry_actix;

mod games;
mod login;
mod spec;

use actix_web::{http, middleware, server, App, HttpRequest, HttpResponse};
use http::Method;

use sentry::{Hub, Level};
use sentry_actix::{ActixWebHubExt, SentryMiddleware};

use std::str;
use std::sync::Arc;

use games::games;
use login::{proxy_get, proxy_post, proxy_redirect};
use spec::*;

const CONFIG_FILE: &'static str = include_str!("../config.json");

lazy_static! {
	static ref CONFIG: GameSpec = serde_json::from_str(CONFIG_FILE).unwrap();
}

/// Log the client error to sentry for investigation
/// later. If a sentry dsn is not provided in the
/// SENTRY_DSN this is a no-op
fn clienterror(_req: &HttpRequest) -> HttpResponse {
	//let hub: Arc<Hub> = Arc::from_request(req);
	//req.extensions()
	// .get::<Arc<Hub>>()
	// .expect("Sentry Middleware was not registered")
	// .clone();

	//hub.capture_message("A client error occurred", Level::Error);

	HttpResponse::Ok().body("")
}

fn ping(_: &HttpRequest) -> HttpResponse {
	HttpResponse::Ok().body("{\"pong\":1}")
}

fn enter(_: &HttpRequest) -> HttpResponse {
	HttpResponse::NotImplemented().body("")
}

fn main() {
	std::env::set_var("RUST_LOG", "info");
	std::env::set_var("RUST_BACKTRACE", "1");
	env_logger::init();
	let _guard = sentry::init(());

	server::new(move || {
		App::new()
			.middleware(middleware::Logger::default())
			.middleware(SentryMiddleware::new())
			.resource("/games", |r| r.method(Method::GET).f(games))
			.resource("/clienterror", |r| r.method(Method::POST).f(clienterror))
			.resource("/ping", |r| r.method(Method::GET).f(ping))
			.resource("/enter", |r| r.method(Method::POST).f(enter))
			.resource("/login", |r| {
				r.method(Method::POST)
					.f(proxy_post("https://airma.sh/login"))
			})
			.resource("/auth", |r| {
				r.method(Method::POST)
					.f(proxy_post("https://airma.sh/auth"))
			})
			.resource("/auth_facebook_cb", |r| {
				r.method(Method::GET)
					.f(proxy_get("https://airma.sh/auth_facebook_cb"))
			})
			.resource("/auth_google_cb", |r| {
				r.method(Method::GET)
					.f(proxy_get("https://airma.sh/auth_google_cb"))
			})
			.resource("/auth_twitter_cb", |r| {
				r.method(Method::GET)
					.f(proxy_get("https://airma.sh/auth_twitter_cb"))
			})
			.resource("/auth_reddit_cb", |r| {
				r.method(Method::GET)
					.f(proxy_get("https://airma.sh/auth_reddit_cb"))
			})
			.resource("/auth_twitch_cb", |r| {
				r.method(Method::GET)
					.f(proxy_get("https://airma.sh/auth_twitch_cb"))
			})
			.resource("/auth_facebook", |r| {
				r.method(Method::GET)
					.f(proxy_redirect("https://airma.sh/auth_facebook"))
			})
			.resource("/auth_google", |r| {
				r.method(Method::GET)
					.f(proxy_redirect("https://airma.sh/auth_google"))
			})
			.resource("/auth_twitter", |r| {
				r.method(Method::GET)
					.f(proxy_redirect("https://airma.sh/auth_twitter"))
			})
			.resource("/auth_reddit", |r| {
				r.method(Method::GET)
					.f(proxy_redirect("https://airma.sh/auth_reddit"))
			})
			.resource("/auth_twitch", |r| {
				r.method(Method::GET)
					.f(proxy_redirect("https://airma.sh/auth_twitch"))
			})
	}).bind("0.0.0.0:9000")
		.unwrap()
		.shutdown_timeout(1)
		.run();
}
