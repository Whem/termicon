//! Routing Graph View
//! 
//! Visual representation of data paths:
//! - Multi-hop pipelines
//! - Transformers/filters between transports
//! - Graph-based routing configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node in the routing graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingNode {
    /// Unique node ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Position for visualization (x, y)
    pub position: (f32, f32),
    /// Node configuration
    pub config: HashMap<String, String>,
    /// Is this node active?
    pub active: bool,
}

/// Types of routing nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Input source (Serial, TCP, etc.)
    Source { transport_type: String },
    /// Output destination
    Sink { transport_type: String },
    /// Transform/filter node
    Transform { transform_type: TransformType },
    /// Splitter - one input, multiple outputs
    Splitter,
    /// Merger - multiple inputs, one output
    Merger,
    /// Protocol decoder
    Decoder { protocol: String },
    /// Protocol encoder
    Encoder { protocol: String },
    /// Logger/tap point
    Logger { log_path: String },
    /// Script processor
    Script { script_path: String },
}

/// Transform types for data processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransformType {
    /// Pass through unchanged
    Passthrough,
    /// Add prefix/suffix
    Wrapper { prefix: Vec<u8>, suffix: Vec<u8> },
    /// Strip bytes
    Strip { head: usize, tail: usize },
    /// Replace bytes
    Replace { from: Vec<u8>, to: Vec<u8> },
    /// Filter - only pass matching packets
    Filter { pattern: Vec<u8> },
    /// Rate limiter
    RateLimiter { bytes_per_second: u64 },
    /// Delay
    Delay { delay_ms: u64 },
    /// Checksum calculator
    Checksum { algorithm: String },
    /// Hex encode/decode
    HexCodec { encode: bool },
    /// Base64 encode/decode
    Base64Codec { encode: bool },
    /// Custom script
    Custom { script: String },
}

/// Edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingEdge {
    /// Source node ID
    pub from: String,
    /// Destination node ID  
    pub to: String,
    /// Edge label
    pub label: String,
    /// Edge is active
    pub active: bool,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Packets transferred
    pub packets_transferred: u64,
}

/// Complete routing graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingGraph {
    /// Graph name
    pub name: String,
    /// Description
    pub description: String,
    /// All nodes
    pub nodes: HashMap<String, RoutingNode>,
    /// All edges
    pub edges: Vec<RoutingEdge>,
    /// Is the graph running?
    pub running: bool,
}

impl Default for RoutingGraph {
    fn default() -> Self {
        Self {
            name: "New Graph".to_string(),
            description: String::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            running: false,
        }
    }
}

impl RoutingGraph {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: RoutingNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Remove a node and its edges
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    /// Add an edge
    pub fn add_edge(&mut self, from: &str, to: &str, label: &str) {
        self.edges.push(RoutingEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            active: true,
            bytes_transferred: 0,
            packets_transferred: 0,
        });
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, from: &str, to: &str) {
        self.edges.retain(|e| e.from != from || e.to != to);
    }

    /// Get all nodes connected to a node
    pub fn get_connected(&self, node_id: &str) -> Vec<&RoutingNode> {
        let connected_ids: Vec<_> = self.edges.iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to.as_str())
            .collect();
        
        self.nodes.values()
            .filter(|n| connected_ids.contains(&n.id.as_str()))
            .collect()
    }

    /// Get data path from source to sink
    pub fn get_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        // Simple BFS
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        visited.insert(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                // Reconstruct path
                let mut path = vec![to.to_string()];
                let mut node = to.to_string();
                while let Some(p) = parent.get(&node) {
                    path.push(p.clone());
                    node = p.clone();
                }
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.from == current && !visited.contains(&edge.to) {
                    visited.insert(edge.to.clone());
                    parent.insert(edge.to.clone(), current.clone());
                    queue.push_back(edge.to.clone());
                }
            }
        }

        None
    }

    /// Validate graph (check for cycles, disconnected nodes, etc.)
    pub fn validate(&self) -> Result<(), String> {
        // Check for cycles (would cause infinite loops)
        for node in self.nodes.keys() {
            if let Some(path) = self.get_path(node, node) {
                if path.len() > 1 {
                    return Err(format!("Cycle detected at node: {}", node));
                }
            }
        }

        // Check for orphan nodes
        for node_id in self.nodes.keys() {
            let has_incoming = self.edges.iter().any(|e| e.to == *node_id);
            let has_outgoing = self.edges.iter().any(|e| e.from == *node_id);
            
            let node = &self.nodes[node_id];
            match &node.node_type {
                NodeType::Source { .. } if !has_outgoing => {
                    return Err(format!("Source '{}' has no outputs", node_id));
                }
                NodeType::Sink { .. } if !has_incoming => {
                    return Err(format!("Sink '{}' has no inputs", node_id));
                }
                _ if !has_incoming && !has_outgoing => {
                    return Err(format!("Node '{}' is disconnected", node_id));
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Export graph to DOT format for visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph routing {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Nodes
        for (id, node) in &self.nodes {
            let shape = match &node.node_type {
                NodeType::Source { .. } => "ellipse",
                NodeType::Sink { .. } => "ellipse",
                NodeType::Splitter => "diamond",
                NodeType::Merger => "diamond",
                _ => "box",
            };
            let color = if node.active { "green" } else { "gray" };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n({:?})\" shape={} color={}];\n",
                id, node.name, node.node_type, shape, color
            ));
        }

        dot.push_str("\n");

        // Edges
        for edge in &self.edges {
            let style = if edge.active { "solid" } else { "dashed" };
            let label = if edge.bytes_transferred > 0 {
                format!("{} ({} bytes)", edge.label, edge.bytes_transferred)
            } else {
                edge.label.clone()
            };
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\" style={}];\n",
                edge.from, edge.to, label, style
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

/// Graph manager
#[derive(Debug, Default)]
pub struct GraphManager {
    /// All graphs
    pub graphs: HashMap<String, RoutingGraph>,
    /// Active graph ID
    pub active_graph: Option<String>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a simple serial-to-tcp bridge graph
    pub fn create_serial_tcp_bridge(&mut self, name: &str) -> &mut RoutingGraph {
        let mut graph = RoutingGraph::new(name);
        
        graph.add_node(RoutingNode {
            id: "serial_in".to_string(),
            name: "Serial Port".to_string(),
            node_type: NodeType::Source { transport_type: "serial".to_string() },
            position: (100.0, 200.0),
            config: HashMap::new(),
            active: true,
        });

        graph.add_node(RoutingNode {
            id: "tcp_out".to_string(),
            name: "TCP Server".to_string(),
            node_type: NodeType::Sink { transport_type: "tcp".to_string() },
            position: (400.0, 200.0),
            config: HashMap::new(),
            active: true,
        });

        graph.add_edge("serial_in", "tcp_out", "Raw Data");

        self.graphs.insert(name.to_string(), graph);
        self.graphs.get_mut(name).unwrap()
    }
}




