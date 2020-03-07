#[macro_use] extern crate quick_error;
mod prom;

use plotters::prelude::*;
use hsluv::hsluv_to_rgb;

use itertools::Itertools;
use std::ops::Range;

fn iter_to_range<I: Iterator<Item = f64>>(i: I) -> Range<f64> {
	use itertools::MinMaxResult::*;
	match i.minmax() {
		NoElements => 0. .. 1.,
		OneElement(a) => a - 0.5 .. a + 0.5,
		MinMax(a, b) => a .. b,
	}
}

// Generates colors using perceptually uniform color space (HSLuv in this case)
fn colors() -> impl Iterator<Item = RGBColor> {
	(0..5).map(|i| {
		let (r, g, b) = hsluv_to_rgb(((i*60) as f64, 80., 70.));
		RGBColor(
			(r * 255.) as u8,
			(g * 255.) as u8,
			(b * 255.) as u8,
		)
	})
	.cycle()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let data = prom::fetch()?;

	let root = BitMapBackend::new("test.png", (800, 480)).into_drawing_area();
	root.fill(&WHITE)?;

	let mut chart = ChartBuilder::on(&root)
		.set_label_area_size(LabelAreaPosition::Left, 40)
		.set_label_area_size(LabelAreaPosition::Bottom, 30)
		.build_ranged(
			iter_to_range(data.iter().flatten().map(|(x, _)| *x)),
			iter_to_range(data.iter().flatten().map(|(_, y)| *y)),
		)?;

	chart.configure_mesh().draw()?;

	for (data, color) in data.into_iter().zip(colors()) {
		chart.draw_series(LineSeries::new(data, &color))?;
	}

	Ok(())
}
