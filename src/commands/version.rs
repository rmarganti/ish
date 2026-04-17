pub(crate) fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::version_output;

    #[test]
    fn version_output_uses_package_version() {
        assert_eq!(
            version_output(),
            format!("ish {}", env!("CARGO_PKG_VERSION"))
        );
    }
}
