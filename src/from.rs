use nu_protocol::{Record, Span, Value};

use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue};

pub(crate) fn parse_document(document: &KdlDocument) -> Value {
    let mut record = Record::new();

    for node in document.nodes() {
        record.insert(node.name().to_string(), parse_node(node));
    }

    let span = Span::new(
        document.span().offset(),
        document.span().offset() + document.len(),
    );

    Value::record(record, span)
}

fn parse_node(node: &KdlNode) -> Value {
    let entries: Vec<Value> = node.entries().iter().map(parse_entry).collect();

    let span = Span::new(node.span().offset(), node.span().offset() + node.len());

    if let Some(children) = node.children() {
        let children = parse_document(children);

        if entries.is_empty() {
            return children;
        }

        let entries = if entries.len() == 1 {
            entries[0].clone()
        } else {
            // FIXME: use a real span
            Value::list(entries, Span::unknown())
        };

        let mut record = Record::new();
        record.insert("entries".to_string(), entries);
        record.insert("children".to_string(), children);
        Value::record(record, span)
    } else {
        if entries.is_empty() {
            // FIXME: use a real span
            Value::nothing(Span::unknown())
        } else if entries.len() == 1 {
            entries[0].clone()
        } else {
            // FIXME: use a real span
            Value::list(entries, Span::unknown())
        }
    }
}

fn parse_entry(entry: &KdlEntry) -> Value {
    let span = Span::new(entry.span().offset(), entry.span().offset() + entry.len());

    let value = match entry.value() {
        KdlValue::String(val) => Value::string(val, span),
        KdlValue::Bool(val) => Value::bool(*val, span),
        KdlValue::Null => Value::nothing(span),
        val => {
            // Try to convert to string first, then parse as number
            let s = val.to_string();
            // Try parsing as int first
            if let Ok(i) = s.parse::<i64>() {
                Value::int(i, span)
            } else if let Ok(f) = s.parse::<f64>() {
                Value::float(f, span)
            } else {
                // Fallback to string representation
                Value::string(s, span)
            }
        }
    };

    match entry.name() {
        Some(name) => {
            let mut record = Record::new();
            record.insert(name.value().to_string(), value);
            Value::record(record, span)
        }
        None => value,
    }
}
