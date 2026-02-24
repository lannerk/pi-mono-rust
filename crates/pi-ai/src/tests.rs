#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(ProviderType::from("openai"), ProviderType::OpenAI);
        assert_eq!(ProviderType::from("anthropic"), ProviderType::Anthropic);
        assert_eq!(ProviderType::from("google"), ProviderType::Google);
        assert_eq!(ProviderType::from("azure"), ProviderType::Azure);
        assert_eq!(ProviderType::from("custom"), ProviderType::Custom);
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::OpenAI.to_string(), "openai");
        assert_eq!(ProviderType::Anthropic.to_string(), "anthropic");
        assert_eq!(ProviderType::Google.to_string(), "google");
        assert_eq!(ProviderType::Azure.to_string(), "azure");
        assert_eq!(ProviderType::Custom.to_string(), "custom");
    }

    #[test]
    fn test_message_role_from_str() {
        assert_eq!(MessageRole::from("system"), MessageRole::System);
        assert_eq!(MessageRole::from("user"), MessageRole::User);
        assert_eq!(MessageRole::from("assistant"), MessageRole::Assistant);
        assert_eq!(MessageRole::from("tool"), MessageRole::Tool);
    }

    #[test]
    fn test_message_creation() {
        let system_msg = Message::system("You are a helpful assistant");
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(system_msg.content, "You are a helpful assistant");

        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = Message::assistant("Hi there!");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there!");

        let tool_msg = Message::tool("Result", "call-123");
        assert_eq!(tool_msg.role, MessageRole::Tool);
        assert_eq!(tool_msg.content, "Result");
        assert_eq!(tool_msg.tool_call_id, Some("call-123".to_string()));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_provider, "openai");
        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_tool_choice() {
        let auto = ToolChoice::auto();
        let none = ToolChoice::none();
        let required = ToolChoice::required();
        let function = ToolChoice::function("test_function");

        match auto {
            ToolChoice::Auto(s) => assert_eq!(s, "auto"),
            _ => panic!("Expected Auto"),
        }

        match none {
            ToolChoice::None(s) => assert_eq!(s, "none"),
            _ => panic!("Expected None"),
        }

        match required {
            ToolChoice::Required(s) => assert_eq!(s, "required"),
            _ => panic!("Expected Required"),
        }

        match function {
            ToolChoice::Function { func } => assert_eq!(func.name, "test_function"),
            _ => panic!("Expected Function"),
        }
    }
}
