use std::fs;

use neuronav::data::NeuroscopePage;
use scraper::Html;

const FLOAT_REGEX: &str = r"(-?\d+(?:\.\d*)?)";
pub fn main() {
    println!("{}", &format!(r"<h4>Max Range: <b>{FLOAT_REGEX}</b>."));
}
