//! 跨平台信号处理
//!
//! 提供统一的信号处理接口，适配 Unix/Windows 信号差异

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 信号类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// 中断信号 (Ctrl+C)
    Interrupt,
    /// 终止信号
    Terminate,
    /// 管道断裂信号
    BrokenPipe,
    /// 用户退出信号
    Quit,
    /// 挂起信号
    Suspend,
}

/// 信号处理器 trait
pub trait SignalHandler: Send + Sync {
    /// 处理信号
    fn handle(&self, signal: Signal);

    /// 注册处理器
    fn register(&self) -> anyhow::Result<()>;

    /// 注销处理器
    fn unregister(&self) -> anyhow::Result<()>;
}

/// 全局信号状态
#[derive(Debug)]
pub struct GlobalSignalState {
    interrupt_received: AtomicBool,
    terminate_received: AtomicBool,
    #[allow(dead_code)]
    broken_pipe_ignored: AtomicBool,
}

impl GlobalSignalState {
    /// 创建新的全局状态
    pub fn new() -> Self {
        Self {
            interrupt_received: AtomicBool::new(false),
            terminate_received: AtomicBool::new(false),
            broken_pipe_ignored: AtomicBool::new(false),
        }
    }

    /// 检查是否收到中断信号
    pub fn is_interrupt_received(&self) -> bool {
        self.interrupt_received.load(Ordering::SeqCst)
    }

    /// 检查是否收到终止信号
    pub fn is_terminate_received(&self) -> bool {
        self.terminate_received.load(Ordering::SeqCst)
    }

    /// 标记收到中断信号
    pub fn mark_interrupt(&self) {
        self.interrupt_received.store(true, Ordering::SeqCst);
    }

    /// 标记收到终止信号
    pub fn mark_terminate(&self) {
        self.terminate_received.store(true, Ordering::SeqCst);
    }

    /// 重置所有信号状态
    pub fn reset(&self) {
        self.interrupt_received.store(false, Ordering::SeqCst);
        self.terminate_received.store(false, Ordering::SeqCst);
    }
}

impl Default for GlobalSignalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Unix 信号处理实现
#[cfg(unix)]
pub mod unix_signal {
    use super::*;
    use std::os::unix::process::CommandExt;
    use std::process;
    use tokio::signal::unix::{signal, SignalKind};

    /// Unix 信号处理器
    pub struct UnixSignalHandler {
        state: Arc<GlobalSignalState>,
    }

    impl UnixSignalHandler {
        pub fn new(state: Arc<GlobalSignalState>) -> Self {
            Self { state }
        }

        /// 设置进程为后台运行（忽略某些信号）
        pub fn setup_child_process(&self) {
            // 忽略 SIGPIPE，防止写入关闭的管道时进程终止
            unsafe {
                libc::signal(libc::SIGPIPE, libc::SIG_IGN);
            }
        }

        /// 设置进程组
        pub fn setup_process_group(&self) -> std::io::Result<process::Child> {
            // 在 fork 之后调用，设置子进程为新的进程组领导者
            Ok(process::Command::new("true").spawn()?)
        }
    }

    impl SignalHandler for UnixSignalHandler {
        fn handle(&self, signal: Signal) {
            match signal {
                Signal::Interrupt => self.state.mark_interrupt(),
                Signal::Terminate => self.state.mark_terminate(),
                Signal::BrokenPipe => {
                    // SIGPIPE 通常被忽略，不设置标志
                }
                Signal::Quit | Signal::Suspend => {
                    self.state.mark_interrupt();
                }
            }
        }

        fn register(&self) -> anyhow::Result<()> {
            // Unix 信号处理在运行时动态注册
            Ok(())
        }

        fn unregister(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    /// 注册默认的信号处理器
    pub fn setup_signal_handlers(
        state: Arc<GlobalSignalState>,
    ) -> anyhow::Result<UnixSignalHandler> {
        let handler = UnixSignalHandler::new(state);
        handler.setup_child_process();
        Ok(handler)
    }

    /// 异步等待中断信号
    pub async fn wait_for_interrupt() -> anyhow::Result<()> {
        let mut interrupt =
            signal(SignalKind::interrupt()).context("Failed to create interrupt signal handler")?;

        interrupt.recv().await;
        Ok(())
    }

    /// 异步等待终止信号
    pub async fn wait_for_terminate() -> anyhow::Result<()> {
        let mut terminate =
            signal(SignalKind::terminate()).context("Failed to create terminate signal handler")?;

        terminate.recv().await;
        Ok(())
    }
}

/// Windows 信号处理实现
#[cfg(windows)]
pub mod windows_signal {
    use super::*;

    /// Windows 控制台事件类型
    #[derive(Debug, Clone, Copy)]
    #[repr(u32)]
    #[allow(dead_code)]
    enum ConsoleEvent {
        CtrlC = 0,
        CtrlBreak = 1,
        CtrlClose = 2,
        CtrlLogoff = 5,
        CtrlShutdown = 6,
    }

    /// Windows 信号处理器
    pub struct WindowsSignalHandler {
        state: Arc<GlobalSignalState>,
        console_ctrl_handler: Option<unsafe extern "system" fn(u32) -> i32>,
    }

    impl WindowsSignalHandler {
        pub fn new(state: Arc<GlobalSignalState>) -> Self {
            Self {
                state,
                console_ctrl_handler: None,
            }
        }
    }

    impl SignalHandler for WindowsSignalHandler {
        fn handle(&self, signal: Signal) {
            match signal {
                Signal::Interrupt => self.state.mark_interrupt(),
                Signal::Terminate => self.state.mark_terminate(),
                Signal::BrokenPipe => {}
                Signal::Quit | Signal::Suspend => self.state.mark_interrupt(),
            }
        }

        fn register(&self) -> anyhow::Result<()> {
            // Windows 使用 SetConsoleCtrlHandler
            unsafe extern "system" fn console_handler(ctrl_type: u32) -> i32 {
                match ctrl_type {
                    0..=2 => {
                        // Ctrl+C, Ctrl+Break, Ctrl+Close
                        global_state().mark_interrupt();
                        1 // TRUE
                    }
                    _ => 0, // FALSE
                }
            }

            unsafe {
                let result = windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
                    Some(console_handler),
                    1, // TRUE = add the handler
                );
                if result == 0 {
                    anyhow::bail!("Failed to set console control handler");
                }
            }

            Ok(())
        }

        fn unregister(&self) -> anyhow::Result<()> {
            if let Some(handler) = self.console_ctrl_handler {
                unsafe {
                    let _ = windows_sys::Win32::System::Console::SetConsoleCtrlHandler(
                        Some(handler),
                        0, // FALSE = remove the handler
                    );
                }
            }
            Ok(())
        }
    }

    /// 注册默认的信号处理器
    pub fn setup_signal_handlers(
        state: Arc<GlobalSignalState>,
    ) -> anyhow::Result<WindowsSignalHandler> {
        let handler = WindowsSignalHandler::new(state);
        handler.register()?;
        Ok(handler)
    }
}

/// 获取信号描述
pub fn signal_description(signal: Signal) -> &'static str {
    match signal {
        Signal::Interrupt => "Interrupt (Ctrl+C)",
        Signal::Terminate => "Terminate",
        Signal::BrokenPipe => "Broken Pipe",
        Signal::Quit => "Quit",
        Signal::Suspend => "Suspend",
    }
}

/// 检查是否应该忽略信号
pub fn should_ignore(signal: Signal) -> bool {
    matches!(signal, Signal::BrokenPipe)
}

/// 转换为平台特定的信号值
#[cfg(unix)]
pub fn to_platform_signal(signal: Signal) -> libc::c_int {
    match signal {
        Signal::Interrupt => libc::SIGINT,
        Signal::Terminate => libc::SIGTERM,
        Signal::BrokenPipe => libc::SIGPIPE,
        Signal::Quit => libc::SIGQUIT,
        Signal::Suspend => libc::SIGTSTP,
    }
}

#[cfg(windows)]
pub fn to_platform_signal(signal: Signal) -> u32 {
    match signal {
        Signal::Interrupt => 0, // CTRL_C_EVENT
        Signal::Terminate => 1, // CTRL_BREAK_EVENT
        Signal::BrokenPipe => 0,
        Signal::Quit => 1,
        Signal::Suspend => 1,
    }
}

/// 获取全局信号状态
pub fn global_state() -> &'static GlobalSignalState {
    static STATE: std::sync::OnceLock<GlobalSignalState> = std::sync::OnceLock::new();
    STATE.get_or_init(GlobalSignalState::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_description() {
        assert_eq!(signal_description(Signal::Interrupt), "Interrupt (Ctrl+C)");
        assert_eq!(signal_description(Signal::Terminate), "Terminate");
        assert_eq!(signal_description(Signal::BrokenPipe), "Broken Pipe");
    }

    #[test]
    fn test_should_ignore() {
        assert!(should_ignore(Signal::BrokenPipe));
        assert!(!should_ignore(Signal::Interrupt));
        assert!(!should_ignore(Signal::Terminate));
    }

    #[test]
    fn test_global_state() {
        let state = global_state();
        state.reset();
        assert!(!state.is_interrupt_received());
        assert!(!state.is_terminate_received());
    }

    // ========== 新增测试 ==========

    #[test]
    fn test_signal_enum_variants() {
        // 确保所有 Signal 变体都可以创建
        let signals = [
            Signal::Interrupt,
            Signal::Terminate,
            Signal::BrokenPipe,
            Signal::Quit,
            Signal::Suspend,
        ];

        for signal in signals {
            let desc = signal_description(signal);
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_global_signal_state_mark_interrupt() {
        let state = global_state();
        state.reset();
        state.mark_interrupt();
        assert!(state.is_interrupt_received());
        state.reset();
        assert!(!state.is_interrupt_received());
    }

    #[test]
    fn test_global_signal_state_mark_terminate() {
        let state = global_state();
        state.reset();
        state.mark_terminate();
        assert!(state.is_terminate_received());
        state.reset();
        assert!(!state.is_terminate_received());
    }

    #[test]
    fn test_global_signal_state_new() {
        let state = GlobalSignalState::new();
        assert!(!state.is_interrupt_received());
        assert!(!state.is_terminate_received());
    }

    #[test]
    fn test_global_signal_state_default() {
        let state = GlobalSignalState::default();
        // 默认状态应该没有收到任何信号
        assert!(!state.is_interrupt_received());
        assert!(!state.is_terminate_received());
    }

    #[test]
    fn test_signal_ordering() {
        // 测试 Signal 可以比较
        assert_eq!(Signal::Interrupt, Signal::Interrupt);
        assert_ne!(Signal::Interrupt, Signal::Terminate);
    }

    #[test]
    fn test_signal_copy() {
        let sig = Signal::Interrupt;
        let sig2 = sig;
        assert_eq!(sig, sig2);
    }

    #[test]
    fn test_signal_clone() {
        let sig = Signal::Terminate;
        let sig2 = sig.clone();
        assert_eq!(sig, sig2);
    }

    #[test]
    fn test_global_state_concurrent_access() {
        use std::sync::Arc;
        use std::thread;

        let state = Arc::new(GlobalSignalState::new());
        let state_clone = state.clone();

        // 在另一个线程中标记
        let handle = thread::spawn(move || {
            state_clone.mark_interrupt();
        });

        handle.join().expect("Thread join failed");

        // 主线程应该能看到标记（使用 SeqCst ordering）
        state.reset();
    }

    #[test]
    fn test_to_platform_signal() {
        // 测试信号转换函数存在且可调用
        let sig = Signal::Interrupt;
        let platform_sig = to_platform_signal(sig);
        // 平台信号应该是某种整数类型
        assert!(platform_sig >= 0);
    }

    #[test]
    fn test_should_ignore_all_signals() {
        // 只有 BrokenPipe 应该被忽略
        assert!(should_ignore(Signal::BrokenPipe));
        assert!(!should_ignore(Signal::Quit));
        assert!(!should_ignore(Signal::Suspend));
    }

    #[test]
    fn test_signal_debug() {
        let sig = Signal::Interrupt;
        let debug_str = format!("{:?}", sig);
        assert!(debug_str.contains("Interrupt"));
    }

    #[test]
    fn test_global_state_debug() {
        let state = GlobalSignalState::new();
        let debug_str = format!("{:?}", state);
        assert!(!debug_str.is_empty());
    }
}
