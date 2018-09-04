use std::convert::AsRef;

use actix_web::client::ClientRequest;
use actix_web::dev::HttpResponseBuilder;
use actix_web::error::*;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};

use futures::Future;

pub fn post_proxy<U>(
	uri: U,
) -> impl Fn(&HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>
where
	U: AsRef<str>,
{
	return move |req| {
		let mut new_req = ClientRequest::post(&uri);
		new_req.no_default_headers();
		for (name, value) in req.headers() {
			new_req.header(name.clone(), value.clone());
		}

		let fut = req
			.body()
			.map_err(|e| ErrorBadGateway(e))
			.and_then(move |body| new_req.body(body))
			.and_then(|new_req| new_req.send().map_err(|e| ErrorBadGateway(e)))
			.and_then(|res| {
				res.body().from_err().and_then(move |bytes| {
					let mut out_res: HttpResponseBuilder = (&res).into();

					Ok(out_res.body(bytes))
				})
			});

		Box::new(fut)
	};
}
