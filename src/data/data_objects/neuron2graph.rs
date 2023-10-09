use std::{collections::HashMap, iter};

use anyhow::{bail, Context, Result};
use graphviz_rust::{
    dot_generator::edge,
    dot_structures::{
        Attribute, Edge, EdgeTy, Graph as DotGraph, GraphAttributes, Id, Node as DotNode, NodeId,
        Stmt, Subgraph, Vertex,
    },
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::data::SimilarNeurons;

use super::{data_object, DataObject};

fn id_to_str(id: &Id) -> &str {
    match id {
        Id::Html(id) | Id::Escaped(id) | Id::Plain(id) | Id::Anonymous(id) => id,
    }
}

fn id_to_usize(id: &Id) -> Result<usize> {
    let id_string = id_to_str(id);
    id_string.parse::<usize>().with_context(|| format!(
        "Could not parse node id {} as usize. It is assumed that all N2G graphs only use positive integer node ids.", id_string
    ))
}

fn dot_node_to_id_label_importance(node: &DotNode) -> Result<(usize, String, f32)> {
    let DotNode {
        id: NodeId(id, _),
        attributes,
    } = node;
    let id = id_to_usize(id)?;
    let Attribute(_, label_id) = attributes
        .iter()
        .find(|Attribute(key, _)| id_to_str(key) == "label")
        .with_context(|| format!("Node with id {id} has no attribute 'label'."))?;
    // Assume that the `fillcolor` attribute is a 9 character string with '"' enclosing a hexadecimal color code.
    let color_str = get_attribute(attributes.as_slice(), "fillcolor").with_context(|| format!(
        "Node {id} has no attribute 'fillcolor'. It is assumed that all N2G nodes have a 'fillcolor' attribute that signifies their importance."
    ))?;
    let importance_hex = color_str.get(4..6).with_context(|| format!(
            "The 'fillcolor' attribute of node {id} is insufficiently long. It is expected to be 9 characters long."
    ))?;
    let importance = 1.-u8::from_str_radix(importance_hex, 16).with_context(|| format!(
        "The green part of the 'fillcolor' attribute of node {id} is not a valid hexadecimal number."
    ))? as f32 / 255.0;

    let label = id_to_str(label_id).to_string();
    Ok((id, label, importance))
}

fn get_attribute(attributes: &[Attribute], key: impl AsRef<str>) -> Option<&str> {
    attributes
        .as_ref()
        .iter()
        .find(|Attribute(attr_key, _)| id_to_str(attr_key) == key.as_ref())
        .map(|Attribute(_, value)| id_to_str(value))
}

fn subgraph_to_nodes(subgraph: &Subgraph) -> Result<Vec<(usize, String, f32)>> {
    let Subgraph {
        id,
        stmts: statements,
    } = subgraph;
    let id_str = id_to_str(id);
    let id: usize = id_str
        .strip_prefix("cluster_")
        .with_context(|| format!("It is assumed that all N2G subgraphs have ids starting with 'cluster_'. Subgraph id: {id_str}"))?
        .parse::<usize>()
        .with_context(|| format!("Failed to parse subgraph id '{id_str}' as usize."))?;
    let nodes = statements
        .iter()
        .filter_map(|statement| match statement {
            Stmt::Node(node) => Some(dot_node_to_id_label_importance(node)),
            _ => None,
        })
        .collect::<Result<Vec<_>>>()
        .with_context(|| format!("Failed to parse nodes for subgraph {id}."))?;
    Ok(nodes)
}

fn dot_edge_to_ids(
    Edge {
        ty: edge_ty,
        attributes,
    }: &Edge,
) -> Result<(usize, usize)> {
    match edge_ty {
        EdgeTy::Pair(Vertex::N(NodeId(node_id1, _)), Vertex::N(NodeId(node_id2, _))) => {
            let id1 = id_to_usize(node_id1).with_context(|| format!("Failed to parse first id for edge {edge_ty:?}."))?;
            let id2 = id_to_usize(node_id2).with_context(|| format!("Failed to parse second id for edge {edge_ty:?}."))?;
            match get_attribute(attributes, "dir") {
                Some("back") => Ok((id2, id1)),
                None => bail!("No direction attribute found for edge {id1}->{id2}. It is assumed that all N2G graphs only use edges with direction 'back'."),
                _ => bail!("Only edges with direction 'back' or 'forward' are supported. It is assumed that all N2G graphs only use edges with direction 'back' or 'forward'. Edge: {:?}", edge_ty)
            }
        }
        EdgeTy::Pair(_, _) => bail!("Only edges between individual nodes are supported. It is assumed that N2G does not use edges between subgraphs. Edge: {:?}", edge_ty),
        EdgeTy::Chain(_) => bail!("Only pair edges are supported. It is assumed that all N2G graphs only use pair edges. Edge: {:?}", edge_ty)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Node {
    token: String,
    required: Vec<usize>,
    importance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    subgraph_indices: Vec<usize>,
    graph: Vec<Node>,
}

impl Graph {
    pub fn from_dot(graph: DotGraph) -> Result<Self> {
        let statements = match graph {
            DotGraph::Graph {
                id: _,
                strict: _,
                stmts: _,
            } => bail!("Can only create N2G graph from a graphviz digraph."),
            DotGraph::DiGraph {
                id: _,
                strict: _,
                stmts,
            } => stmts,
        };
        let subgraphs = statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Subgraph(subgraph) => Some(subgraph_to_nodes(subgraph)),
                _ => None,
            })
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse subgraphs.")?;
        let subgraph_indices = subgraphs
            .iter()
            .scan(0, |index, nodes| {
                let index_old = *index;
                *index += nodes.len();
                Some(index_old)
            })
            .chain(iter::once(subgraphs.iter().map(|nodes| nodes.len()).sum()))
            .collect::<Vec<_>>();
        let id_to_index = subgraphs
            .iter()
            .flat_map(|nodes| nodes.iter())
            .enumerate()
            .map(|(index, (id, _, _))| (*id, index))
            .collect::<HashMap<_, _>>();
        let edges = statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Edge(edge) => Some(dot_edge_to_ids(edge)),
                _ => None,
            })
            .collect::<Result<Vec<_>>>()
            .context("Failed to parse edges.")?;
        let graph = subgraphs
            .into_iter()
            .flatten()
            .map(|(id, token, importance)| {
                // This requires O(n*m) time, where n is the number of nodes and m is the number of edges.
                // It can be improved to O(n+m) by using a hashmap.
                let required = edges
                    .iter()
                    .filter_map(|&(id1, id2)| {
                        if id2 == id {
                            Some(id_to_index[&id1])
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                Node {
                    token,
                    required,
                    importance,
                }
            });
        Ok(Graph {
            subgraph_indices,
            graph: graph.collect::<Vec<_>>(),
        })
    }
}

fn index_to_vertex(index: usize) -> Vertex {
    Vertex::N(NodeId(Id::Plain(format!("{index}")), None))
}

fn attribute(key: impl AsRef<str>, value: impl AsRef<str>) -> Attribute {
    let value = value.as_ref();
    let value_id = if value.contains('"') {
        Id::Escaped(value.to_string())
    } else {
        Id::Plain(value.to_string())
    };
    Attribute(Id::Plain(key.as_ref().to_string()), value_id)
}

fn index_pair_to_dot_edge(index1: usize, index2: usize) -> Edge {
    edge!(index_to_vertex(index1) => index_to_vertex(index2))
}

fn importance_to_color(importance: f32, activating: bool) -> String {
    let importance = ((1. - importance) * 255.0) as u8;
    if activating {
        format!("\"#ff{importance:02x}{importance:02x}\"")
    } else {
        format!("\"#{importance:02x}{importance:02x}ff\"")
    }
}

fn dot_subgraph<'a>(
    (subgraph_index, nodes): (usize, impl Iterator<Item = (usize, &'a Node)>),
) -> Subgraph {
    let node_statements_iter = nodes.map(|(node_index, node)| {
        let Node {
            token,
            required: _,
            importance,
        } = node;
        Stmt::Node(DotNode {
            id: NodeId(Id::Plain(format!("{node_index}")), None),
            attributes: vec![
                attribute("label", token),
                attribute(
                    "fillcolor",
                    importance_to_color(*importance, subgraph_index <= 1),
                ),
            ],
        })
    });
    Subgraph {
        id: Id::Plain(format!("cluster_{subgraph_index}")),
        stmts: node_statements_iter.collect(),
    }
}

impl From<Graph> for DotGraph {
    fn from(value: Graph) -> Self {
        let subgraphs = value
            .subgraph_indices
            .into_iter()
            .tuple_windows()
            .map(|(start_index, end_index)| {
                value.graph[start_index..end_index]
                    .iter()
                    .enumerate()
                    .map(move |(index, node)| (index + start_index, node))
            })
            .enumerate()
            .collect::<Vec<_>>();
        let subgraph_statement_iter = subgraphs.into_iter().map(dot_subgraph).map(Stmt::Subgraph);
        let edge_iter = value.graph.iter().enumerate().flat_map(|(index, node)| {
            node.required
                .iter()
                .map(move |&required_index| (required_index, index))
        });
        let edge_statement_iter =
            edge_iter.map(|(index1, index2)| Stmt::Edge(index_pair_to_dot_edge(index1, index2)));
        let graph_attributes = Stmt::GAttribute(GraphAttributes::Graph(vec![
            attribute("nodesep", "0.2"),
            attribute("rankdir", "LR"),
            attribute("ranksep", "1.5"),
            attribute("splines", "spline"),
            attribute("pencolor", "white"),
            attribute("penwidth", "3"),
        ]));
        // Add more node attributes. E.g. shape, style and maybe penwidth. This removes them from the individual nodes and saves complexity/space.
        // If global attributes can be changed halfway through, use this for fontcolor.
        let node_attributes = Stmt::GAttribute(GraphAttributes::Node(vec![
            attribute("fixedsize", "true"),
            attribute("height", "0.75"),
            attribute("width", "2"),
            attribute("style", "\"filled,solid\""),
            attribute("shape", "box"),
            attribute("fontcolor", "black"),
            attribute("fontsize", "25"),
            attribute("penwidth", "7"),
        ]));
        let edge_attributes =
            Stmt::GAttribute(GraphAttributes::Edge(vec![attribute("penwidth", "3")]));
        // Add global edge attributes.
        let statement_iter = [graph_attributes, node_attributes, edge_attributes]
            .into_iter()
            .chain(edge_statement_iter)
            .chain(subgraph_statement_iter);
        DotGraph::DiGraph {
            id: Id::Anonymous("".to_string()),
            strict: false,
            stmts: statement_iter.collect(),
        }
    }
}

impl DataObject for Graph {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuron2Graph graph")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuron2Graph graph")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Neuron2GraphData {
    pub graph: Graph,
    pub similar: SimilarNeurons,
}

impl DataObject for Neuron2GraphData {
    fn to_binary(&self) -> Result<Vec<u8>> {
        data_object::to_binary(self, "Neuron2Graph data")
    }

    fn from_binary(data: impl AsRef<[u8]>) -> Result<Self> {
        data_object::from_binary(data, "Neuron2Graph data")
    }
}
