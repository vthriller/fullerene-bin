use plotters::prelude::*;
use hsluv::hsluv_to_rgb;

use itertools::Itertools;
use std::ops::Range;

use std::collections::HashMap;

use reqwest::blocking::Client;
use chrono::prelude::*;
use chrono::Duration;
use serde::Deserialize;

fn iter_to_range<I: Iterator<Item = f64>>(i: I) -> Range<f64> {
	use itertools::MinMaxResult::*;
	match i.minmax() {
		NoElements => 0. .. 1.,
		OneElement(a) => a - 0.5 .. a + 0.5,
		MinMax(a, b) => a .. b,
	}
}

#[allow(non_camel_case_types, non_snake_case, dead_code)]
#[derive(Deserialize)]
#[serde(tag = "status")]
enum PromQueryRangeStatus {
	success { data: PromQueryRangeResult },
	error {
		errorType: String,
		error: String,
	},
}

#[allow(non_camel_case_types)]
#[derive(Deserialize)]
#[serde(tag = "resultType")]
enum PromQueryRangeResult {
	matrix { result: Vec<PromMetric> },
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(tag = "resultType")]
struct PromMetric {
	metric: HashMap<String, String>,
	values: Vec<(f64, String)>,
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
	let now = Utc::now();

	let client = Client::new();
	let resp = client.get("http://127.0.0.1:9090/api/v1/query_range")
		.query(&[
			("query", "sum(rate(node_cpu{instance=\"localhost:9100\"} [5m])) by (mode)"),
			("start", &format!("{}", (now - Duration::hours(1)).timestamp())),
			("end", &format!("{}", now.timestamp())),
			("step", "5"),
		])
		.send()?
        .json::<PromQueryRangeStatus>()?;

	let data = match resp {
		PromQueryRangeStatus::success { data } => match data {
			PromQueryRangeResult::matrix { result } => result,
		},
		PromQueryRangeStatus::error { error, .. } => {
			println!("error: {}", error);
			return Ok(());
		},
	};

	// TODO labels
	let data: Vec<Vec<(f64, f64)>> = data.into_iter()
		.map(|metric| {
			metric.values.into_iter()
				// XXX unwrap(): we expect valid floats in strings (including "NaN"s)
				.map(|(k, v)| (k, v.parse().unwrap()))
				.collect()
		})
		.collect();

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
