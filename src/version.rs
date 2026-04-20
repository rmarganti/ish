pub(crate) const BUILD_VERSION: &str = match option_env!("ISH_BUILD_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

pub(crate) fn display_version() -> String {
    format!("ish {BUILD_VERSION}")
}

#[cfg(test)]
mod tests {
    use super::{BUILD_VERSION, display_version};

    #[test]
    fn build_version_uses_injected_version_or_package_version() {
        assert!(!BUILD_VERSION.trim().is_empty());
    }

    #[test]
    fn display_version_prefixes_the_build_version() {
        assert_eq!(display_version(), format!("ish {BUILD_VERSION}"));
    }
}
