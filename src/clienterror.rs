

use std::sync::Arc;
use std::collections::HashMap;

use serde_urlencoded;

use actix_web::{HttpRequest, HttpResponse, HttpMessage, Error};
use actix_web::error::*;
use futures::Future;

use sentry::{Hub, Level};
use sentry_actix::ActixWebHubExt;

/// Log the client error to sentry for investigation
/// later. If a sentry dsn is not provided in the
/// SENTRY_DSN this is a no-op
pub fn clienterror(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
	let hub: Arc<Hub> = Hub::from_request(req);

	let res = req.body()
		.map_err(Into::into)
		.and_then(move |body| {
            let map: HashMap<&str, &str> = serde_urlencoded::from_bytes(&*body)
                .map_err(|_| ErrorBadRequest("Request data contained form data"))?;

            hub.configure_scope(move |scope| {
                for (k, v) in map {
                    scope.set_extra(k, v.to_owned().into());
                }
            });

			hub.capture_message(
				&*format!("An unhandled client error occurred"), 
				Level::Info
			);

			Ok(HttpResponse::Ok().body(""))
		});

	Box::new(res)
}
