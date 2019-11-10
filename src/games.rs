use hyper;
use serde_json;

use hyper::client::HttpConnector;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;

use futures::future::join_all;
use futures::FutureExt;
use futures::compat::Compat01As03;

use actix_web::{http, Error, HttpRequest, HttpResponse};

use crate::spec::*;
use crate::CONFIG;

// Of course it's never used, that's the whole point
// TODO: Replace with never type once that stabilizes
#[allow(dead_code)]
enum Never {}

lazy_static! {
	static ref CLIENT: Client<HttpsConnector<HttpConnector>> = {
		let https = HttpsConnector::new(4).expect("Failed to create HttpsConnector");
		Client::builder().build(https)
	};
}


/// Get the player count from a specific server.
/// Note that the future returned from this can
/// never fail (hence `Error = Never`) since null
/// cases will be handled by returning an option.
async fn fetch_server_players(
	client: &Client<HttpsConnector<HttpConnector>>,
	url: Uri,
) -> Option<u32> {
	use hyper::rt::Stream as _;

	let response = match Compat01As03::new(client.get(url.clone())).await {
		Ok(res) => res,
		Err(e) => {
			warn!(
				"Unable to connect to {} to fetch player count. Error description: {}",
				url, e
			);

			return None;
		}
	};

	let fut = response
		.into_body()
		.fold(vec![], |mut acc, chunk| -> Result<Vec<_>, hyper::Error> {
			acc.extend_from_slice(&*chunk);
			Ok(acc)
		});
	
	let bytes: Vec<u8> = match Compat01As03::new(fut).await {
		Ok(bytes) => bytes,
		Err(e) => {
			warn!("Error occurred during request to {}: {}", url, e);
			return None;
		}
	};

	let res = match serde_json::from_slice(&bytes) {
		Ok(res) => res,
		Err(e) => {
			warn!("Server {} sent invalid JSON, causing error: {}", url, e);
			return None;
		}
	};
	
	Some(res)
}

/// Make an http request to all gameservers
/// to query the number of players online.
pub async fn games(req: HttpRequest) -> Result<HttpResponse, Error> {
	let client = &*CLIENT;

	let headers = req.headers();
	let country = headers
		.get("CF-IPCountry")
		.map(|x| x.to_str().unwrap_or("XX"))
		.unwrap_or("XX")
		.to_owned();

	let external_regions = CONFIG.data.iter().cloned().map(move |region| {
		let requests = region
			.games
			.iter()
			.filter_map(|server| ("https://".to_owned() + &server.url()).parse().ok())
			.map(|server| fetch_server_players(&client, server))
			.collect::<Vec<_>>();

		join_all(requests).map(move |counts| {
			let games = region
				.games
				.iter()
				.cloned()
				.zip(counts.into_iter())
				.map(|(game, count)| ServerSpec {
					players: count,
					url: "wss://".to_owned() + &game.url(),
					..game
				})
				.collect();

			RegionSpec { games, ..region }
		})
	});

	let regions = join_all(external_regions).await;
	let spec = GameSpec {
		protocol: 5,
		country,
		data: regions
	};

	let resp = serde_json::to_string(&spec)
		.expect("Failed to serialize the response");

	Ok(
		HttpResponse::Ok()
			.header(
				http::header::CONTENT_TYPE,
				"application/json; charset=utf-8"
			)
			.body(resp)
	)
}
