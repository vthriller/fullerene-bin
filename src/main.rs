#[macro_use] extern crate quick_error;
mod prom;
mod render;

use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use chrono::prelude::*;
use chrono::Duration;

use serde::Deserialize;
use serde_qs as qs;

use image::{
	ColorType,
	png::PNGEncoder,
};

type Error = Box<dyn std::error::Error + Sync + Send>;

#[derive(Debug, Deserialize)]
struct QueryParams {
	w: Option<u32>,
	h: Option<u32>,
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Error> {
	let params: QueryParams = match qs::from_str(req.uri().query().unwrap_or("")) {
		Ok(params) => params,
		// this error is not Sync, hence this
		Err(_) => return Err("query string")?,
	};
	let w = params.w.unwrap_or(800);
	let h = params.h.unwrap_or(480);

	let now = Utc::now();
	let data = prom::fetch(
		now - Duration::hours(1),
		now,
	).await?;
	let img = render::render(
		data,
		now - Duration::hours(1) .. now,
		w, h,
	)?;

	let mut png = vec![];
	PNGEncoder::new(&mut png)
		.encode(
			&img, w, h,
			ColorType::RGB(8),
		)?;

	let resp = Response::builder()
		.status(200)
		.header("Content-Type", "image/png")
		.body(Body::from(png))?;

	Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = SocketAddr::from(([127, 0, 0, 1], 12345));

	let make_service = make_service_fn(|_conn| async {
		Ok::<_, Error>(service_fn(handle))
	});

	let server = Server::bind(&addr)
		.serve(make_service);

	Ok(server.await?)
}
