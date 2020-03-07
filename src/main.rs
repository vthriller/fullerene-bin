use plotters::prelude::*;
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

	for metric in data {
	let data: Vec<(f64, f64)> = metric.values.into_iter()
		// XXX unwrap(): we expect valid floats in strings (including "NaN"s)
		.map(|(k, v)| (k, v.parse().unwrap()))
		.collect();

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

	return Ok(())
	}

	// XXX no time series
	Ok(())
}
