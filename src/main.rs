extern crate actix;
extern crate actix_web;
#[macro_use]
extern crate serde;
extern crate env_logger;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate serde_json;
extern crate serde_urlencoded;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate sentry;
extern crate sentry_actix;

mod clienterror;
mod games;
mod login;
mod spec;

use actix_web::{http, middleware, server, App, HttpRequest, HttpResponse};
use http::Method;

use sentry_actix::SentryMiddleware;

use std::env;
use std::str;

use games::games;
// use login::*;
use spec::*;

const CONFIG_FILE: &'static str = include_str!("../config.json");

lazy_static! {
	static ref CONFIG: GameSpec = serde_json::from_str(CONFIG_FILE).unwrap();
}

fn ping(_: &HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.header("Content-Type", "application/json; charset=utf-8")
		.body("{\"pong\":1}")
}

fn enter(_: &HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.header("Content-Type", "application/json; charset=utf-8")
		.body("{\"result\":0}")
}

fn gone(_: &HttpRequest) -> HttpResponse {
	HttpResponse::build(http::StatusCode::GONE)
		.finish()
}

/// NOTE: Also initializes env_logger
fn init_sentry() -> Option<sentry::internals::ClientInitGuard> {
	if let Ok(dsn) = env::var("SENTRY_DSN") {
		let guard = sentry::init(&*dsn);

		sentry::integrations::env_logger::init(None, Default::default());
		sentry::integrations::panic::register_panic_handler();

		Some(guard)
	} else {
		env_logger::init();

		None
	}
}

fn main() {
	std::env::set_var("RUST_LOG", "info");
	std::env::set_var("RUST_BACKTRACE", "1");
	let _guard = init_sentry();

	server::new(move || {
		App::new()
			.middleware(middleware::Logger::default())
			.middleware(SentryMiddleware::new())
			.resource("/games", |r| r.method(Method::GET).f(games))
			.resource("/clienterror", |r| {
				r.method(Method::POST).f(clienterror::clienterror)
			})
			.resource("/ping",  |r| r.method(Method::GET).f(ping))
			.resource("/enter", |r| r.method(Method::POST).f(enter))
			.resource("/login", |r| r.method(Method::POST).f(gone))
			.resource("/auth",  |r| r.method(Method::POST).f(gone))
			// .resource("/login", |r| {
			// 	r.method(Method::POST).f(redirect("https://airma.sh/login"))
			// })
			// .resource("/auth", |r| {
			// 	r.method(Method::POST)
			// 		.f(proxy_post("https://airma.sh/auth"))
			// })
			// .resource("/auth2", |r| {
			// 	r.method(Method::POST)
			// 		.f(proxy_post("https://airma.sh/auth"))
			// })
			// .resource("/auth_facebook_cb", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_get("https://airma.sh/auth_facebook_cb"))
			// })
			// .resource("/auth_google_cb", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_get("https://airma.sh/auth_google_cb"))
			// })
			// .resource("/auth_twitter_cb", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_get("https://airma.sh/auth_twitter_cb"))
			// })
			// .resource("/auth_reddit_cb", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_get("https://airma.sh/auth_reddit_cb"))
			// })
			// .resource("/auth_twitch_cb", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_get("https://airma.sh/auth_twitch_cb"))
			// })
			// .resource("/auth_facebook", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_redirect("https://airma.sh/auth_facebook"))
			// })
			// .resource("/auth_google", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_redirect("https://airma.sh/auth_google"))
			// })
			// .resource("/auth_twitter", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_redirect("https://airma.sh/auth_twitter"))
			// })
			// .resource("/auth_reddit", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_redirect("https://airma.sh/auth_reddit"))
			// })
			// .resource("/auth_twitch", |r| {
			// 	r.method(Method::GET)
			// 		.f(proxy_redirect("https://airma.sh/auth_twitch"))
			// })
	})
	.bind("0.0.0.0:9000")
	.unwrap()
	.shutdown_timeout(1)
	.run();
}
