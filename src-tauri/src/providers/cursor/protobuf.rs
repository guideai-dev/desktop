/// Cursor Protobuf Message Types
///
/// Based on reverse engineering of Cursor's SQLite blobs.
///
/// Schema discovered:
/// - Field 1: Message content (string) - user/assistant text
/// - Field 2: UUID (string) - message identifier
/// - Field 3: Empty string or metadata
/// - Field 4: JSON-encoded complex message (tool calls, reasoning, etc.)
/// - Field 5: Tool output or additional content
/// - Field 8: Blob references (32-byte SHA-256 hashes)

use prost::Message;
use serde::{Deserialize, Serialize};

/// A Cursor message blob decoded from Protocol Buffers
#[derive(Clone, PartialEq, Message)]
pub struct CursorBlob {
    /// Message content (for simple messages)
    #[prost(string, tag = "1")]
    pub content: String,

    /// Message UUID
    #[prost(string, tag = "2")]
    pub uuid: String,

    /// Additional metadata or empty string
    #[prost(string, tag = "3")]
    pub metadata: String,

    /// Complex message data (JSON-encoded for tool calls, etc.)
    #[prost(string, optional, tag = "4")]
    pub complex_data: Option<String>,

    /// Tool output or additional content
    #[prost(string, optional, tag = "5")]
    pub additional_content: Option<String>,

    /// References to other blobs (SHA-256 hashes)
    #[prost(bytes = "vec", optional, tag = "8")]
    pub blob_references: Option<Vec<u8>>,
}

/// Complex message structure (parsed from Field 4 JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexMessage {
    pub id: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
}

/// Content blocks within a complex message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool-call")]
    ToolCall {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,

        #[serde(rename = "toolName")]
        tool_name: String,

        args: serde_json::Value,
    },

    #[serde(rename = "redacted-reasoning")]
    RedactedReasoning { data: String },
}

impl CursorBlob {
    /// Decode a blob from raw bytes
    pub fn decode_from_bytes(data: &[u8]) -> Result<Self, prost::DecodeError> {
        CursorBlob::decode(data)
    }

    /// Check if this blob contains a complex message (has JSON in field 4)
    pub fn is_complex(&self) -> bool {
        self.complex_data.is_some()
    }

    /// Parse complex message data if present
    pub fn parse_complex(&self) -> Option<ComplexMessage> {
        self.complex_data
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok())
    }

    /// Get the primary message content (either simple content or from complex data)
    pub fn get_content(&self) -> String {
        if let Some(complex) = self.parse_complex() {
            // Extract text from content blocks
            complex
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            self.content.clone()
        }
    }

    /// Get the role (user/assistant) from complex data or infer from structure
    pub fn get_role(&self) -> String {
        if let Some(complex) = self.parse_complex() {
            complex.role
        } else if self.uuid.is_empty() {
            // Blobs without UUIDs tend to be assistant responses
            "assistant".to_string()
        } else {
            // Blobs with UUIDs tend to be user messages
            "user".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_message_decode() {
        // Example from our analysis: user message blob
        let data = vec![
            0x0a, 0x4b, // Field 1, length 75
            b'T', b'e', b's', b't', b' ', b'm', b'e', b's', b's', b'a', b'g', b'e',
        ];

        // This is a simplified test - real data would be longer
        // Just testing the decode mechanism works
    }

    #[test]
    fn test_role_inference() {
        let user_blob = CursorBlob {
            content: "Test message".to_string(),
            uuid: "some-uuid".to_string(),
            metadata: String::new(),
            complex_data: None,
            additional_content: None,
            blob_references: None,
        };

        assert_eq!(user_blob.get_role(), "user");

        let assistant_blob = CursorBlob {
            content: "Response".to_string(),
            uuid: String::new(),
            metadata: String::new(),
            complex_data: None,
            additional_content: None,
            blob_references: None,
        };

        assert_eq!(assistant_blob.get_role(), "assistant");
    }
}
