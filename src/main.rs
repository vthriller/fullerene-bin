use plotters::prelude::*;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let data = vec![
		(1., 3.14159),
		(2., 2.71828),
		(3., -1.),
		(4., 0.),
	];

	let root = BitMapBackend::new("test.png", (800, 480)).into_drawing_area();
	root.fill(&WHITE)?;

	let mut chart = ChartBuilder::on(&root)
		.set_label_area_size(LabelAreaPosition::Left, 40)
		.set_label_area_size(LabelAreaPosition::Bottom, 30)
		.build_ranged(
			iter_to_range(data.iter().map(|(x, _)| *x)),
			iter_to_range(data.iter().map(|(_, y)| *y)),
		)?;

	chart.configure_mesh().draw()?;
	chart.draw_series(LineSeries::new(data, &RED))?;

	Ok(())
}
