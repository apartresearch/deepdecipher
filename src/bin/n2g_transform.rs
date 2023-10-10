use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use deepdecipher::data::data_objects::Graph;
use graphviz_rust::{dot_structures::Graph as DotGraph, printer::PrinterContext};

#[derive(Parser, Debug)]
pub struct Config {
    directory: PathBuf,
    destination: Option<PathBuf>,
}

pub fn main() -> Result<()> {
    let Config {
        directory,
        destination,
    } = Config::parse();

    let dot_string = std::fs::read_to_string(directory)?;
    let dot_graph = graphviz_rust::parse(&dot_string).unwrap();

    let graph = Graph::from_dot(dot_graph)?;
    let mut graphviz_printer = PrinterContext::new(false, 4, "\n".to_string(), 30);
    let dot_graph_new: DotGraph = graph.into();
    let dot_graph_string = graphviz_rust::print(dot_graph_new, &mut graphviz_printer);
    if let Some(destination) = destination {
        fs::write(destination, dot_graph_string)?;
    } else {
        println!("{dot_graph_string}");
    }

    Ok(())
}
