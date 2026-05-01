//! ArkCore 平台抽象层
//!
//! 提供平台差异化抽象，支持 Windows/Unix 等不同操作系统。

pub mod commands;
pub mod paths;
pub mod signals;
pub mod tty;

// 重新导出常用类型
pub use commands::{
    command_separator, path_separator, shell_args, shell_name, translate_command, CommandProvider,
    CommandTranslation, CurrentPlatformCommands, CustomCommandMappings, UnixCommands,
    WindowsCommands,
};
pub use paths::{
    cache_dir, config_dir, data_dir, home_dir, temp_dir, CurrentPlatformPaths, PlatformPaths,
    UnixPaths, WindowsPaths,
};
pub use signals::{
    global_state, should_ignore, signal_description, to_platform_signal, GlobalSignalState, Signal,
    SignalHandler,
};
pub use tty::{
    is_tty, supports_color, supports_true_color, AnsiColor, AnsiColorType, AnsiSequence,
    TerminalCapabilities, TtyAdapter,
};
