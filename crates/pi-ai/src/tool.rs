use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{ToolDefinition, FunctionDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: ToolInputSchema,
}

impl Tool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, input_schema: ToolInputSchema) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema,
        }
    }

    pub fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: serde_json::to_value(&self.input_schema).unwrap_or_default(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, Property>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

impl ToolInputSchema {
    pub fn new() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: None,
            additional_properties: Some(false),
        }
    }

    pub fn with_property(mut self, name: impl Into<String>, property: Property) -> Self {
        self.properties.insert(name.into(), property);
        self
    }

    pub fn with_required(mut self, required: Vec<impl Into<String>>) -> Self {
        self.required = Some(required.into_iter().map(|s| s.into()).collect());
        self
    }
}

impl Default for ToolInputSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    #[serde(rename = "type")]
    pub property_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Property>>,
}

impl Property {
    pub fn string() -> Self {
        Self {
            property_type: "string".to_string(),
            description: None,
            enum_values: None,
            items: None,
        }
    }

    pub fn number() -> Self {
        Self {
            property_type: "number".to_string(),
            description: None,
            enum_values: None,
            items: None,
        }
    }

    pub fn integer() -> Self {
        Self {
            property_type: "integer".to_string(),
            description: None,
            enum_values: None,
            items: None,
        }
    }

    pub fn boolean() -> Self {
        Self {
            property_type: "boolean".to_string(),
            description: None,
            enum_values: None,
            items: None,
        }
    }

    pub fn array(items: Property) -> Self {
        Self {
            property_type: "array".to_string(),
            description: None,
            enum_values: None,
            items: Some(Box::new(items)),
        }
    }

    pub fn object() -> Self {
        Self {
            property_type: "object".to_string(),
            description: None,
            enum_values: None,
            items: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_enum(mut self, values: Vec<impl Into<String>>) -> Self {
        self.enum_values = Some(values.into_iter().map(|s| s.into()).collect());
        self
    }
}
