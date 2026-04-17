pub(crate) fn version_output() -> String {
    format!("ish {}", env!("CARGO_PKG_VERSION"))
}
