
use serde_json;
use hyper;

use hyper::{Body, Client, Uri};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

use futures::future::join_all;
use futures::{Future, Stream};

use actix_web::{http, Error, HttpRequest, HttpResponse};

use spec::*;
use CONFIG;

fn fetch_server_players(
    client: &Client<HttpsConnector<HttpConnector>>, 
    url: Uri
) -> impl Future<Item = Option<u32>, Error = !>
{
    client
        .get(url.clone())
        .and_then(|response| {
            response.into_body().fold(
                vec![],
                |mut acc, chunk| -> Result<Vec<_>, hyper::Error> {
                    acc.extend_from_slice(&*chunk);
                    Ok(acc)
                },
            )
        })
        .or_else({
            let url = url.clone();
            move |e| {
                warn!("Unable to connect to {} to fetch player count", url);
                Err(e)
            }
        })
        .map(move |v: Vec<u8>| {
            serde_json::from_slice(&v)
                .map_err(|e| {
                    warn!("Server {} sent invalid JSON, causing error: {}", url, e);
                    ()
                })
                .ok()
        })
        .map(|v: Option<ServerResponse>| v.map(|x| x.players))
        .or_else(|_| Ok(None))
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

    let regions = CONFIG.data.iter()
        .cloned()
        .map(move |region| {
            let requests = region.games
                .iter()
                .filter_map(|server| server.url.parse().ok())
                .map(|server| fetch_server_players(&client, server))
                .collect::<Vec<_>>();

            join_all(requests)
                .map(move |counts| {
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
            .map_err(|e| e),
	)
}
