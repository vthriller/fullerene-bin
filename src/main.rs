#[macro_use] extern crate quick_error;
mod prom;
mod render;

use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

type Error = Box<dyn std::error::Error + Sync + Send>;

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Error> {
	let data = prom::fetch()?;
	let img = render::render(data)?;

	let resp = Response::builder()
		.status(200)
		.header("Content-Type", "image/png")
		.body(Body::from(img))?;

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
