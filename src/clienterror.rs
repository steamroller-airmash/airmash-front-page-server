use actix_web::{Error, HttpRequest, HttpResponse};


/// This is currently a noop - maybe it can be implemented
/// at some point?
/// 
/// Would be useful for finding client errors in the wild
pub async fn clienterror(_: HttpRequest) -> Result<HttpResponse, Error> {
	// TODO: Maybe implement this?

	Ok(HttpResponse::Ok().body(""))
}
