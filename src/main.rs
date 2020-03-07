#[macro_use] extern crate quick_error;
mod prom;
mod render;

use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use chrono::prelude::*;
use chrono::Duration;

use image::{
	ColorType,
	png::PNGEncoder,
};

type Error = Box<dyn std::error::Error + Sync + Send>;

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Error> {
	let now = Utc::now();
	let data = prom::fetch(
		now - Duration::hours(1),
		now,
	).await?;
	let img = render::render(
		data,
		now - Duration::hours(1) .. now,
	)?;

	let mut png = vec![];
	PNGEncoder::new(&mut png)
		.encode(
			&img, 800, 480,
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
