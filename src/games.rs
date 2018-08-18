
use serde_json;
use hyper;

use hyper::{Body, Client};
use hyper_tls::HttpsConnector;

use futures::future::join_all;
use futures::{Future, Stream};

use actix_web::error::InternalError;
use actix_web::http::StatusCode;
use actix_web::{http, Error, HttpRequest, HttpResponse};

use spec::*;
use CONFIG;

/// Make an http request to all gameservers
/// to query the number of players online.
pub fn games(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
	let https = HttpsConnector::new(4).unwrap();
	let client: Client<_, Body> = Client::builder().build(https);

	let headers = req.request().headers();
	let country = headers
		.get("CF-IPCountry")
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
					.map(|v: Vec<u8>| serde_json::from_slice(&v).unwrap())
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
					.header(
						http::header::CONTENT_TYPE,
						"application/json; charset=utf-8",
					)
					.body(resp)
			})
			.map_err(|e| e.into()),
	)
}
