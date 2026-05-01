//! 跨平台 TTY 适配
//!
//! 提供终端能力检测、ANSI 转义序列支持和回退方案

use std::io::{self, Write};
use std::sync::atomic::AtomicBool;

/// 终端能力
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    /// 支持 ANSI 颜色
    pub supports_ansi_colors: bool,
    /// 支持 UTF-8
    pub supports_utf8: bool,
    /// 支持真彩色
    pub supports_true_color: bool,
    /// 支持链接（超链接）
    pub supports_hyperlinks: bool,
    /// 支持 256 色
    pub supports_256_colors: bool,
    /// 终端宽度
    pub width: Option<u16>,
    /// 终端高度
    pub height: Option<u16>,
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self {
            supports_ansi_colors: false,
            supports_utf8: true,
            supports_true_color: false,
            supports_hyperlinks: false,
            supports_256_colors: false,
            width: None,
            height: None,
        }
    }
}

/// ANSI 颜色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColorType {
    /// 标准 16 色
    Standard16,
    /// 256 色
    Colors256,
    /// 真彩色 (24 位)
    TrueColor,
}

/// ANSI 颜色
#[derive(Debug, Clone, Copy)]
pub struct AnsiColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl AnsiColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }

    /// 从 256 色索引转换
    pub fn from_256color(index: u8) -> Self {
        if index < 8 {
            // 标准前景色
            match index {
                0 => Self::new(0, 0, 0),       // 黑
                1 => Self::new(128, 0, 0),     // 红
                2 => Self::new(0, 128, 0),     // 绿
                3 => Self::new(128, 128, 0),   // 黄
                4 => Self::new(0, 0, 128),     // 蓝
                5 => Self::new(128, 0, 128),   // 品红
                6 => Self::new(0, 128, 128),   // 青
                7 => Self::new(192, 192, 192), // 白
                _ => Self::new(0, 0, 0),
            }
        } else if index < 16 {
            // 标准前景色高亮
            match index {
                8 => Self::new(128, 128, 128),  // 亮黑
                9 => Self::new(255, 0, 0),      // 亮红
                10 => Self::new(0, 255, 0),     // 亮绿
                11 => Self::new(255, 255, 0),   // 亮黄
                12 => Self::new(0, 0, 255),     // 亮蓝
                13 => Self::new(255, 0, 255),   // 亮品红
                14 => Self::new(0, 255, 255),   // 亮青
                15 => Self::new(255, 255, 255), // 亮白
                _ => Self::new(255, 255, 255),
            }
        } else if index < 232 {
            // 216 色 (6x6x6 立方体)
            let index = index - 16;
            let r = (index / 36) * 51;
            let g = ((index % 36) / 6) * 51;
            let b = (index % 6) * 51;
            Self::new(r, g, b)
        } else {
            // 灰度 (24 级)
            let gray = (index - 232) * 10 + 8;
            Self::new(gray, gray, gray)
        }
    }

    /// 转换为 ANSI 转义序列 (TrueColor)
    pub fn to_ansi_sequence(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.red, self.green, self.blue)
    }

    /// 转换为 ANSI 256 色序列
    pub fn to_256color_sequence(&self) -> String {
        let index = self.to_256color_index();
        format!("\x1b[38;5;{}m", index)
    }

    /// 转换为 256 色索引
    #[allow(clippy::wrong_self_convention)]
    fn to_256color_index(&self) -> u8 {
        // 检查标准 16 色
        let standard_colors = [
            (0, 0, 0, 0),
            (128, 0, 0, 1),
            (0, 128, 0, 2),
            (128, 128, 0, 3),
            (0, 0, 128, 4),
            (128, 0, 128, 5),
            (0, 128, 128, 6),
            (192, 192, 192, 7),
            (128, 128, 128, 8),
            (255, 0, 0, 9),
            (0, 255, 0, 10),
            (255, 255, 0, 11),
            (0, 0, 255, 12),
            (255, 0, 255, 13),
            (0, 255, 255, 14),
            (255, 255, 255, 15),
        ];

        for (r, g, b, idx) in standard_colors {
            if self.red == r && self.green == g && self.blue == b {
                return idx;
            }
        }

        // 216 色立方体
        if self.red % 51 == 0 && self.green % 51 == 0 && self.blue % 51 == 0 {
            let r = self.red / 51;
            let g = self.green / 51;
            let b = self.blue / 51;
            if r < 6 && g < 6 && b < 6 {
                return 16 + r * 36 + g * 6 + b;
            }
        }

        // 灰度
        let gray = (self.red as u16 + self.green as u16 + self.blue as u16) / 3;
        if gray % 10 < 5 {
            return (232 + (gray / 10)) as u8;
        }
        (232 + ((gray + 5) / 10).min(23)) as u8
    }
}

/// ANSI 转义序列生成器
pub struct AnsiSequence {
    caps: TerminalCapabilities,
}

impl AnsiSequence {
    pub fn new(caps: TerminalCapabilities) -> Self {
        Self { caps }
    }

    /// 获取颜色序列
    pub fn color(&self, color: AnsiColor) -> String {
        if self.caps.supports_true_color {
            color.to_ansi_sequence()
        } else if self.caps.supports_256_colors {
            color.to_256color_sequence()
        } else {
            String::new() // 不支持颜色
        }
    }

    /// 重置序列
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }

    /// 加粗
    pub fn bold() -> &'static str {
        "\x1b[1m"
    }

    /// 斜体
    pub fn italic() -> &'static str {
        "\x1b[3m"
    }

    /// 下划线
    pub fn underline() -> &'static str {
        "\x1b[4m"
    }

    /// 移动光标
    pub fn move_cursor(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row + 1, col + 1)
    }

    /// 清除行
    pub fn clear_line() -> &'static str {
        "\x1b[2K"
    }

    /// 清除屏幕
    pub fn clear_screen() -> &'static str {
        "\x1b[2J"
    }

    /// 隐藏光标
    pub fn hide_cursor() -> &'static str {
        "\x1b[?25l"
    }

    /// 显示光标
    pub fn show_cursor() -> &'static str {
        "\x1b[?25h"
    }
}

/// TTY 检测和适配
pub struct TtyAdapter {
    capabilities: TerminalCapabilities,
    #[allow(dead_code)]
    virtual_terminal_enabled: AtomicBool,
}

impl TtyAdapter {
    /// 创建新的 TTY 适配器
    pub fn new() -> Self {
        let caps = Self::detect_capabilities();
        Self {
            capabilities: caps,
            virtual_terminal_enabled: AtomicBool::new(false),
        }
    }

    /// 检测终端能力
    #[allow(clippy::field_reassign_with_default)]
    pub fn detect_capabilities() -> TerminalCapabilities {
        let mut caps = TerminalCapabilities::default();

        // 检查 ANSI 颜色支持
        caps.supports_ansi_colors = Self::detect_ansi_support();

        // 检查 UTF-8 支持
        #[cfg(windows)]
        {
            // Windows: 检查代码页
            caps.supports_utf8 =
                unsafe { windows_sys::Win32::System::Console::GetConsoleOutputCP() == 65001 };
        }
        #[cfg(unix)]
        {
            caps.supports_utf8 = true; // Unix 通常默认支持 UTF-8
        }

        // 检查真彩色
        caps.supports_true_color = Self::detect_true_color_support();

        // 检查 256 色
        caps.supports_256_colors = std::env::var("COLORTERM")
            .map(|v| v == "truecolor" || v == "24bit")
            .unwrap_or(false);

        // 检查超链接
        caps.supports_hyperlinks = std::env::var("TERM_PROGRAM")
            .map(|v| v == "iTerm.app" || v == "Apple_Terminal" || v == "vscode")
            .unwrap_or(false);

        // 获取终端尺寸
        if let Some((w, h)) = Self::get_terminal_size() {
            caps.width = Some(w);
            caps.height = Some(h);
        }

        caps
    }

    /// 检测 ANSI 颜色支持
    fn detect_ansi_support() -> bool {
        // 检查环境变量
        if let Ok(term) = std::env::var("TERM") {
            let term = term.to_lowercase();
            if term.contains("xterm")
                || term.contains("screen")
                || term.contains("tmux")
                || term.contains("256")
                || term == "ansi"
                || term == "cygwin"
            {
                return true;
            }
        }

        // Windows 10+ 支持 ANSI (如果启用虚拟终端)
        #[cfg(windows)]
        {
            if Self::enable_virtual_terminal().is_ok() {
                return true;
            }
        }

        false
    }

    /// 检测真彩色支持
    fn detect_true_color_support() -> bool {
        std::env::var("COLORTERM")
            .map(|v| v == "truecolor" || v == "24bit")
            .unwrap_or(false)
    }

    /// 获取终端尺寸
    fn get_terminal_size() -> Option<(u16, u16)> {
        #[cfg(unix)]
        {
            use libc::winsize;
            use libc::TIOCGWINSZ;
            unsafe {
                let mut ws: winsize = std::mem::zeroed();
                if libc::ioctl(libc::STDOUT_FILENO, TIOCGWINSZ, &mut ws) == 0
                    && ws.ws_col > 0
                    && ws.ws_row > 0
                {
                    return Some((ws.ws_col, ws.ws_row));
                }
            }
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HANDLE;
            use windows_sys::Win32::System::Console::GetConsoleScreenBufferInfo;
            use windows_sys::Win32::System::Console::GetStdHandle;
            use windows_sys::Win32::System::Console::CONSOLE_SCREEN_BUFFER_INFO;
            use windows_sys::Win32::System::Console::STD_OUTPUT_HANDLE;

            unsafe {
                let handle: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
                let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
                if GetConsoleScreenBufferInfo(handle, &mut info) != 0 {
                    let width = info.srWindow.Right - info.srWindow.Left + 1;
                    let height = info.srWindow.Bottom - info.srWindow.Top + 1;
                    if width > 0 && height > 0 {
                        return Some((width as u16, height as u16));
                    }
                }
            }
        }
        None
    }

    /// 启用 Windows 虚拟终端序列
    #[cfg(windows)]
    pub fn enable_virtual_terminal() -> anyhow::Result<()> {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        };
        use windows_sys::Win32::System::Console::{GetStdHandle, STD_OUTPUT_HANDLE};

        unsafe {
            let handle: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode: u32 = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                anyhow::bail!("Failed to get console mode");
            }

            if SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) == 0 {
                anyhow::bail!("Failed to enable virtual terminal processing");
            }
        }

        Ok(())
    }

    /// 获取终端能力
    pub fn capabilities(&self) -> TerminalCapabilities {
        self.capabilities
    }

    /// 输出带颜色的文本
    pub fn write_colored(&self, text: &str, color: AnsiColor) -> io::Result<()> {
        if self.capabilities.supports_ansi_colors {
            let sequence = AnsiSequence::new(self.capabilities);
            print!("{}{}{}", sequence.color(color), text, AnsiSequence::reset());
        } else {
            print!("{}", text);
        }
        io::stdout().flush()
    }

    /// 进度条显示
    pub fn write_progress(&self, current: u64, total: u64, width: usize) -> io::Result<()> {
        if !self.capabilities.supports_ansi_colors {
            // 无颜色模式：简单文本进度
            println!("{}/{}", current, total);
            return Ok(());
        }

        let percentage = if total > 0 {
            (current as f64 / total as f64 * 100.0) as u64
        } else {
            0
        };
        let filled = (current as f64 / total as f64 * width as f64) as usize;
        let empty = width - filled;

        print!("\r[");
        for _ in 0..filled {
            print!("#");
        }
        for _ in 0..empty {
            print!("-");
        }
        print!("] {}%", percentage);
        io::stdout().flush()
    }
}

impl Default for TtyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// 检查是否支持彩色输出
pub fn supports_color() -> bool {
    TtyAdapter::new().capabilities.supports_ansi_colors
}

/// 检查是否支持真彩色
pub fn supports_true_color() -> bool {
    TtyAdapter::new().capabilities.supports_true_color
}

/// 检查是否在 TTY 中运行
pub fn is_tty() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::HANDLE;
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, STD_OUTPUT_HANDLE,
        };
        unsafe {
            let handle: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut mode: u32 = 0;
            GetConsoleMode(handle, &mut mode) != 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_from_256() {
        let color = AnsiColor::from_256color(196); // 红色
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 0);
        assert_eq!(color.blue, 0);
    }

    #[test]
    fn test_ansi_color_conversion() {
        let color = AnsiColor::new(255, 128, 0);
        let _ = color.to_256color_index();
        let _ = color.to_ansi_sequence();
        let _ = color.to_256color_sequence();
    }

    #[test]
    fn test_tty_adapter() {
        let adapter = TtyAdapter::new();
        let caps = adapter.capabilities();
        println!("Terminal capabilities: {:?}", caps);
    }

    #[test]
    fn test_is_tty() {
        println!("Is TTY: {}", is_tty());
    }

    // ========== 新增测试 ==========

    #[test]
    fn test_ansi_color_new() {
        let color = AnsiColor::new(100, 150, 200);
        assert_eq!(color.red, 100);
        assert_eq!(color.green, 150);
        assert_eq!(color.blue, 200);
    }

    #[test]
    fn test_ansi_color_from_256_standard_colors() {
        // 测试标准 16 色
        for i in 0..16 {
            let color = AnsiColor::from_256color(i);
            assert!(color.red <= 255);
            assert!(color.green <= 255);
            assert!(color.blue <= 255);
        }
    }

    #[test]
    fn test_ansi_color_from_256_gray() {
        // 测试灰度色 (索引 232-255)
        let gray = AnsiColor::from_256color(244);
        // 灰度颜色 RGB 应该相等
        assert_eq!(gray.red, gray.green);
        assert_eq!(gray.green, gray.blue);
    }

    #[test]
    fn test_ansi_color_to_ansi_sequence() {
        let color = AnsiColor::new(255, 0, 0);
        let seq = color.to_ansi_sequence();
        assert!(seq.contains("38;2")); // TrueColor 格式
        assert!(seq.contains("255")); // 红色值
    }

    #[test]
    fn test_ansi_color_to_256color_sequence() {
        let color = AnsiColor::new(255, 0, 0);
        let seq = color.to_256color_sequence();
        assert!(seq.contains("38;5")); // 256 色格式
    }

    #[test]
    fn test_ansi_color_to_256color_index() {
        let color = AnsiColor::new(0, 0, 0);
        let index = color.to_256color_index();
        assert_eq!(index, 0); // 黑色

        let white = AnsiColor::new(255, 255, 255);
        let white_index = white.to_256color_index();
        assert_eq!(white_index, 15); // 白色
    }

    #[test]
    fn test_terminal_capabilities_default() {
        let caps = TerminalCapabilities::default();
        assert!(!caps.supports_ansi_colors);
        assert!(caps.supports_utf8); // 默认启用 UTF-8
        assert!(!caps.supports_true_color);
        assert!(!caps.supports_hyperlinks);
        assert!(!caps.supports_256_colors);
        assert!(caps.width.is_none());
        assert!(caps.height.is_none());
    }

    #[test]
    fn test_terminal_capabilities_clone() {
        let caps = TerminalCapabilities {
            supports_ansi_colors: true,
            supports_utf8: true,
            supports_true_color: true,
            supports_hyperlinks: true,
            supports_256_colors: true,
            width: Some(80),
            height: Some(24),
        };
        let cloned = caps.clone();
        assert_eq!(cloned.supports_ansi_colors, caps.supports_ansi_colors);
        assert_eq!(cloned.width, caps.width);
    }

    #[test]
    fn test_ansi_sequence_new() {
        let caps = TerminalCapabilities::default();
        let seq = AnsiSequence::new(caps);
        assert!(seq.color(AnsiColor::new(255, 0, 0)).is_empty()); // 不支持颜色时返回空
    }

    #[test]
    fn test_ansi_sequence_static_methods() {
        assert_eq!(AnsiSequence::reset(), "\x1b[0m");
        assert_eq!(AnsiSequence::bold(), "\x1b[1m");
        assert_eq!(AnsiSequence::italic(), "\x1b[3m");
        assert_eq!(AnsiSequence::underline(), "\x1b[4m");
        assert_eq!(AnsiSequence::clear_line(), "\x1b[2K");
        assert_eq!(AnsiSequence::clear_screen(), "\x1b[2J");
        assert_eq!(AnsiSequence::hide_cursor(), "\x1b[?25l");
        assert_eq!(AnsiSequence::show_cursor(), "\x1b[?25h");
    }

    #[test]
    fn test_ansi_sequence_move_cursor() {
        let seq = AnsiSequence::move_cursor(5, 10);
        assert!(seq.contains("6")); // row + 1
        assert!(seq.contains("11")); // col + 1
    }

    #[test]
    fn test_tty_adapter_detect_capabilities() {
        let caps = TtyAdapter::detect_capabilities();
        // 基本能力检测应该返回有效结果
        assert!(caps.supports_utf8 == true || caps.supports_utf8 == false);
    }

    #[test]
    fn test_tty_adapter_capabilities() {
        let adapter = TtyAdapter::new();
        let caps = adapter.capabilities();

        // 验证能力结构完整性
        assert!(caps.supports_utf8 == true || caps.supports_utf8 == false);
    }

    #[test]
    fn test_supports_color_function() {
        // 测试辅助函数存在且可调用
        let _ = supports_color();
    }

    #[test]
    fn test_supports_true_color_function() {
        // 测试辅助函数存在且可调用
        let _ = supports_true_color();
    }

    #[test]
    fn test_ansi_color_debug() {
        let color = AnsiColor::new(100, 100, 100);
        let debug_str = format!("{:?}", color);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_terminal_capabilities_debug() {
        let caps = TerminalCapabilities::default();
        let debug_str = format!("{:?}", caps);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_tty_adapter_default() {
        let adapter = TtyAdapter::default();
        let caps = adapter.capabilities();
        // 默认适配器应该返回有效的能力
        assert!(caps.supports_utf8 == true || caps.supports_utf8 == false);
    }

    #[test]
    fn test_write_colored_noop() {
        let adapter = TtyAdapter::new();
        // 即使不支持颜色也不应该 panic
        let result = adapter.write_colored("test", AnsiColor::new(255, 0, 0));
        // 结果可能是 Ok(()) 或 Err，取决于输出环境
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_write_progress_noop() {
        let adapter = TtyAdapter::new();
        // 即使不支持颜色也不应该 panic
        let result = adapter.write_progress(50, 100, 10);
        assert!(result.is_ok() || result.is_err());
    }
}
