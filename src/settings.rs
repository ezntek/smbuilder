pub trait RunnableSettings {
    fn error(&self, text: &String);
    fn show_build_output(&self, text: &String);
    fn warn(&self, text: &String);
    fn log_info(&self, text: &String);
}
