/// Converter from Cursor protobuf format to canonical JSONL

use super::protobuf::{CursorBlob, ContentBlock as CursorContentBlock};
use crate::providers::canonical::{
    CanonicalMessage, ContentBlock, ContentValue, MessageContent, MessageType,
};
use crate::providers::canonical::converter::ToCanonical;
use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Value};

impl ToCanonical for CursorBlob {
    fn to_canonical(&self) -> Result<Option<CanonicalMessage>> {
        // Skip empty messages
        if self.content.is_empty() && !self.is_complex() {
            return Ok(None);
        }

        // Generate timestamp (we don't have timestamps in Cursor blobs, use current time)
        // The watcher will need to provide ordering based on database order
        let timestamp = Utc::now().to_rfc3339();

        // Determine message type and role
        let role = self.get_role();
        let message_type = match role.as_str() {
            "user" => MessageType::User,
            "assistant" => MessageType::Assistant,
            _ => MessageType::Meta,
        };

        // Build content
        let content = if self.is_complex() {
            // Complex message with tool calls, reasoning, etc.
            self.build_structured_content()?
        } else {
            // Simple text message
            ContentValue::Text(self.get_content())
        };

        // Extract model if available from complex data
        let model = self
            .parse_complex()
            .and_then(|c| Some(c.role.clone()))
            .filter(|r| r == "assistant")
            .map(|_| "default".to_string()); // Cursor doesn't expose model details in blobs

        // Build canonical message
        Ok(Some(CanonicalMessage {
            uuid: if !self.uuid.is_empty() {
                self.uuid.clone()
            } else {
                // Generate UUID for messages without one (assistant responses)
                uuid::Uuid::new_v4().to_string()
            },
            timestamp,
            message_type,
            session_id: String::new(), // Will be set by watcher
            provider: self.provider_name().to_string(),
            cwd: None, // Cursor doesn't track CWD in blobs
            git_branch: None,
            version: None,
            parent_uuid: None,
            is_sidechain: None,
            user_type: None,
            message: MessageContent {
                role: role.clone(),
                content,
                model,
                usage: None, // Cursor doesn't expose token usage in blobs
            },
            provider_metadata: Some(self.build_metadata()),
            is_meta: None,
            request_id: None,
            tool_use_result: None,
        }))
    }

    fn provider_name(&self) -> &str {
        "cursor"
    }

    fn extract_cwd(&self) -> Option<String> {
        // Cursor doesn't track CWD in message blobs
        // We could potentially extract from tool calls if present
        None
    }

    fn extract_git_branch(&self) -> Option<String> {
        None
    }

    fn extract_version(&self) -> Option<String> {
        None
    }
}

impl CursorBlob {
    /// Build structured content from complex message data
    fn build_structured_content(&self) -> Result<ContentValue> {
        let complex = self
            .parse_complex()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse complex message"))?;

        let mut blocks = Vec::new();

        for cursor_block in complex.content {
            match cursor_block {
                CursorContentBlock::Text { text } => {
                    blocks.push(ContentBlock::Text { text });
                }
                CursorContentBlock::ToolCall {
                    tool_call_id,
                    tool_name,
                    args,
                } => {
                    blocks.push(ContentBlock::ToolUse {
                        id: tool_call_id,
                        name: tool_name,
                        input: args,
                    });
                }
                CursorContentBlock::RedactedReasoning { data } => {
                    // Treat redacted reasoning as thinking
                    blocks.push(ContentBlock::Thinking {
                        thinking: format!("[Redacted reasoning: {} bytes]", data.len()),
                    });
                }
            }
        }

        Ok(ContentValue::Structured(blocks))
    }

    /// Build provider-specific metadata
    fn build_metadata(&self) -> Value {
        let mut metadata = json!({});

        if !self.metadata.is_empty() {
            metadata["metadata"] = json!(self.metadata);
        }

        if let Some(ref _complex) = self.complex_data {
            metadata["has_complex_data"] = json!(true);

            // Try to parse and extract interesting fields
            if let Some(parsed) = self.parse_complex() {
                metadata["message_id"] = json!(parsed.id);
            }
        }

        if self.additional_content.is_some() {
            metadata["has_additional_content"] = json!(true);
        }

        if self.blob_references.is_some() {
            metadata["has_blob_references"] = json!(true);
        }

        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_message_conversion() {
        let blob = CursorBlob {
            content: "Test message".to_string(),
            uuid: "test-uuid".to_string(),
            metadata: String::new(),
            complex_data: None,
            additional_content: None,
            blob_references: None,
        };

        let canonical = blob.to_canonical().unwrap().unwrap();

        assert_eq!(canonical.provider, "cursor");
        assert_eq!(canonical.uuid, "test-uuid");

        match canonical.message.content {
            ContentValue::Text(text) => assert_eq!(text, "Test message"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_skip_empty_messages() {
        let blob = CursorBlob {
            content: String::new(),
            uuid: String::new(),
            metadata: String::new(),
            complex_data: None,
            additional_content: None,
            blob_references: None,
        };

        let result = blob.to_canonical().unwrap();
        assert!(result.is_none(), "Empty messages should be skipped");
    }
}
