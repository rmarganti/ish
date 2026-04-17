use crate::config::Config;
use crate::core::store::LinkCheckResult;
use crate::output::is_supported_color_name;

#[derive(Debug, Clone)]
pub(super) struct ConfigChecks {
    pub(super) default_status: Option<String>,
    pub(super) default_type: Option<String>,
    pub(super) invalid_colors: Vec<String>,
}

impl ConfigChecks {
    pub(super) fn issue_count(&self) -> usize {
        usize::from(self.default_status.is_some())
            + usize::from(self.default_type.is_some())
            + self.invalid_colors.len()
    }
}

pub(super) fn validate_config(config: &Config) -> ConfigChecks {
    let mut invalid_colors = Vec::new();

    for status_name in config.status_names() {
        if let Some(status) = config.get_status(status_name)
            && !is_supported_color_name(status.color)
        {
            invalid_colors.push(format!(
                "status `{}` uses unsupported color `{}`",
                status.name, status.color
            ));
        }
    }

    for type_name in config.type_names() {
        if let Some(ishoo_type) = config.get_type(type_name)
            && !is_supported_color_name(ishoo_type.color)
        {
            invalid_colors.push(format!(
                "type `{}` uses unsupported color `{}`",
                ishoo_type.name, ishoo_type.color
            ));
        }
    }

    for priority_name in config.priority_names() {
        if let Some(priority) = config.get_priority(priority_name)
            && !is_supported_color_name(priority.color)
        {
            invalid_colors.push(format!(
                "priority `{}` uses unsupported color `{}`",
                priority.name, priority.color
            ));
        }
    }

    ConfigChecks {
        default_status: (!config.is_valid_status(&config.ish.default_status))
            .then(|| format!("invalid default_status `{}`", config.ish.default_status)),
        default_type: (!config.is_valid_type(&config.ish.default_type))
            .then(|| format!("invalid default_type `{}`", config.ish.default_type)),
        invalid_colors,
    }
}

pub(super) fn link_issue_count(result: &LinkCheckResult) -> usize {
    result.broken_links.len() + result.self_links.len() + result.cycles.len()
}
