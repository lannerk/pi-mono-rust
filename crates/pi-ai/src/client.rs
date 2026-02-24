use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, Stream, StreamExt};
use http::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client as ReqwestClient;
use serde_json::{json, Value};

use crate::config::{Config};
use crate::error::{Error, Result};
use crate::models::{ChatCompletionRequest, ChatCompletionResponse};
use crate::provider::{Provider, ProviderType};
use crate::stream::{StreamChunk, StreamEvent};

#[derive(Debug, Clone)]
pub struct Client {
    config: Arc<Config>,
    http_client: Arc<ReqwestClient>,
    providers: Arc<HashMap<String, Provider>>,
}

impl Client {
    pub fn new(mut config: Config) -> Result<Self> {
        let http_client = ReqwestClient::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(Error::Http)?;

        let mut providers = HashMap::new();
        let providers_iter = config.providers.drain();
        
        for (name, provider_config) in providers_iter {
            let provider_type = ProviderType::from(name.as_str());
            let provider = Provider::new(name.clone(), provider_type, provider_config);
            providers.insert(name, provider);
        }

        Ok(Self {
            config: Arc::new(config),
            http_client: Arc::new(http_client),
            providers: Arc::new(providers),
        })
    }

    pub async fn chat(
        &self,
        provider_name: &str,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| Error::UnsupportedProvider(provider_name.to_string()))?;

        let url = provider.get_endpoint("chat/completions");
        let headers = self.build_headers(provider)?;

        let body = self.build_request_body(&request)?;

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Err(Error::ApiError(
                response.status().as_u16(),
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let response: ChatCompletionResponse = response.json().await.map_err(Error::Http)?;
        Ok(response)
    }

    pub async fn chat_stream(
        &self,
        provider_name: &str,
        request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| Error::UnsupportedProvider(provider_name.to_string()))?;

        let url = provider.get_endpoint("chat/completions");
        let headers = self.build_headers(provider)?;

        let mut body = self.build_request_body(&request)?;
        body["stream"] = Value::Bool(true);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Err(Error::ApiError(
                response.status().as_u16(),
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let stream = response.bytes_stream();
        let stream = stream.flat_map(|chunk| {
            match chunk {
                Ok(bytes) => {
                    let lines = std::str::from_utf8(&bytes)
                        .unwrap_or_default()
                        .lines()
                        .filter(|line| !line.is_empty())
                        .filter(|line| line.starts_with("data: "))
                        .map(|line| line.trim_start_matches("data: "))
                        .collect::<Vec<_>>();

                    let events = lines.into_iter().flat_map(|line| {
                        if line == "[DONE]" {
                            vec![Ok(StreamEvent::Done)]
                        } else {
                            match serde_json::from_str::<StreamChunk>(line) {
                                Ok(chunk) => {
                                    // 解析 StreamChunk 为 StreamEvent
                                    let chunk_events = crate::stream::chunk_to_event(&chunk);
                                    chunk_events.into_iter().map(Ok).collect()
                                }
                                Err(e) => {
                                    vec![Err(Error::Stream(e.to_string()))]
                                }
                            }
                        }
                    }).collect::<Vec<_>>();

                    stream::iter(events)
                }
                Err(e) => {
                    stream::iter(vec![Err(Error::Stream(e.to_string()))])
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn build_headers(&self, provider: &Provider) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // 添加 Content-Type 头
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );

        // 添加 API Key 头
        match provider.provider_type {
            ProviderType::OpenAI => {
                headers.insert(
                    HeaderName::from_static("authorization"),
                    HeaderValue::from_str(&format!("Bearer {}", provider.config.api_key))
                        .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                );
                if let Some(org) = &provider.config.organization {
                    headers.insert(
                        HeaderName::from_static("openai-organization"),
                        HeaderValue::from_str(org)
                            .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                    );
                }
            }
            ProviderType::Anthropic => {
                headers.insert(
                    HeaderName::from_static("x-api-key"),
                    HeaderValue::from_str(&provider.config.api_key)
                        .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                );
                headers.insert(
                    HeaderName::from_static("anthropic-version"),
                    HeaderValue::from_static("2023-06-01"),
                );
            }
            ProviderType::Google => {
                headers.insert(
                    HeaderName::from_static("x-goog-api-key"),
                    HeaderValue::from_str(&provider.config.api_key)
                        .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                );
            }
            ProviderType::Azure => {
                headers.insert(
                    HeaderName::from_static("api-key"),
                    HeaderValue::from_str(&provider.config.api_key)
                        .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                );
            }
            ProviderType::Custom => {
                headers.insert(
                    HeaderName::from_static("authorization"),
                    HeaderValue::from_str(&format!("Bearer {}", provider.config.api_key))
                        .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?,
                );
            }
        }

        // 添加自定义头
        for (k, v) in &provider.config.headers {
            let key = HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| Error::InvalidHeaderName(e.to_string()))?;
            let value = HeaderValue::from_str(v)
                .map_err(|e| Error::InvalidHeaderValue(e.to_string()))?;
            headers.insert(key, value);
        }

        Ok(headers)
    }

    fn build_request_body(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<Value> {
        let mut body = json!({"model": request.model,"messages": request.messages,"temperature": request.temperature,"top_p": request.top_p,"stream": false,"stop": request.stop,"max_tokens": request.max_tokens,"presence_penalty": request.presence_penalty,"frequency_penalty": request.frequency_penalty,"user": request.user,});

        if let Some(tools) = &request.tools {
            if !tools.is_empty() {
                body["tools"] = serde_json::to_value(tools).map_err(Error::Json)?;
            }
        }

        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = serde_json::to_value(tool_choice).map_err(Error::Json)?;
        }

        Ok(body)
    }

    pub async fn list_models(&self, provider_name: &str) -> Result<Vec<String>> {
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| Error::UnsupportedProvider(provider_name.to_string()))?;

        let url = match provider.provider_type {
            ProviderType::OpenAI => provider.get_endpoint("models"),
            ProviderType::Anthropic => provider.get_endpoint("models"),
            ProviderType::Google => provider.get_endpoint("models"),
            _ => return Err(Error::UnsupportedProviderType(provider.provider_type.to_string())),
        };

        let headers = self.build_headers(provider)?;

        let response = self
            .http_client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(Error::Http)?;

        if !response.status().is_success() {
            return Err(Error::ApiError(
                response.status().as_u16(),
                response.text().await.unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let json: Value = response.json().await.map_err(Error::Http)?;
        let models = self.extract_models(&json, provider)?;

        Ok(models)
    }

    fn extract_models(&self, json: &Value, provider: &Provider) -> Result<Vec<String>> {
        match provider.provider_type {
            ProviderType::OpenAI => {
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| Error::InvalidResponse("Expected 'data' array".to_string()))?
                    .iter()
                    .filter_map(|item| item["id"].as_str())
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>();
                Ok(models)
            }
            ProviderType::Anthropic => {
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| Error::InvalidResponse("Expected 'models' array".to_string()))?
                    .iter()
                    .filter_map(|item| item["id"].as_str())
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>();
                Ok(models)
            }
            ProviderType::Google => {
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| Error::InvalidResponse("Expected 'models' array".to_string()))?
                    .iter()
                    .filter_map(|item| item["name"].as_str())
                    .map(|name| name.to_string())
                    .collect::<Vec<String>>();
                Ok(models)
            }
            _ => Err(Error::UnsupportedProviderType(provider.provider_type.to_string())),
        }
    }

    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    pub fn providers(&self) -> &Arc<HashMap<String, Provider>> {
        &self.providers
    }
}
