#[macro_use] extern crate quick_error;
mod prom;
mod render;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	render::render()
	Ok(())
}
