#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod clienterror;
mod games;
mod spec;

use actix_web::{http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use futures::prelude::*;

use std::str;

use games::games;
use spec::*;

const CONFIG_FILE: &'static str = include_str!("../config.json");

lazy_static! {
	static ref CONFIG: GameSpec = serde_json::from_str(CONFIG_FILE).unwrap();
}

fn ping(_: HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.header("Content-Type", "application/json; charset=utf-8")
		.body("{\"pong\":1}")
}

fn enter(_: HttpRequest) -> HttpResponse {
	HttpResponse::Ok()
		.header("Content-Type", "application/json; charset=utf-8")
		.body("{\"result\":0}")
}

fn gone(_: HttpRequest) -> HttpResponse {
	HttpResponse::build(http::StatusCode::GONE).finish()
}

/// NOTE: Also initializes env_logger
fn init_sentry() -> Option<()> {
	// Option<sentry::internals::ClientInitGuard> {
	// if let Ok(dsn) = env::var("SENTRY_DSN") {
	// 	let guard = sentry::init(&*dsn);

	// 	sentry::integrations::env_logger::init(None, Default::default());
	// 	sentry::integrations::panic::register_panic_handler();

	// 	Some(guard)
	// } else {
	env_logger::init();

	None
	// }
}

fn games_wrapper(
	req: HttpRequest,
) -> Box<dyn hyper::rt::Future<Item = HttpResponse, Error = Error>> {
	Box::new(games(req).boxed_local().compat())
}
fn clienterror_wrapper(
	req: HttpRequest,
) -> impl hyper::rt::Future<Item = HttpResponse, Error = Error> {
	clienterror::clienterror(req).boxed_local().compat()
}

fn main() {
	std::env::set_var("RUST_LOG", "info");
	std::env::set_var("RUST_BACKTRACE", "1");
	let _guard = init_sentry();

	let appfn = move || {
		App::new()
			.wrap(middleware::Logger::default())
			.service(web::resource("/games").route(web::get().to_async(games_wrapper)))
			.service(web::resource("/clienterror").route(web::get().to_async(clienterror_wrapper)))
			.service(web::resource("/ping").route(web::get().to(ping)))
			.service(web::resource("/enter").route(web::post().to(enter)))
			.service(web::resource("/login").route(web::post().to(gone)))
			.service(web::resource("/auth").route(web::post().to(gone)))
	};

	HttpServer::new(appfn)
		.bind("0.0.0.0:9000")
		.unwrap()
		.shutdown_timeout(1)
		.run()
		.unwrap();
}
