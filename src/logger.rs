pub struct Logger {}

pub enum LogLevel {
    Whisper,
    Chant,
    Omen,
    Curse,
}

/// logs based on log level, the message will be printed with a different prefix to indicate the severity of the log.
pub fn log(level: LogLevel, message: &str) {
    match level {
        LogLevel::Whisper => whisper(message),
        LogLevel::Chant => chant(message),
        LogLevel::Omen => omen(message),
        LogLevel::Curse => curse(message),
    };
}

/// WHISPER - for informational messages that are not critical but might be useful for debugging or understanding the flow of execution.
pub fn whisper(message: &str) {
    println!("Whisper: {}", message);
}

/// Chant or INFO - for messages that indicate informational events.
pub fn chant(message: &str) {
    println!("Chant: {}", message);
}
/// Omen or WARN - for warnings that don't prevent execution but might indicate potential issues.
pub fn omen(message: &str) {
    println!("Omen: {}", message);
}

/// Curse or ERROR - for critical issues that prevent execution or indicate severe problems.
pub fn curse(message: &str) {
    println!("Curse: {}", message);
}
