use gaussian::gaussian_sample;
use serde::{de::Error, Serialize, Deserialize, Deserializer};
use chrono::NaiveDate;
use plotters::prelude::*;

use std::error::Error as sError;

mod gaussian;

#[derive(Serialize, Deserialize, Debug)]
struct DataEntry {
    #[serde(deserialize_with="from_str")]
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: u32,
}
fn from_str<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(s,"%Y-%m-%d").map_err(D::Error::custom)
}

fn read_from_file(path: &str) -> Result<Vec<DataEntry>, Box<dyn sError>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = Vec::new();
    for result in reader.deserialize() {
        match result {
            Ok(entry) =>  data.push(entry),
            Err(_) => (),
        }
    }
    Ok(data)
}

fn backtest(data: &Vec<DataEntry>, investment: f64) -> (Vec<f64>, Vec<f64>) {
    // strategy is to buy when yesterdays close > open
    //  and to sell when yesterdays close < open
    // assume that all interest is banked and same investment used at every occasion => 

    // Buying indicates we buy as many of the shares as possible with investment
    // Selling indicates we sell all shares we possess

    // choose that we buy and sell at open price based on last day

    let mut shares = (0.0,0.0); // fractional shares allowed, (# shares, price paid)
    let mut banked_profits = vec![0.0];

    let mut prefix_squares = vec![0.0];
    let mut variances = vec![0.0];

    for i in 1..data.len() {
        if data[i-1].close > data[i-1].open && shares.0 == 0.0 { // want to buy and have no shares
            shares = (investment / data[i].open, data[i].open);
            banked_profits.push(*banked_profits.last().unwrap());

            prefix_squares.push(*prefix_squares.last().unwrap());
        } else if data[i-1].close < data[i-1].open && shares.0 != 0.0 { // want to sell and have shares
            banked_profits.push(banked_profits.last().unwrap() + shares.0 * (data[i].open - shares.1));

            prefix_squares.push(*prefix_squares.last().unwrap() + (shares.0 * (data[i].open - shares.1)).powi(2) );
            shares = (0.0, 0.0);
        } else {
            banked_profits.push(*banked_profits.last().unwrap());

            prefix_squares.push(*prefix_squares.last().unwrap());
        }
        variances.push(prefix_squares[i] / (i-1) as f64);
    }
    return (banked_profits, variances);
}

fn backtest_random(data: &Vec<DataEntry>, investment: f64) -> (Vec<f64>, Vec<f64>) {
    // strategy is to do something when we flip heads
    // Buying indicates we buy as many of the shares as possible with investment
    // Selling indicates we sell all shares we possess

    // choose that we buy and sell at open price based on last day

    let mut shares = (0.0,0.0); // fractional shares allowed, (# shares, price paid)
    let mut banked_profits = vec![0.0];

    let mut prefix_squares = vec![0.0];
    let mut variances = vec![0.0];

    for i in 1..data.len() {
        let doing_something:bool = rand::random();
        if doing_something && shares.0 == 0.0 { // want to buy and have no shares
            shares = (investment / data[i].open, data[i].open);
            banked_profits.push(*banked_profits.last().unwrap());

            prefix_squares.push(*prefix_squares.last().unwrap());
        } else if doing_something && shares.0 != 0.0 { // want to sell and have shares
            banked_profits.push(banked_profits.last().unwrap() + shares.0 * (data[i].open - shares.1));

            prefix_squares.push(*prefix_squares.last().unwrap() + (shares.0 * (data[i].open - shares.1)).powi(2) );
            shares = (0.0, 0.0);
        } else {
            banked_profits.push(*banked_profits.last().unwrap());

            prefix_squares.push(*prefix_squares.last().unwrap());
        }
        variances.push(prefix_squares[i] / (i-1) as f64);
    }
    return (banked_profits, variances);
}

fn gen_price_series(length: usize, mean: f64, std_dev: f64) -> Vec<f64> {
    let mut ret = vec![0.0];
    for i in 1..length {
        ret.push(ret[i-1] + gaussian_sample(mean, std_dev));
    }
    ret
}

fn plot(title: &str,
        x_axis: std::iter::IntoIterator<i32>,
        y_axis: std::iter::IntoIterator<i32>,
        x: std::iter) -> Result<(),String> {
    let root_area = BitMapBackend::new("random.png", (600, 400))
    .into_drawing_area();
  root_area.fill(&WHITE).map_err(|e| e.to_string());

  let mut ctx = ChartBuilder::on(&root_area)
    .set_label_area_size(LabelAreaPosition::Left, 40)
    .set_label_area_size(LabelAreaPosition::Bottom, 40)
    .caption(title, ("sans-serif", 40))
    .build_cartesian_2d(0..1260, -200..200)
    .unwrap();

  ctx.configure_mesh().draw().map_err(|e| e.to_string());

  ctx.draw_series(
    LineSeries::new((0..=1257).map(|x| (x as i32, profits[x] as i32)), &BLUE)
  ).unwrap();
  Ok(())
}

fn main() {
    println!("Hello, world!");

    /*
    let data = read_from_file("AAPL.csv").unwrap();
    println!("{}", data.len());

    let root_area = BitMapBackend::new("random.png", (600, 400))
    .into_drawing_area();
  root_area.fill(&WHITE).unwrap();

  let mut ctx = ChartBuilder::on(&root_area)
    .set_label_area_size(LabelAreaPosition::Left, 40)
    .set_label_area_size(LabelAreaPosition::Bottom, 40)
    .caption("Random", ("sans-serif", 40))
    .build_cartesian_2d(0..1260, -200..200)
    .unwrap();

  ctx.configure_mesh().draw().unwrap();

  let (profits, variations) = backtest_random(&data, 1000.0);
  ctx.draw_series(
    LineSeries::new((0..=1257).map(|x| (x as i32, profits[x] as i32)), &BLUE)
  ).unwrap();
  println!("Sharpe ratio is a huge: {}", profits.last().unwrap() / variations.last().unwrap());
  */

  let (profits, variations) = backtest_random(&data, 1000.0);

  println!("{:?}",gen_price_series(10, 0.0, 1.0));
}


