/// A trait to represent a wrapper for
/// a settings instance.
///
/// It allows for core utility functions
/// (such as log functions) that depends
/// on a common set of settings to be
/// implemented in one place.
///
/// TODO: example
///
pub trait Log {
    /// Log an error.
    fn log_error(&self, text: &str);

    /// Log build output.
    fn log_build_output(&self, text: &str);

    /// Log a warning.
    fn log_warn(&self, text: &str);

    /// Log some information.
    fn log_info(&self, text: &str);
}
