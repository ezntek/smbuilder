const ERR_PREFIX: &str = "\u{1b}[1;31merror: \u{1b}[0m";
const BUILD_PREFIX: &str = "\u{1b}[1;34mbuild: \u{1b}[0m";
const WARN_PREFIX: &str = "\u{1b}[1;33mwarn: \u{1b}[0m";
const INFO_PREFIX: &str = "\u{1b}[1;34minfo: \u{1b}[0m";

pub enum LogLevel {
    ErrorsOnly,
    BuildOutput,
    Default,
    Noisy,
}

impl From<LogLevel> for u8 {
    fn from(val: LogLevel) -> u8 {
        use LogLevel::*;

        match val {
            ErrorsOnly => 0,
            BuildOutput => 1,
            Default => 2,
            Noisy => 3,
        }
    }
}

impl From<u8> for LogLevel {
    fn from(value: u8) -> Self {
        use LogLevel::*;

        match value {
            0 => ErrorsOnly,
            1 => BuildOutput,
            2 => Default,
            3 => Noisy,
            _ => Default,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CmdoutSettings {
    Silent,
    LogProgress { log_level: u8 },
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub cmdout_settings: CmdoutSettings,
}

pub struct RunnableSettings {
    settings: Settings,
}

enum SettingsAction {
    Error,
    OutputBuildOutput,
    Warn,
    Log,
}

impl Settings {
    pub fn get_runnable(self) -> RunnableSettings {
        RunnableSettings { settings: self }
    }
}

impl RunnableSettings {
    fn should_perform_action(&self, action: SettingsAction) -> bool {
        use SettingsAction::*;

        let log_level_num =
            if let CmdoutSettings::LogProgress { log_level } = self.settings.cmdout_settings {
                log_level
            } else {
                return false;
            };

        match action {
            Error => log_level_num != 0,
            OutputBuildOutput => log_level_num >= 1,
            Warn => log_level_num >= 2,
            Log => log_level_num == 3,
        }
    }

    pub fn error<S: AsRef<str>>(&self, text: S) {
        if self.should_perform_action(SettingsAction::Error) {
            println!("{}{}", ERR_PREFIX, text.as_ref());
        }
    }

    pub fn print_build_output<S: AsRef<str>>(&self, text: S) {
        if self.should_perform_action(SettingsAction::OutputBuildOutput) {
            println!("{}{}", BUILD_PREFIX, text.as_ref());
        }
    }

    pub fn warn<S: AsRef<str>>(&self, text: S) {
        if self.should_perform_action(SettingsAction::Warn) {
            println!("{}{}", WARN_PREFIX, text.as_ref());
        }
    }

    pub fn log<S: AsRef<str>>(&self, text: S) {
        if self.should_perform_action(SettingsAction::Log) {
            println!("{}{}", INFO_PREFIX, text.as_ref());
        }
    }
}
