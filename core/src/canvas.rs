use serde_json::{Number, Value};
use std::{error::Error, fmt};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasDocument {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasNode {
    pub id: String,
    pub kind: CanvasNodeKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CanvasNodeKind {
    Text { text: String },
    File { file: String },
    Link { url: String },
    Group { label: Option<String> },
    Unknown { node_type: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasEdge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanvasParseError {
    message: String,
}

impl CanvasParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CanvasParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CanvasParseError {}

pub fn parse_canvas_document(input: &str) -> Result<CanvasDocument, CanvasParseError> {
    let input = input.trim_start_matches('\u{feff}');

    if input.trim().is_empty() {
        return Ok(CanvasDocument::default());
    }

    let root = serde_json::from_str::<Value>(input)
        .map_err(|error| CanvasParseError::new(format!("invalid canvas JSON: {error}")))?;
    let object = root
        .as_object()
        .ok_or_else(|| CanvasParseError::new("canvas root must be a JSON object"))?;

    let nodes = object
        .get("nodes")
        .and_then(Value::as_array)
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(parse_canvas_node)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let edges = object
        .get("edges")
        .and_then(Value::as_array)
        .map(|edges| {
            edges
                .iter()
                .filter_map(parse_canvas_edge)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(CanvasDocument { nodes, edges })
}

pub fn update_canvas_node_position(
    input: &str,
    node_id: &str,
    x: f32,
    y: f32,
) -> Result<String, CanvasParseError> {
    let input = input.trim_start_matches('\u{feff}');

    if input.trim().is_empty() {
        return Ok(input.to_owned());
    }

    let mut root = serde_json::from_str::<Value>(input)
        .map_err(|error| CanvasParseError::new(format!("invalid canvas JSON: {error}")))?;
    let object = root
        .as_object_mut()
        .ok_or_else(|| CanvasParseError::new("canvas root must be a JSON object"))?;
    let Some(nodes) = object.get_mut("nodes").and_then(Value::as_array_mut) else {
        return serde_json::to_string_pretty(&root).map_err(|error| {
            CanvasParseError::new(format!("could not write canvas JSON: {error}"))
        });
    };

    for node in nodes {
        let Some(node_object) = node.as_object_mut() else {
            continue;
        };
        if node_object.get("id").and_then(Value::as_str) != Some(node_id) {
            continue;
        }

        node_object.insert("x".to_owned(), json_number_from_f32(x)?);
        node_object.insert("y".to_owned(), json_number_from_f32(y)?);
        break;
    }

    serde_json::to_string_pretty(&root)
        .map_err(|error| CanvasParseError::new(format!("could not write canvas JSON: {error}")))
}

fn parse_canvas_node(value: &Value) -> Option<CanvasNode> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str()?.to_owned();
    let node_type = object.get("type")?.as_str()?.to_owned();
    let x = numeric_field(object.get("x")).unwrap_or(0.0);
    let y = numeric_field(object.get("y")).unwrap_or(0.0);
    let width = numeric_field(object.get("width")).unwrap_or(240.0);
    let height = numeric_field(object.get("height")).unwrap_or(120.0);
    let kind = match node_type.as_str() {
        "text" => CanvasNodeKind::Text {
            text: object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        },
        "file" => CanvasNodeKind::File {
            file: object
                .get("file")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        },
        "link" => CanvasNodeKind::Link {
            url: object
                .get("url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        },
        "group" => CanvasNodeKind::Group {
            label: object
                .get("label")
                .and_then(Value::as_str)
                .map(str::to_owned),
        },
        _ => CanvasNodeKind::Unknown { node_type },
    };

    Some(CanvasNode {
        id,
        kind,
        x,
        y,
        width,
        height,
    })
}

fn parse_canvas_edge(value: &Value) -> Option<CanvasEdge> {
    let object = value.as_object()?;
    Some(CanvasEdge {
        id: object.get("id")?.as_str()?.to_owned(),
        from_node: object.get("fromNode")?.as_str()?.to_owned(),
        to_node: object.get("toNode")?.as_str()?.to_owned(),
    })
}

fn numeric_field(value: Option<&Value>) -> Option<f32> {
    value.and_then(Value::as_f64).map(|value| value as f32)
}

fn json_number_from_f32(value: f32) -> Result<Value, CanvasParseError> {
    Number::from_f64(value as f64)
        .map(Value::Number)
        .ok_or_else(|| CanvasParseError::new("canvas coordinates must be finite"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_canvas_nodes() {
        let canvas = parse_canvas_document(
            r##"{
                "nodes": [
                    {"id":"a","type":"text","text":"# Title","x":10,"y":20,"width":300,"height":80},
                    {"id":"b","type":"file","file":"refs/door.png","x":40,"y":60,"width":0,"height":0}
                ],
                "edges": [{"id":"e","fromNode":"a","toNode":"b"}]
            }"##,
        )
        .expect("valid canvas");

        assert_eq!(canvas.nodes.len(), 2);
        assert_eq!(canvas.edges.len(), 1);
        assert!(matches!(canvas.nodes[0].kind, CanvasNodeKind::Text { .. }));
        assert!(matches!(canvas.nodes[1].kind, CanvasNodeKind::File { .. }));
    }

    #[test]
    fn parses_blank_canvas_as_empty_document() {
        let canvas = parse_canvas_document(" \n\t ").expect("blank canvas");

        assert!(canvas.nodes.is_empty());
        assert!(canvas.edges.is_empty());
    }

    #[test]
    fn parses_utf8_bom_canvas() {
        let canvas = parse_canvas_document(
            "\u{feff}{\"nodes\":[{\"id\":\"a\",\"type\":\"text\",\"x\":0,\"y\":0}],\"edges\":[]}",
        )
        .expect("canvas with bom");

        assert_eq!(canvas.nodes.len(), 1);
        assert!(canvas.edges.is_empty());
    }

    #[test]
    fn updates_canvas_node_position_without_dropping_fields() {
        let updated = update_canvas_node_position(
            r#"{"nodes":[{"id":"a","type":"text","text":"Note","color":"1","x":0,"y":0}],"edges":[]}"#,
            "a",
            12.5,
            -7.0,
        )
        .expect("updated canvas");
        let value = serde_json::from_str::<Value>(&updated).expect("json");
        let node = &value["nodes"][0];

        assert_eq!(node["x"], 12.5);
        assert_eq!(node["y"], -7.0);
        assert_eq!(node["color"], "1");
        assert_eq!(node["text"], "Note");
    }
}
