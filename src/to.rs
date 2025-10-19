use nu_protocol::{LabeledError, Span, Value};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, KdlIdentifier};
use miette::SourceSpan;

fn span(value: &Value) -> SourceSpan {
    let Span { start, end } = value.span();
    SourceSpan::new(start.into(), (end - start).into())
}

pub(crate) fn build_document(document: &Value) -> Result<KdlDocument, LabeledError> {
    let mut doc = KdlDocument::new();

    doc.set_span(span(document));

    let nodes = doc.nodes_mut();

    // TODO: implement the else branch
    let record = document.as_record().map_err(|_| LabeledError::new("Expected a record"))?;

    for (col, val) in record.iter() {
        let node = build_node(col, val)?;
        nodes.push(node);
    }

    Ok(doc)
}

fn build_node(name: &str, node: &Value) -> Result<KdlNode, LabeledError> {
    let mut identifier = KdlIdentifier::from(name);
    identifier.set_repr(name);
    let mut kdl_node = KdlNode::new(identifier);

    kdl_node.set_span(span(node));

    kdl_node.clear_children();
    let entries = kdl_node.entries_mut();
    match node {
        Value::Nothing { .. } => {}
        Value::String { .. } | Value::Int { .. } | Value::Float { .. } | Value::Bool { .. } => {
            entries.push(build_entry(node).unwrap())
        }
        Value::List { vals, .. } => {
            for val in vals {
                entries.push(build_entry(val).unwrap())
            }
        }
        // TODO: implement when node is a record, i.e. with children
        // TODO: default arm
        _ => todo!(),
    }

    Ok(kdl_node)
}

fn build_entry(entry: &Value) -> Result<KdlEntry, LabeledError> {
    let entry_span = span(entry);

    let mut entry = match entry {
        Value::Record { val: record, .. } => {
            if record.len() != 1 {
                return Err(LabeledError::new("entry should be either a record with one key"));
            }

            let (key, val) = record.iter().next().unwrap();

            let kdl_val = match val {
                Value::String { val, .. } => KdlValue::String(val.to_string()),
                Value::Int { val, .. } => KdlValue::from(*val as i128),
                Value::Float { val, .. } => KdlValue::from(*val),
                Value::Bool { val, .. } => KdlValue::Bool(*val),
                Value::Nothing { .. } => KdlValue::Null,
                _ => {
                    return Err(LabeledError::new("value not supported, expected string, int, float, bool or null"));
                }
            };

            KdlEntry::new_prop(key.clone(), kdl_val)
        }
        Value::String { val, .. } => KdlEntry::new(KdlValue::String(val.to_string())),
        Value::Int { val, .. } => KdlEntry::new(KdlValue::from(*val as i128)),
        Value::Float { val, .. } => KdlEntry::new(KdlValue::from(*val)),
        Value::Bool { val, .. } => KdlEntry::new(KdlValue::Bool(*val)),
        Value::Nothing { .. } => KdlEntry::new(KdlValue::Null),
        // TODO: default arm
        _ => todo!(),
    };

    entry.set_span(entry_span);

    Ok(entry)
}
