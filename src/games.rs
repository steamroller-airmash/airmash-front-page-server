use hyper;
use serde_json;

use hyper::client::HttpConnector;
use hyper::{Body, Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;

use futures::future::join_all;
use futures::{Future, Stream};

use actix_web::{http, Error, HttpRequest, HttpResponse};

use spec::*;
use CONFIG;

/// Get the player count from a specific server.
/// Note that the future returned from this can
/// never fail (hence `Error = !`) since null
/// cases will be handled by returning an option.
fn fetch_server_players(
	client: &Client<HttpsConnector<HttpConnector>>,
	url: Uri,
) -> impl Future<Item = Option<u32>, Error = !> {
	client
		.get(url.clone())
		.map_err({
			let url = url.clone();
			move |e| {
				warn!(
					"Unable to connect to {} to fetch player count. Error description: {}",
					url, e
				);
			}
		})
		.and_then({
			let url = url.clone();
			move |response| {
				if response.status() != StatusCode::OK {
					warn!(
						"{} responded with non-200 status {}",
						url,
						response.status()
					);
					return Err(());
				}

				Ok(response)
			}
		})
		.and_then({
			let url = url.clone();
			move |response| {
				response
					.into_body()
					.fold(vec![], |mut acc, chunk| -> Result<Vec<_>, hyper::Error> {
						acc.extend_from_slice(&*chunk);
						Ok(acc)
					})
					.map_err(move |e| {
						warn!("Error occurred during request to {}: {}", url, e);
					})
			}
		})
		.and_then(move |v: Vec<u8>| {
			serde_json::from_slice(&v).map_err(|e| {
				warn!("Server {} sent invalid JSON, causing error: {}", url, e);
			})
		})
		.map(|v: ServerResponse| Some(v.players))
		.or_else(|_| Ok(None))
}

/// Get the data from the official airmash server.
/// The future from this can never fail, 
/// (hence `Error = !`) since all error cases
/// are handled as part of control flow.
fn fetch_official_server_info(
	client: &Client<HttpsConnector<HttpConnector>>,
) -> impl Future<Item = Vec<RegionSpec>, Error = !> {
	client
		.get("https://airma.sh/games".parse().unwrap())
		.map_err(|e| {
			warn!(
				"Unable to connect to https://airma.sh/games to fetch official server player counts. Error description: {}",
				e
			);
		})
		.and_then(|response| {
			if response.status() != StatusCode::OK {
				warn!(
					"{} responded with non-200 status {}",
					"https://airma.sh/games",
					response.status()
				);
				return Err(())
			}

			Ok(response)
		})
		.and_then(|response| {
			response
				.into_body()
				.fold(vec![], |mut acc, chunk| -> Result<_, hyper::Error> {
					acc.extend_from_slice(&*chunk);
					Ok(acc)
				})
				.map_err(|e| {
					warn!(
						"Error occurred during request to {}: {}",
						"https://airma.sh/games",
						e
					);
				})
		})
		.and_then(|v: Vec<u8>| {
			serde_json::from_slice(&v).map_err(|e| {
				warn!(
					"Server {} sent invalid JSON, causing error: {}",
					"https://airma.sh/games",
					e
				);
			})
		})
		.and_then(|v: AltGameSpec| {
			serde_json::from_str(&v.data)
				.map(|spec: Vec<AltRegionSpec>| -> Vec<RegionSpec> {
					spec.into_iter().map(|x| x.into_normal()).collect()
				})
				.map_err(|e| {
					warn!(
						"Server {} sent invalid JSON, causing error: {}",
						"https://airma.sh/games",
						e
					);
				})
		})
		.or_else(|_|{
			Ok(vec![])
		})
}

/// Make an http request to all gameservers
/// to query the number of players online.
pub fn games(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
	let https = HttpsConnector::new(4).expect("Failed to create HttpsConnector");
	let client: Client<_, Body> = Client::builder().build(https);

	let headers = req.request().headers();
	let country = headers
		.get("CF-IPCountry")
		.map(|x| x.to_str().unwrap_or("XX"))
		.unwrap_or("XX")
		.to_owned();
		
	let official_regions = fetch_official_server_info(&client);

	let external_regions = CONFIG.data.iter().cloned().map(move |region| {
		let requests = region
			.games
			.iter()
			.filter_map(|server| server.url.parse().ok())
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
					..game
				})
				.collect();

			RegionSpec { games, ..region }
		})
	});

	let external_regions = join_all(external_regions);

	let regions = external_regions.join(official_regions)
		.map(|(mut a, mut b)| {
			a.append(&mut b);
			a
		});

	Box::new(
		regions
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
			.map_err(|e| e),
	)
}
