use crate::config::Config;
use crate::model::ishoo::IshooJson;
use colored::{Color, Colorize};
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

#[allow(dead_code)]
pub fn render_status(config: &Config, status: &str) -> String {
    let rendered = render_badge(status, config.get_status(status).map(|status| status.color));
    if config.is_archive_status(status) {
        rendered.dimmed().to_string()
    } else {
        rendered.to_string()
    }
}

#[allow(dead_code)]
pub fn render_type(config: &Config, ishoo_type: &str) -> String {
    render_badge(
        ishoo_type,
        config
            .get_type(ishoo_type)
            .map(|ishoo_type| ishoo_type.color),
    )
    .to_string()
}

#[allow(dead_code)]
pub fn render_priority(config: &Config, priority: &str) -> String {
    render_badge(
        priority,
        config.get_priority(priority).map(|priority| priority.color),
    )
    .to_string()
}

#[allow(dead_code)]
pub fn render_id(id: &str) -> String {
    id.bold().white().to_string()
}

#[allow(dead_code)]
pub fn muted(text: &str) -> String {
    text.dimmed().to_string()
}

#[allow(dead_code)]
pub fn heading(text: &str) -> String {
    text.bold().to_string()
}

pub fn success(text: &str) -> String {
    text.green().bold().to_string()
}

pub fn danger(text: &str) -> String {
    text.red().bold().to_string()
}

pub fn warning(text: &str) -> String {
    text.yellow().bold().to_string()
}

fn render<T: Serialize>(response: Response<T>) -> Result<String, String> {
    serde_json::to_string_pretty(&response)
        .map_err(|error| format!("failed to serialize JSON output: {error}"))
}

#[cfg_attr(not(test), allow(dead_code))]
fn render_badge(label: &str, color_name: Option<&str>) -> colored::ColoredString {
    let badge = format!("[{label}]");
    match color_name.and_then(color_name_to_color) {
        Some(color) => badge.color(color).bold(),
        None => badge.bold(),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn color_name_to_color(color_name: &str) -> Option<Color> {
    match color_name {
        "red" => Some(Color::Red),
        "yellow" => Some(Color::Yellow),
        "green" => Some(Color::Green),
        "blue" => Some(Color::Blue),
        "purple" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" => Some(Color::BrightBlack),
        "white" => Some(Color::White),
        _ => None,
    }
}

pub(crate) fn is_supported_color_name(color_name: &str) -> bool {
    color_name_to_color(color_name).is_some()
}

#[cfg(test)]
mod tests {
    use super::{
        ErrorCode, color_name_to_color, danger, heading, muted, output_error, output_message,
        output_success, output_success_multiple, render_id, render_priority, render_status,
        render_type, success, warning,
    };
    use crate::config::Config;
    use crate::model::ishoo::Ishoo;
    use chrono::{TimeZone, Utc};
    use colored::{Color, control};
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

    #[test]
    fn color_name_mapping_matches_config_palette() {
        assert_eq!(color_name_to_color("red"), Some(Color::Red));
        assert_eq!(color_name_to_color("yellow"), Some(Color::Yellow));
        assert_eq!(color_name_to_color("green"), Some(Color::Green));
        assert_eq!(color_name_to_color("blue"), Some(Color::Blue));
        assert_eq!(color_name_to_color("purple"), Some(Color::Magenta));
        assert_eq!(color_name_to_color("cyan"), Some(Color::Cyan));
        assert_eq!(color_name_to_color("gray"), Some(Color::BrightBlack));
        assert_eq!(color_name_to_color("white"), Some(Color::White));
        assert_eq!(color_name_to_color("unknown"), None);
    }

    #[test]
    fn render_helpers_apply_expected_labels_and_styles() {
        let config = Config::default();
        let previous = control::SHOULD_COLORIZE.should_colorize();
        control::set_override(true);

        let active_status = render_status(&config, "todo");
        let archive_status = render_status(&config, "completed");
        let rendered_type = render_type(&config, "task");
        let rendered_priority = render_priority(&config, "high");
        let rendered_id = render_id("ish-abcd");
        let rendered_muted = muted("secondary text");
        let rendered_heading = heading("Heading");
        let rendered_success = success("deleted");
        let rendered_danger = danger("failed");
        let rendered_warning = warning("careful");

        control::set_override(previous);

        assert!(active_status.contains("[todo]"));
        assert!(archive_status.contains("[completed]"));
        assert_ne!(archive_status, active_status);
        assert!(archive_status.contains("\u{1b}["));
        assert!(rendered_type.contains("[task]"));
        assert!(rendered_priority.contains("[high]"));
        assert!(rendered_id.contains("ish-abcd"));
        assert!(rendered_muted.contains("secondary text"));
        assert!(rendered_heading.contains("Heading"));
        assert!(rendered_success.contains("deleted"));
        assert!(rendered_danger.contains("failed"));
        assert!(rendered_warning.contains("careful"));
    }
}
