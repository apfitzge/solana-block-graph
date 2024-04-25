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
    pub source: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct GraphiaInputNode {
    pub id: String,
    pub metadata: GraphiaInputNodeMetaData,
}

#[derive(Serialize)]
pub struct GraphiaInputNodeMetaData {
    pub signature: String,
    pub num_signatures: usize,
    pub fee: u64,
    pub compute: u64,
    pub depth: usize,
}
