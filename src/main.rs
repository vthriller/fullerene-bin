#[macro_use] extern crate quick_error;
mod prom;
mod render;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let data = prom::fetch()?;
	let _ = render::render(data);
	Ok(())
}
