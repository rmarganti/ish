pub(crate) fn version_output() -> String {
    crate::version::display_version()
}

#[cfg(test)]
mod tests {
    use super::version_output;
    use crate::version::BUILD_VERSION;

    #[test]
    fn version_output_uses_build_version() {
        assert_eq!(version_output(), format!("ish {BUILD_VERSION}"));
    }
}
