use std::convert::AsRef;
use std::str;

use actix_web::client::{ClientRequest, ClientResponse};
use actix_web::dev::HttpResponseBuilder;
use actix_web::error::*;
use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse};

use futures::{lazy, Future};

pub fn proxy_redirect<U>(
	uri: U,
) -> impl Fn(&HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>
where
	U: AsRef<str>,
{
	move |req| {
		let mut new_req = ClientRequest::get(&format!(
			"{uri}?{query}",
			uri = uri.as_ref(),
			query = req.query_string()
		));
		new_req.no_default_headers();
		new_req.header("Host", "airma.sh");
		for (name, value) in req.headers() {
			if name == "Host" {
				continue;
			}
			new_req.header(name.clone(), value.clone());
		}

		let fut = req
			.body()
			.map_err(ErrorBadGateway)
			.and_then(move |body| new_req.body(body))
			.and_then(move |new_req| new_req.send().map_err(ErrorBadGateway))
			.and_then(|res| {
				res.body().from_err().and_then(move |bytes| {
					let mut out_res: HttpResponseBuilder = (&res).into();

					Ok(out_res.body(bytes))
				})
			});

		Box::new(fut)
	}
}

pub fn proxy_get<U>(
	uri: U,
) -> impl Fn(&HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>
where
	U: AsRef<str>,
{
	move |req| {
		let host = match req.headers().get("Host") {
			Some(h) => match h.to_str() {
				Ok(h) => h.to_owned(),
				_ => return Box::new(lazy(|| Err(ErrorBadRequest("Invalid Host header")))),
			},
			None => return Box::new(lazy(|| Err(ErrorForbidden("No Host header")))),
		};

		let mut new_req = ClientRequest::get(&format!(
			"{uri}?{query}",
			uri = uri.as_ref(),
			query = req.query_string()
		));
		new_req.no_default_headers();
		new_req.header("Host", "airma.sh");
		// TODO: Figure out which headers can be forwarded
		/*for (name, value) in req.headers() {
			if name == "Host" ||  {
				continue;
			}
			new_req.header(name.clone(), value.clone());
		}*/

		let fut = req
			.body()
			.map_err(ErrorBadGateway)
			.and_then(move |body| new_req.body(body))
			.and_then(move |new_req| new_req.send().map_err(ErrorBadGateway))
			.and_then(|res: ClientResponse| {
				res.body().from_err().and_then(move |bytes| {
					let mut out_res: HttpResponseBuilder = HttpResponse::build(res.status());
					let headers: &HeaderMap = res.headers();

					if let Some(v) = headers.get("Content-Type") {
						out_res.header("Content-Type", v.to_str().map_err(ErrorBadGateway)?);
					}
					if let Some(v) = headers.get("Content-Length") {
						out_res.header("Content-Length", v.to_str().map_err(ErrorBadGateway)?);
					}
					if let Some(v) = headers.get("Set-Cookie") {
						let s =
							str::replace(v.to_str().map_err(ErrorBadGateway)?, "airma.sh", &host);
						// Don't set cloudflare cookies, it causes CF to fail
						if !s.contains("__cfduid") {
							out_res.header("Set-Cookie", s);
						}
					}

					Ok(out_res.body(bytes))
				})
			});

		Box::new(fut)
	}
}

// Will try to get this to work later
#[allow(dead_code)]
pub fn proxy_post<U>(
	uri: U,
) -> impl Fn(&HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>
where
	U: AsRef<str>,
{
	return move |req| {
		let host = match req.headers().get("Host") {
			Some(h) => match h.to_str() {
				Ok(h) => h.to_owned(),
				_ => {
					error!("Invalid host header: {:?}", h);
					return Box::new(lazy(|| Err(ErrorBadRequest("Invalid Host header"))));
				}
			},
			None => return Box::new(lazy(|| Err(ErrorForbidden("No Host header")))),
		};

		let mut new_req = ClientRequest::post(&uri);
		new_req.no_default_headers();
		new_req.header("Host", "airma.sh");

		let fut = req
			.body()
			.map_err(ErrorBadGateway)
			.and_then(move |body| new_req.body(body))
			.and_then(|new_req| new_req.send().map_err(ErrorBadGateway))
			.and_then(|res| {
				res.body().from_err().and_then(move |bytes| {
					let mut out_res: HttpResponseBuilder = HttpResponse::build(res.status());
					let headers: &HeaderMap = res.headers();

					if let Some(v) = headers.get("Content-Type") {
						out_res.header("Content-Type", v.to_str().map_err(ErrorBadGateway)?);
					}
					if let Some(v) = headers.get("Content-Length") {
						out_res.header("Content-Length", v.to_str().map_err(ErrorBadGateway)?);
					}
					if let Some(v) = headers.get("Set-Cookie") {
						let s =
							str::replace(v.to_str().map_err(ErrorBadGateway)?, "airma.sh", &host);
						// Don't set cloudflare cookies, it causes CF to fail
						if !s.contains("__cfduid") {
							out_res.header("Set-Cookie", s);
						}
					}

					Ok(out_res.body(bytes))
				})
			})
			.map_err(|e| {
				error!("An internal request error occurred: {}", e);
				e
			});

		Box::new(fut)
	};
}

pub fn redirect<U>(uri: U) -> impl Fn(&HttpRequest) -> HttpResponse
where
	U: AsRef<str>,
{
	return move |_| {
		let uri: &str = uri.as_ref();
		HttpResponse::build(StatusCode::PERMANENT_REDIRECT)
			.header("Location", uri)
			.body("")
	};
}
