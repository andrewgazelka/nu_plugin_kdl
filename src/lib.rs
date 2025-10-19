mod from;
mod to;

use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{Category, LabeledError, PipelineData, Signature, Type, Value};

use kdl::KdlDocument;

pub struct KDL;

impl KDL {
    pub fn from(&self, call: &EvaluatedCall, input: &Value) -> Result<Value, LabeledError> {
        let input_str = input
            .as_str()
            .map_err(|e| LabeledError::new(format!("input is not a string: {}", e)))?;

        // Check for version flags
        let v1_fallback = call.has_flag("v1-fallback")?;
        let force_v1 = call.has_flag("v1")?;

        let doc = if force_v1 {
            // Explicitly parse as KDL v1
            KdlDocument::parse_v1(input_str)
                .map_err(|e| LabeledError::new(format!("invalid KDL v1 format: {}", e)))?
        } else if v1_fallback {
            // Try v2, if that fails, try v1
            match input_str.parse::<KdlDocument>() {
                Ok(doc) => doc,
                Err(_) => KdlDocument::parse_v1(input_str)
                    .map_err(|e| LabeledError::new(format!("invalid KDL format (tried v2 and v1): {}", e)))?
            }
        } else {
            // Default: strict v2 only
            input_str.parse::<KdlDocument>()
                .map_err(|e| LabeledError::new(format!("invalid KDL v2 format: {}", e)))?
        };

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
            .switch("v1", "Force parsing as KDL v1 only", Some('1'))
            .switch("v1-fallback", "Try KDL v2, fall back to v1 if parsing fails", None)
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
    fn test_parse_zellij_layout_v1() {
        // Zellij layout files use KDL v1 syntax, so we need to use parse_v1
        let input = include_str!("../zellij-layout.kdl");
        let result = KdlDocument::parse_v1(input);

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
    fn test_parse_kdl_v1_properties() {
        // KDL v1 uses key=value syntax
        let input = r#"pane size=1 borderless=true"#;
        let result = KdlDocument::parse_v1(input);
        if let Err(e) = &result {
            println!("Error parsing v1 properties: {}", e);
        }
        assert!(result.is_ok(), "Failed to parse KDL v1 with properties: {:?}", result.err());
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

    #[test]
    fn test_parse_v1_explicit() {
        // KDL v1 syntax with key=value properties
        let input = r#"pane size=1 borderless=true"#;
        let result = KdlDocument::parse_v1(input);
        assert!(result.is_ok(), "Failed to parse KDL v1: {:?}", result.err());
    }

    #[test]
    fn test_parse_v2_syntax() {
        // KDL v2 syntax - simpler node with arguments
        let input = r#"node "arg1" "arg2""#;
        // Default parse is v2
        let result = input.parse::<KdlDocument>();
        assert!(result.is_ok(), "Failed to parse KDL v2: {:?}", result.err());
    }

    #[test]
    fn test_v1_fallback_behavior() {
        // KDL v1 syntax with properties
        let kdl_v1_input = r#"node size=1"#;

        // Test v1 parsing explicitly
        let v1_result = KdlDocument::parse_v1(kdl_v1_input);
        assert!(v1_result.is_ok(), "V1 parsing should work: {:?}", v1_result.err());

        // Test manual fallback logic (simulating --v1-fallback flag)
        // Try v2 first, fall back to v1
        let fallback_result = match kdl_v1_input.parse::<KdlDocument>() {
            Ok(doc) => Ok(doc),
            Err(_) => KdlDocument::parse_v1(kdl_v1_input),
        };
        assert!(fallback_result.is_ok(), "Fallback should work: {:?}", fallback_result.err());
    }

    #[test]
    fn test_different_kdl_versions() {
        // Both versions should handle basic nodes fine
        let simple = r#"node "value""#;

        let v1_result = KdlDocument::parse_v1(simple);
        let v2_result = simple.parse::<KdlDocument>();

        assert!(v1_result.is_ok(), "V1 should parse simple node");
        assert!(v2_result.is_ok(), "V2 should parse simple node");
    }
}
