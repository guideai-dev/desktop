use crate::providers::canonical::{
    CanonicalMessage, ContentBlock, ContentValue, MessageContent, MessageType,
};
use crate::providers::copilot_parser::TimelineEntry;
use anyhow::{Context, Result};
use serde_json::Value;
use uuid::Uuid;

/// Convert a timeline entry to one or more canonical messages
///
/// Some timeline entries (like tool_call_completed) need to be split into multiple messages:
/// - tool_call_completed → tool_use + tool_result (2 messages)
/// - All others → single message
pub fn convert_timeline_entry_to_canonical(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<Vec<CanonicalMessage>> {
    // Extract type from the data object
    let entry_type = entry
        .data
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    match entry_type {
        "user" => Ok(vec![convert_user_message(entry, session_id, cwd)?]),
        "copilot" => Ok(vec![convert_assistant_message(entry, session_id, cwd)?]),
        "info" => Ok(vec![convert_info_message(entry, session_id, cwd)?]),
        "tool_call_requested" => Ok(vec![convert_tool_use(entry, session_id, cwd)?]),
        "tool_call_completed" => {
            // Split into tool_use + tool_result
            let tool_use = convert_tool_use(entry, session_id, cwd)?;
            let tool_result = convert_tool_result(entry, session_id, cwd)?;
            Ok(vec![tool_use, tool_result])
        }
        _ => {
            // Unknown type - create a meta message
            Ok(vec![convert_unknown_message(entry, session_id, cwd)?])
        }
    }
}

/// Convert user message timeline entry
fn convert_user_message(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;
    let text = extract_text(entry);

    Ok(CanonicalMessage {
        uuid: id,
        timestamp,
        message_type: MessageType::User,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: None,
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "user".to_string(),
            content: ContentValue::Text(text),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": "user",
        })),
        is_meta: None,
        request_id: None,
        tool_use_result: None,
    })
}

/// Convert assistant (copilot) message timeline entry
fn convert_assistant_message(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;
    let text = extract_text(entry);

    // Check for intention summary
    let intention = entry
        .data
        .get("intentionSummary")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(CanonicalMessage {
        uuid: id,
        timestamp,
        message_type: MessageType::Assistant,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: None,
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "assistant".to_string(),
            content: ContentValue::Text(text),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": "copilot",
            "has_intention": intention.is_some(),
        })),
        is_meta: None,
        request_id: None,
        tool_use_result: None,
    })
}

/// Convert info message timeline entry
fn convert_info_message(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;
    let text = extract_text(entry);

    Ok(CanonicalMessage {
        uuid: id,
        timestamp,
        message_type: MessageType::Meta,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: None,
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "assistant".to_string(),
            content: ContentValue::Text(text),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": "info",
        })),
        is_meta: Some(true),
        request_id: None,
        tool_use_result: None,
    })
}

/// Convert tool_call_requested or tool_call_completed to tool_use message
fn convert_tool_use(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;

    // Extract tool call details
    let call_id = entry
        .data
        .get("callId")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();

    let tool_name = entry
        .data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    // Parse arguments (can be a JSON string or object)
    let arguments = if let Some(args) = entry.data.get("arguments") {
        if let Some(args_str) = args.as_str() {
            // Try to parse JSON string
            serde_json::from_str(args_str).unwrap_or(Value::String(args_str.to_string()))
        } else {
            // Already an object
            args.clone()
        }
    } else {
        Value::Null
    };

    let tool_block = ContentBlock::ToolUse {
        id: call_id.clone(),
        name: tool_name,
        input: arguments,
    };

    // Check for intention summary and tool title
    let intention = entry
        .data
        .get("intentionSummary")
        .and_then(|v| v.as_str())
        .map(String::from);
    let tool_title = entry
        .data
        .get("toolTitle")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(CanonicalMessage {
        uuid: call_id,
        timestamp,
        message_type: MessageType::Assistant,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: None,
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "assistant".to_string(),
            content: ContentValue::Structured(vec![tool_block]),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": "tool_call_requested",
            "has_intention": intention.is_some(),
            "has_tool_title": tool_title.is_some(),
        })),
        is_meta: None,
        request_id: None,
        tool_use_result: None,
    })
}

/// Convert tool_call_completed to tool_result message
fn convert_tool_result(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;

    // Extract call ID
    let call_id = entry
        .data
        .get("callId")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();

    // Extract result
    let result_content = if let Some(result) = entry.data.get("result") {
        if result.is_string() {
            result.as_str().unwrap_or("").to_string()
        } else {
            serde_json::to_string(result).unwrap_or_default()
        }
    } else {
        String::new()
    };

    let tool_result_block = ContentBlock::ToolResult {
        tool_use_id: call_id.clone(),
        content: result_content,
        is_error: Some(false), // Copilot doesn't clearly indicate errors
    };

    Ok(CanonicalMessage {
        uuid: format!("{}_result", id),
        timestamp,
        message_type: MessageType::Assistant,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: Some(call_id),
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "assistant".to_string(),
            content: ContentValue::Structured(vec![tool_result_block]),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": "tool_result",
        })),
        is_meta: None,
        request_id: None,
        tool_use_result: None,
    })
}

/// Convert unknown message type to meta message
fn convert_unknown_message(
    entry: &TimelineEntry,
    session_id: &str,
    cwd: Option<&str>,
) -> Result<CanonicalMessage> {
    let id = extract_id(entry);
    let timestamp = extract_timestamp(entry)?;
    let text = extract_text(entry);
    let entry_type = entry
        .data
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    Ok(CanonicalMessage {
        uuid: id,
        timestamp,
        message_type: MessageType::Meta,
        session_id: session_id.to_string(),
        provider: "github-copilot".to_string(),
        cwd: cwd.map(String::from),
        git_branch: None,
        version: None,
        parent_uuid: None,
        is_sidechain: None,
        user_type: Some("external".to_string()),
        message: MessageContent {
            role: "assistant".to_string(),
            content: ContentValue::Text(text),
            model: None,
            usage: None,
        },
        provider_metadata: Some(serde_json::json!({
            "copilot_type": entry_type,
            "warning": "unknown_type",
        })),
        is_meta: Some(true),
        request_id: None,
        tool_use_result: None,
    })
}

/// Extract ID from timeline entry
fn extract_id(entry: &TimelineEntry) -> String {
    entry
        .data
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Extract timestamp from timeline entry
fn extract_timestamp(entry: &TimelineEntry) -> Result<String> {
    entry
        .timestamp
        .clone()
        .or_else(|| {
            // Fallback: use current time if no timestamp
            Some(chrono::Utc::now().to_rfc3339())
        })
        .context("Missing timestamp")
}

/// Extract text content from timeline entry
fn extract_text(entry: &TimelineEntry) -> String {
    entry
        .data
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_entry(entry_type: &str, data: Value) -> TimelineEntry {
        let mut data_obj = data.as_object().unwrap().clone();
        data_obj.insert("type".to_string(), json!(entry_type));
        data_obj.insert("id".to_string(), json!("test-123"));

        TimelineEntry {
            timestamp: Some("2025-01-01T10:00:00.000Z".to_string()),
            data: Value::Object(data_obj),
        }
    }

    #[test]
    fn test_convert_user_message() {
        let entry = create_test_entry("user", json!({ "text": "Hello" }));
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", Some("/test"))
            .unwrap();

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.message_type, MessageType::User);
        assert_eq!(msg.session_id, "session-1");
        assert_eq!(msg.cwd, Some("/test".to_string()));

        match &msg.message.content {
            ContentValue::Text(text) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_convert_assistant_message() {
        let entry = create_test_entry("copilot", json!({ "text": "Hi there" }));
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", None).unwrap();

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.message_type, MessageType::Assistant);

        match &msg.message.content {
            ContentValue::Text(text) => assert_eq!(text, "Hi there"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_convert_tool_call_requested() {
        let entry = create_test_entry(
            "tool_call_requested",
            json!({
                "callId": "call-456",
                "name": "read_file",
                "arguments": r#"{"path": "/test/file.txt"}"#
            }),
        );
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", None).unwrap();

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.message_type, MessageType::Assistant);
        assert_eq!(msg.uuid, "call-456");

        match &msg.message.content {
            ContentValue::Structured(blocks) => {
                assert_eq!(blocks.len(), 1);
                match &blocks[0] {
                    ContentBlock::ToolUse { id, name, .. } => {
                        assert_eq!(id, "call-456");
                        assert_eq!(name, "read_file");
                    }
                    _ => panic!("Expected tool_use block"),
                }
            }
            _ => panic!("Expected structured content"),
        }
    }

    #[test]
    fn test_convert_tool_call_completed() {
        let entry = create_test_entry(
            "tool_call_completed",
            json!({
                "callId": "call-789",
                "name": "read_file",
                "arguments": r#"{"path": "/test/file.txt"}"#,
                "result": "File contents here"
            }),
        );
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", None).unwrap();

        // Should produce 2 messages: tool_use + tool_result
        assert_eq!(result.len(), 2);

        // First message: tool_use
        let tool_use = &result[0];
        assert_eq!(tool_use.message_type, MessageType::Assistant);
        assert_eq!(tool_use.uuid, "call-789");
        match &tool_use.message.content {
            ContentValue::Structured(blocks) => {
                assert_eq!(blocks.len(), 1);
                match &blocks[0] {
                    ContentBlock::ToolUse { id, .. } => assert_eq!(id, "call-789"),
                    _ => panic!("Expected tool_use block"),
                }
            }
            _ => panic!("Expected structured content"),
        }

        // Second message: tool_result
        let tool_result = &result[1];
        assert_eq!(tool_result.message_type, MessageType::Assistant);
        assert_eq!(tool_result.parent_uuid, Some("call-789".to_string()));
        match &tool_result.message.content {
            ContentValue::Structured(blocks) => {
                assert_eq!(blocks.len(), 1);
                match &blocks[0] {
                    ContentBlock::ToolResult { tool_use_id, content, .. } => {
                        assert_eq!(tool_use_id, "call-789");
                        assert_eq!(content, "File contents here");
                    }
                    _ => panic!("Expected tool_result block"),
                }
            }
            _ => panic!("Expected structured content"),
        }
    }

    #[test]
    fn test_convert_info_message() {
        let entry = create_test_entry("info", json!({ "text": "Session started" }));
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", None).unwrap();

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.message_type, MessageType::Meta);
        assert_eq!(msg.is_meta, Some(true));
    }

    #[test]
    fn test_convert_unknown_type() {
        let entry = create_test_entry("weird_type", json!({ "text": "Unknown" }));
        let result = convert_timeline_entry_to_canonical(&entry, "session-1", None).unwrap();

        assert_eq!(result.len(), 1);
        let msg = &result[0];
        assert_eq!(msg.message_type, MessageType::Meta);

        let metadata = msg.provider_metadata.as_ref().unwrap();
        assert_eq!(metadata["copilot_type"], "weird_type");
        assert_eq!(metadata["warning"], "unknown_type");
    }
}
