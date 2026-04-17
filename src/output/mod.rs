use crate::model::ishoo::IshooJson;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound,
    Validation,
    Conflict,
    FileError,
}

impl ErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            ErrorCode::NotFound => "not_found",
            ErrorCode::Validation => "validation",
            ErrorCode::Conflict => "conflict",
            ErrorCode::FileError => "file_error",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Response<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ishoos: Option<Vec<IshooJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
}

pub fn output_success<T: Serialize>(data: T) -> Result<String, String> {
    render(Response {
        success: true,
        message: None,
        data: Some(data),
        ishoos: None,
        count: None,
        code: None,
    })
}

#[allow(dead_code)]
pub fn output_success_multiple(ishoos: Vec<IshooJson>) -> Result<String, String> {
    let count = ishoos.len();
    render(Response::<()> {
        success: true,
        message: None,
        data: None,
        ishoos: Some(ishoos),
        count: Some(count),
        code: None,
    })
}

pub fn output_message(message: impl Into<String>) -> Result<String, String> {
    render(Response::<()> {
        success: true,
        message: Some(message.into()),
        data: None,
        ishoos: None,
        count: None,
        code: None,
    })
}

pub fn output_error(code: ErrorCode, message: impl Into<String>) -> String {
    render(Response::<()> {
        success: false,
        message: Some(message.into()),
        data: None,
        ishoos: None,
        count: None,
        code: Some(code.as_str()),
    })
    .expect("error responses should serialize")
}

fn render<T: Serialize>(response: Response<T>) -> Result<String, String> {
    serde_json::to_string_pretty(&response)
        .map_err(|error| format!("failed to serialize JSON output: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, output_error, output_message, output_success, output_success_multiple};
    use crate::model::ishoo::Ishoo;
    use chrono::{TimeZone, Utc};
    use serde_json::{Value, json};

    fn sample_ishoo_json(id: &str) -> crate::model::ishoo::IshooJson {
        Ishoo {
            id: id.to_string(),
            slug: "sample".to_string(),
            path: format!("{id}--sample.md"),
            title: "Sample".to_string(),
            status: "todo".to_string(),
            ishoo_type: "task".to_string(),
            priority: Some("normal".to_string()),
            tags: vec!["backend".to_string()],
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap(),
            order: None,
            body: "Body".to_string(),
            parent: None,
            blocking: Vec::new(),
            blocked_by: Vec::new(),
        }
        .to_json("etag-1")
    }

    #[test]
    fn output_success_wraps_structured_data() {
        let rendered = output_success(json!({"version": "0.1.0"})).expect("json should render");
        let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(
            parsed["data"]["version"],
            Value::String("0.1.0".to_string())
        );
        assert!(parsed.get("message").is_none());
    }

    #[test]
    fn output_success_multiple_includes_count() {
        let rendered =
            output_success_multiple(vec![sample_ishoo_json("ish-a1")]).expect("json should render");
        let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["count"], Value::from(1));
        assert_eq!(
            parsed["ishoos"][0]["id"],
            Value::String("ish-a1".to_string())
        );
    }

    #[test]
    fn output_error_includes_code_and_message() {
        let parsed: Value =
            serde_json::from_str(&output_error(ErrorCode::Conflict, "etag mismatch"))
                .expect("json should parse");

        assert_eq!(parsed["success"], Value::Bool(false));
        assert_eq!(parsed["code"], Value::String("conflict".to_string()));
        assert_eq!(
            parsed["message"],
            Value::String("etag mismatch".to_string())
        );
    }

    #[test]
    fn output_message_uses_message_field() {
        let rendered = output_message("ready").expect("json should render");
        let parsed: Value = serde_json::from_str(&rendered).expect("json should parse");

        assert_eq!(parsed["success"], Value::Bool(true));
        assert_eq!(parsed["message"], Value::String("ready".to_string()));
        assert!(parsed.get("data").is_none());
    }

    #[test]
    fn error_code_strings_match_cli_contract() {
        assert_eq!(ErrorCode::NotFound.as_str(), "not_found");
        assert_eq!(ErrorCode::Validation.as_str(), "validation");
        assert_eq!(ErrorCode::Conflict.as_str(), "conflict");
        assert_eq!(ErrorCode::FileError.as_str(), "file_error");
    }
}
