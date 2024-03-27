use serde::Serialize;

#[derive(Default, Serialize)]
pub struct GraphiaInput {
    pub graph: GraphiaInputGraph,
}

#[derive(Serialize)]
pub struct GraphiaInputGraph {
    pub directed: bool,
    pub edges: Vec<GraphiaInputEdge>,
    pub nodes: Vec<GraphiaInputNode>,
}

impl Default for GraphiaInputGraph {
    fn default() -> Self {
        Self {
            directed: true,
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }
}

#[derive(Serialize)]
pub struct GraphiaInputEdge {
    pub id: String,
    // metadata: GraphiaInputEdgeMetaData,
    pub source: String,
    pub target: String,
}

// #[derive(Serialize)]
// struct GraphiaInputEdgeMetaData {}

#[derive(Serialize)]
pub struct GraphiaInputNode {
    pub id: String,
    pub metadata: GraphiaInputNodeMetaData,
}

#[derive(Serialize)]
pub struct GraphiaInputNodeMetaData {
    pub signature: String,
}
