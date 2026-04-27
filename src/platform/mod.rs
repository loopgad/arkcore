//! ArkCore 平台抽象层
//!
//! 提供平台差异化抽象，支持 Windows/Unix 等不同操作系统。

pub mod commands;
pub mod paths;
pub mod signals;
pub mod tty;

// 重新导出常用类型
pub use commands::{
    translate_command, shell_name, shell_args, command_separator, path_separator,
    CommandProvider, CommandTranslation, CustomCommandMappings, CurrentPlatformCommands,
    WindowsCommands, UnixCommands,
};
pub use paths::{
    config_dir, data_dir, cache_dir, temp_dir, home_dir, PlatformPaths, CurrentPlatformPaths,
    WindowsPaths, UnixPaths,
};
pub use signals::{
    Signal, SignalHandler, GlobalSignalState, signal_description, should_ignore,
    to_platform_signal, global_state,
};
pub use tty::{
    TerminalCapabilities, AnsiColorType, AnsiColor, AnsiSequence, TtyAdapter,
    supports_color, supports_true_color, is_tty,
};
