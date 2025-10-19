mod from;
mod to;

use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{Category, LabeledError, PipelineData, Signature, Type, Value};

use kdl::KdlDocument;

pub struct KDL;

impl KDL {
    pub fn from(&self, _call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
        let input_str = input
            .as_str()
            .map_err(|e| LabeledError::new(format!("input is not a string: {}", e)))?;

        let doc = input_str.parse::<KdlDocument>()
            .map_err(|e| LabeledError::new(format!("invalid KDL format: {}", e)))?;

        Ok(from::parse_document(&doc))
    }

    pub fn to(&self, call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
        let document = to::build_document(input)?;
        Ok(Value::string(document.to_string(), call.head))
    }
}

pub struct FromKdl;
pub struct ToKdl;

impl Plugin for KDL {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(FromKdl), Box::new(ToKdl)]
    }
}

impl PluginCommand for FromKdl {
    type Plugin = KDL;

    fn name(&self) -> &str {
        "from kdl"
    }

    fn description(&self) -> &str {
        "Convert KDL document to Nushell record"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::String, Type::Record(vec![].into()))
            .category(Category::Experimental)
    }

    fn run(
        &self,
        plugin: &KDL,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let value = input.into_value(call.head)?;
        let result = plugin.from(call, &value)?;
        Ok(PipelineData::Value(result, None))
    }
}

impl PluginCommand for ToKdl {
    type Plugin = KDL;

    fn name(&self) -> &str {
        "to kdl"
    }

    fn description(&self) -> &str {
        "Convert Nushell record to KDL document"
    }

    fn signature(&self) -> Signature {
        Signature::build(PluginCommand::name(self))
            .input_output_type(Type::Record(vec![].into()), Type::String)
            .category(Category::Experimental)
    }

    fn run(
        &self,
        plugin: &KDL,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let value = input.into_value(call.head)?;
        let result = plugin.to(call, &value)?;
        Ok(PipelineData::Value(result, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_zellij_layout() {
        let input = include_str!("../zellij-layout.kdl");
        let result = input.parse::<KdlDocument>();

        match &result {
            Ok(doc) => {
                println!("Successfully parsed document with {} nodes", doc.nodes().len());
                for node in doc.nodes() {
                    println!("  Node: {}", node.name());
                }
            }
            Err(e) => {
                println!("Parse error: {}", e);
            }
        }

        assert!(result.is_ok(), "Failed to parse zellij-layout.kdl: {:?}", result.err());
    }

    #[test]
    fn test_parse_simple_kdl() {
        let input = r#"node1 "value1"
node2 123"#;
        let result = input.parse::<KdlDocument>();
        assert!(result.is_ok(), "Failed to parse simple KDL: {:?}", result.err());
    }

    #[test]
    fn test_parse_kdl_with_children() {
        let input = r#"parent {
    child "value"
}"#;
        let result = input.parse::<KdlDocument>();
        assert!(result.is_ok(), "Failed to parse KDL with children: {:?}", result.err());
    }

    #[test]
    fn test_parse_kdl_with_properties() {
        let input = r#"pane size=1 borderless=true"#;
        let result = input.parse::<KdlDocument>();
        if let Err(e) = &result {
            println!("Error parsing properties: {}", e);
        }
        assert!(result.is_ok(), "Failed to parse KDL with properties: {:?}", result.err());
    }

    #[test]
    fn test_parse_kdl_bare_word() {
        let input = r#"children"#;
        let result = input.parse::<KdlDocument>();
        if let Err(e) = &result {
            println!("Error parsing bare word: {}", e);
        }
        assert!(result.is_ok(), "Failed to parse bare word: {:?}", result.err());
    }
}
