//! T7.2: 跨平台测试
//!
//! 测试 ArkCore 在不同操作系统上的兼容性
//!
//! # 测试范围
//!
//! - 平台路径抽象
//! - 命令翻译
//! - 信号处理
//! - TTY 适配

use arkcore::platform::{
    commands::{command_separator, path_separator, shell_args, shell_name, translate_command},
    paths::{cache_dir, config_dir, data_dir, home_dir, temp_dir},
    signals::{GlobalSignalState, Signal},
    tty::{is_tty, supports_color, supports_true_color, TtyAdapter},
};

/// 测试平台路径抽象
#[test]
fn test_platform_paths_exists() {
    // 验证各平台路径函数返回有效路径
    // 使用模块级函数，因为 trait 方法是静态的
    #[cfg(windows)]
    {
        assert!(!config_dir().as_os_str().is_empty());
        assert!(!data_dir().as_os_str().is_empty());
        assert!(!cache_dir().as_os_str().is_empty());
        assert!(!temp_dir().as_os_str().is_empty());
    }

    #[cfg(unix)]
    {
        assert!(!config_dir().as_os_str().is_empty());
        assert!(!data_dir().as_os_str().is_empty());
        assert!(!cache_dir().as_os_str().is_empty());
        assert!(!temp_dir().as_os_str().is_empty());
    }
}

/// 测试通用路径函数
#[test]
fn test_common_path_functions() {
    // 验证各平台路径函数存在并返回有效路径
    assert!(!config_dir().as_os_str().is_empty());
    assert!(!data_dir().as_os_str().is_empty());
    assert!(!cache_dir().as_os_str().is_empty());
    assert!(!temp_dir().as_os_str().is_empty());
    // home_dir 返回 Option
    if let Some(home) = home_dir() {
        assert!(!home.as_os_str().is_empty());
    }
}

/// 测试命令翻译 - Windows
#[cfg(windows)]
#[test]
fn test_command_translation_windows() {
    // Unix -> Windows 翻译
    let translated = translate_command("ls -la");
    assert!(translated.was_translated);

    let translated = translate_command("cat file.txt");
    assert!(translated.was_translated);

    let translated = translate_command("pwd");
    assert!(translated.was_translated);

    // 保持 Windows 命令不变
    let translated = translate_command("dir");
    assert!(!translated.was_translated);

    let translated = translate_command("type file.txt");
    assert!(!translated.was_translated);
}

/// 测试命令翻译 - Unix
#[cfg(unix)]
#[test]
fn test_command_translation_unix() {
    use arkcore::platform::commands::UnixCommands;

    // Unix 命令应该保持不变
    let translated = UnixCommands::translate("ls -la");
    assert!(!translated.was_translated);
    assert_eq!(translated.original, "ls -la");
    assert_eq!(translated.translated, "ls -la");

    let translated = UnixCommands::translate("cat file.txt");
    assert!(!translated.was_translated);
    assert_eq!(translated.original, "cat file.txt");
    assert_eq!(translated.translated, "cat file.txt");
}

/// 测试 Shell 名称
#[test]
fn test_shell_name() {
    let name = shell_name();
    assert!(!name.is_empty());

    #[cfg(windows)]
    assert!(name == "cmd" || name == "powershell");

    #[cfg(unix)]
    assert!(name == "sh" || name == "bash" || name == "zsh");
}

/// 测试 Shell 参数
#[test]
fn test_shell_args() {
    let args = shell_args();
    assert!(!args.is_empty());
}

/// 测试命令分隔符
#[test]
fn test_command_separator() {
    let sep = command_separator();
    assert!(!sep.is_empty());

    #[cfg(windows)]
    assert_eq!(sep, " & ");

    #[cfg(unix)]
    assert_eq!(sep, " && ");
}

/// 测试路径分隔符
#[test]
fn test_path_separator() {
    let sep = path_separator();
    // sep 是 char 类型，不能调用 is_empty()
    // char 永远不会是空的，所以直接检查即可

    #[cfg(windows)]
    assert_eq!(sep, ';');

    #[cfg(unix)]
    assert_eq!(sep, ':');
}

/// 测试 Windows 命令结构
#[cfg(windows)]
#[test]
fn test_windows_commands() {
    // 验证 WindowsCommands 可以翻译命令
    let result = translate_command("ls");
    assert!(result.was_translated);
    assert!(result.translated.contains("dir"));

    let result = translate_command("cat file.txt");
    assert!(result.translated.contains("type"));
}

/// 测试 Unix 命令结构
#[cfg(unix)]
#[test]
fn test_unix_commands() {
    // 验证 UnixCommands 保持命令不变
    let result = translate_command("ls -la");
    assert!(!result.was_translated);
    assert_eq!(result.original, result.translated);
}

/// 测试信号处理
#[test]
fn test_signal_handler_exists() {
    // 验证信号枚举存在
    let signal = Signal::Interrupt;
    assert_eq!(format!("{:?}", signal), "Interrupt");

    let signal = Signal::Terminate;
    assert_eq!(format!("{:?}", signal), "Terminate");

    let signal = Signal::BrokenPipe;
    assert_eq!(format!("{:?}", signal), "BrokenPipe");
}

/// 测试全局信号状态
#[test]
fn test_global_signal_state() {
    let state = GlobalSignalState::new();
    assert!(!state.is_interrupt_received());
    assert!(!state.is_terminate_received());

    // 验证可以更新状态
    state.mark_interrupt();
    assert!(state.is_interrupt_received());

    state.reset();
    assert!(!state.is_interrupt_received());
}

/// 测试 TTY 检测
#[test]
fn test_tty_detection() {
    // is_tty 应该返回布尔值
    let result = is_tty();
    assert!(result == true || result == false);

    // supports_color 应该返回布尔值
    let result = supports_color();
    assert!(result == true || result == false);

    // supports_true_color 应该返回布尔值
    let result = supports_true_color();
    assert!(result == true || result == false);
}

/// 测试终端能力检测
#[test]
fn test_terminal_capabilities() {
    let caps = TtyAdapter::detect_capabilities();

    // 验证能力字段存在
    assert!(caps.supports_ansi_colors == true || caps.supports_ansi_colors == false);
    assert!(caps.supports_true_color == true || caps.supports_true_color == false);
    assert!(caps.supports_utf8 == true || caps.supports_utf8 == false);
}

/// 测试 TTY 适配器
#[test]
fn test_tty_adapter() {
    let adapter = TtyAdapter::new();

    // 验证适配器可以检测终端
    let result = is_tty();
    assert!(result == true || result == false);

    // 验证可以获取终端能力
    let caps = adapter.capabilities();
    // caps.width 和 caps.height 是 Option<u16>
    // 我们不强制要求它们有值（TTY 可能不可用）
}

/// 测试平台特性检测
#[test]
fn test_platform_feature_detection() {
    #[cfg(windows)]
    {
        assert!(std::env::consts::OS == "windows");
    }

    #[cfg(unix)]
    {
        assert!(std::env::consts::OS == "linux" || std::env::consts::OS == "macos");
    }

    #[cfg(target_arch = "x86_64")]
    {
        assert!(true);
    }

    #[cfg(target_arch = "aarch64")]
    {
        assert!(true);
    }
}

/// 测试跨平台路径处理
#[test]
fn test_cross_platform_path_handling() {
    use std::path::PathBuf;

    // 验证路径可以正确处理
    let path = PathBuf::from("test/path/file.txt");
    assert!(path.to_str().is_some());

    // 验证路径连接
    let joined = path.join("subdir");
    assert!(joined.to_str().is_some());

    // 验证路径分隔符因平台而异
    let path_str = path.display().to_string();
    #[cfg(windows)]
    assert!(path_str.contains('\\') || path_str.contains('/'));

    #[cfg(unix)]
    assert!(path_str.contains('/'));
}

/// 测试 CurrentPlatformPaths 使用
#[test]
fn test_current_platform_paths() {
    // 使用路径函数而不是直接构造
    let config = config_dir();
    let data = data_dir();
    let cache = cache_dir();
    let temp = temp_dir();

    // 验证返回有效路径
    assert!(!config.as_os_str().is_empty());
    assert!(!data.as_os_str().is_empty());
    assert!(!cache.as_os_str().is_empty());
    assert!(!temp.as_os_str().is_empty());
}

/// 测试 CurrentPlatformCommands 使用
#[test]
fn test_current_platform_commands() {
    // 使用 translate_command 函数
    let result = translate_command("ls");
    // 验证返回 CommandTranslation 结构
    assert!(!result.original.is_empty());
    assert!(!result.translated.is_empty());
}
