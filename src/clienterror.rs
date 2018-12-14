use std::collections::HashMap;
use std::str;
use std::sync::Arc;

use serde_urlencoded;

use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};
use futures::Future;

use sentry::{Hub, Level};
use sentry_actix::ActixWebHubExt;

/// Log the client error to sentry for investigation
/// later. If a sentry dsn is not provided in the
/// SENTRY_DSN this is a no-op
pub fn clienterror(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
	let hub: Arc<Hub> = Hub::from_request(req);

	let res = req.body().map_err(Into::into).and_then(move |body| {
		let msg: &str = str::from_utf8(&*body).unwrap_or("");
		let map: HashMap<&str, &str> = serde_urlencoded::from_bytes(&*body).unwrap_or_default();

		hub.configure_scope(move |scope| {
			for (k, v) in map {
				scope.set_extra(k, v.to_owned().into());
			}
		});

		hub.capture_message(
			&*format!("An unhandled client error occurred:\n {}", msg),
			Level::Info,
		);

		Ok(HttpResponse::Ok().body(""))
	});

	Box::new(res)
}
