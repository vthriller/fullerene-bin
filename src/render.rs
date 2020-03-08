use plotters::prelude::*;
use hsluv::hsluv_to_rgb;

use itertools::Itertools;
use std::ops::Range;
use chrono::prelude::*;

fn iter_to_range<T, E, I>(elems: I, epsilon: E, empty: Range<T>) -> Range<T>
where
	T: PartialOrd + std::ops::Sub<E, Output = T> + std::ops::Add<E, Output = T> + Copy,
	E: Copy,
	I: Iterator<Item = T>,
{
	use itertools::MinMaxResult::*;
	match elems.minmax() {
		NoElements => empty,
		OneElement(a) => a - epsilon .. a + epsilon,
		MinMax(a, b) => a .. b,
	}
}

// Generates colors using perceptually uniform color space (HSLuv in this case)
fn colors() -> impl Iterator<Item = RGBColor> {
	vec![0., 20., 40.].into_iter().map(|hue_offset| {
		vec![65., 50., 80.].into_iter().map(move |lightness| {
			(0..5).map(move |hue| {
				let (r, g, b) = hsluv_to_rgb((
					(hue*60) as f64 + hue_offset,
					80.,
					lightness,
				));
				RGBColor(
					(r * 255.) as u8,
					(g * 255.) as u8,
					(b * 255.) as u8,
				)
			})
		})
		.flatten()
	})
	.flatten()
	.cycle()
}

fn date_format(range: &Range<DateTime<Utc>>, width: u32) -> String {
	#[derive(Clone, Copy, PartialEq, PartialOrd)]
	enum Unit { S, Mi, H, D, Mo, Y }
	use Unit::*;

	let mut wide = match (range.start, range.end) {
		// do we span multiple ${time_period}s?
		(a, b) if a.year()   != b.year()   => Y,
		(a, b) if a.month()  != b.month()  => Mo,
		(a, b) if a.day()    != b.day()    => D,
		(a, b) if a.hour()   != b.hour()   => H,
		(a, b) if a.minute() != b.minute() => Mi,
		(a, b) if a.second() != b.second() => S,
		(_, _) => unimplemented!(), // sub-second?!
	};
	// how much time passes between two adjacent pixels?
	let pitch = (range.end - range.start).num_seconds() / width as i64;
	let mut narrow = match pitch {
		p if p >= 60 * 60 * 24 => D,
		p if p >= 60 * 60 => H,
		p if p >= 60 => Mi,
		_ => S,
	};

	// "31 23:59" makes little sense, expand to "12-31 23:59"
	if wide == D { wide = Mo; }
	// ditto, "12-31 23" → "12-31 23:59"
	if narrow == H { narrow = Mi; }

	// XXX what if range covers, say, multiple hours (less than a day), but pitch is measured in days?
	// is that even possible?
	let mut fmt: Vec<_> = vec![Y, Mo, D, H, Mi, S].iter().filter_map(|&u| {
		if u > wide { return None }
		if u < narrow { return None }
		Some(match u {
			Y  => vec!["%Y", "-"],
			Mo => vec!["%m", "-"],
			D  => vec!["%d", " "],
			H  => vec!["%H", ":"],
			Mi => vec!["%M", ":"],
			S  => vec!["%S", "."],
		})
	}).flatten().collect();
	fmt.pop();

	fmt.join("")
}

pub fn render(data: Vec<Vec<(DateTime<Utc>, f64)>>, date_range: Range<DateTime<Utc>>, w: u32, h: u32) -> Result<Vec<u8>, DrawingAreaErrorKind<impl std::error::Error>> {
	let mut buf = vec![0; (w * h * 3) as usize];
	{
		let root = BitMapBackend::with_buffer(&mut buf, (w, h)).into_drawing_area();
		root.fill(&WHITE)?;

		let xfmt = date_format(&date_range, w);

		let mut chart = ChartBuilder::on(&root)
			.set_label_area_size(LabelAreaPosition::Left, 40)
			.set_label_area_size(LabelAreaPosition::Bottom, 30)
			.build_ranged(
				date_range,
				iter_to_range(data.iter().flatten().map(|(_, y)| *y), 0.5, 0. .. 1.),
			)?;

		chart.configure_mesh()
			.x_label_formatter(&|x: &DateTime<Utc>| x.format(&xfmt).to_string())
			.draw()?;

		for (data, color) in data.into_iter().zip(colors()) {
			chart.draw_series(LineSeries::new(data, &color))?;
		}
	}

	Ok(buf)
}
