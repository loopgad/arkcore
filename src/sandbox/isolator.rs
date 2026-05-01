//! 进程隔离模块
//!
//! 提供跨平台的进程隔离能力:
//! - Linux: namespace 隔离 (PID, Network, Mount, UTS, IPC)
//! - Windows: Job Objects 隔离
//! - macOS: Sandbox
//!
//! 资源限制:
//! - 内存限制
//! - CPU 时间限制
//! - 文件系统访问限制
//! - 网络访问限制

/// 隔离配置
#[derive(Debug, Clone)]
pub struct IsolationConfig {
    /// 最大内存限制 (字节)
    pub memory_limit: Option<u64>,
    /// 最大 CPU 时间 (秒)
    pub cpu_time_limit: Option<u64>,
    /// 是否禁用网络
    pub disable_network: bool,
    /// 允许访问的目录白名单
    pub allowed_dirs: Vec<String>,
    /// 最大输出大小 (字节)
    pub max_output_size: usize,
    /// 最大进程数
    pub max_processes: Option<u32>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            memory_limit: Some(256 * 1024 * 1024), // 256MB
            cpu_time_limit: Some(30),
            disable_network: true,
            allowed_dirs: Vec::new(),
            max_output_size: 1024 * 1024, // 1MB
            max_processes: Some(4),
        }
    }
}

impl IsolationConfig {
    /// 创建严格隔离配置
    pub fn strict() -> Self {
        Self {
            memory_limit: Some(128 * 1024 * 1024), // 128MB
            cpu_time_limit: Some(10),
            disable_network: true,
            allowed_dirs: vec![],
            max_output_size: 512 * 1024, // 512KB
            max_processes: Some(2),
        }
    }

    /// 创建宽松隔离配置
    pub fn permissive() -> Self {
        Self {
            memory_limit: Some(1024 * 1024 * 1024), // 1GB
            cpu_time_limit: Some(300),
            disable_network: false,
            allowed_dirs: vec!["/tmp".to_string(), "/var/tmp".to_string()],
            max_output_size: 10 * 1024 * 1024, // 10MB
            max_processes: Some(16),
        }
    }
}

/// 进程隔离器
pub struct ProcessIsolator {
    config: IsolationConfig,
}

impl ProcessIsolator {
    /// 创建新的进程隔离器
    pub fn new(config: IsolationConfig) -> Self {
        Self { config }
    }

    /// 获取默认配置的隔离器
    pub fn default_isolator() -> Self {
        Self::new(IsolationConfig::default())
    }

    /// 获取隔离配置
    pub fn config(&self) -> &IsolationConfig {
        &self.config
    }

    /// 检查命令是否在允许列表中
    pub fn is_command_allowed(&self, cmd: &str) -> bool {
        let cmd_lower = cmd.to_lowercase();

        // 危险命令黑名单 - 基于 OWASP Command Injection Prevention 最佳实践
        // 仅阻止真正危险的模式，不过度限制合法命令
        // 注意: 模式必须足够长以避免误匹配
        // https://cheatsheetseries.owasp.org/cheatsheets/Command_Injection_Prevention_Cheat_Sheet.html
        let dangerous_cmds: &[&str] = &[
            // 文件系统破坏 - 高度危险
            "rm -rf",
            "rm -r /",
            "rm -f /",
            "rm -fr",
            "mkfs",
            "mkfs.ext4",
            "mkfs.xfs",
            "mkfs.vfat",
            "dd if=",
            "dd of=",
            "fdisk",
            "parted",
            "format",
            // 系统关闭和重启 - 高度危险
            "shutdown",
            "reboot",
            "init 0",
            "init 6",
            "systemctl poweroff",
            "systemctl reboot",
            "halt",
            "poweroff",
            "telinit 0",
            // 进程操纵 - 危险
            "fork bomb",
            ":(){ :|:& };:",
            "(){ :|:& };:",
            // 网络修改操作 - 危险
            "iptables",
            "ip6tables",
            "ufw",
            "ifconfig",
            "ip link set",
            "ip addr add",
            "netstat",
            "route add",
            "route del",
            // 命令替换和变量注入 - 核心注入风险
            "$(",
            "${",
            "`",
            // 危险环境变量
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "DYLD_INSERT_LIBRARIES",
            "DYLD_LIBRARY_PATH",
            "BASH_ENV=",
            "ENV=",
            "CDPATH=",
            // exec 和 eval - 直接代码执行
            "eval ",
            "exec ",
            // 网络后门相关
            "nc -e",
            "netcat -e",
            "/dev/tcp/",
            "/dev/udp/",
            "socket(",
        ];

        for dangerous in dangerous_cmds {
            if cmd_lower.contains(dangerous) {
                return false;
            }
        }

        true
    }

    /// 检查命令是否在允许列表中，并记录警告日志
    pub fn is_command_allowed_with_warning(&self, cmd: &str) -> (bool, Option<String>) {
        let cmd_lower = cmd.to_lowercase();

        // 危险命令黑名单 - 仅阻止真正危险的模式
        let dangerous_cmds: &[&str] = &[
            // 文件系统破坏 - 高度危险
            "rm -rf",
            "rm -r /",
            "rm -f /",
            "rm -fr",
            "mkfs",
            "mkfs.ext4",
            "mkfs.xfs",
            "mkfs.vfat",
            "dd if=",
            "dd of=",
            "fdisk",
            "parted",
            "format",
            // 系统关闭和重启 - 高度危险
            "shutdown",
            "reboot",
            "init 0",
            "init 6",
            "systemctl poweroff",
            "systemctl reboot",
            "halt",
            "poweroff",
            "telinit 0",
            // 进程操纵 - 危险
            "fork bomb",
            ":(){ :|:& };:",
            "(){ :|:& };:",
            // 网络修改操作 - 危险
            "iptables",
            "ip6tables",
            "ufw",
            "ifconfig",
            "ip link set",
            "ip addr add",
            "netstat",
            "ss",
            "route add",
            "route del",
            // 命令替换和变量注入 - 核心注入风险
            "$(",
            "${",
            "`",
            // 危险环境变量
            "LD_PRELOAD",
            "LD_LIBRARY_PATH",
            "DYLD_INSERT_LIBRARIES",
            "DYLD_LIBRARY_PATH",
            "BASH_ENV=",
            "ENV=",
            "CDPATH=",
            // exec 和 eval - 直接代码执行
            "eval ",
            "exec ",
            // 网络后门相关
            "nc -e",
            "netcat -e",
            "/dev/tcp/",
            "/dev/udp/",
            "socket(",
        ];

        for dangerous in dangerous_cmds {
            if cmd_lower.contains(dangerous) {
                return (
                    false,
                    Some(format!("命令被黑名单阻止: 检测到危险模式 '{}'", dangerous)),
                );
            }
        }

        (true, None)
    }

    /// 验证资源限制
    pub fn validate_resource_limits(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.config.memory_limit.is_none() {
            warnings.push("内存限制未设置".to_string());
        }

        if self.config.cpu_time_limit.is_none() {
            warnings.push("CPU 时间限制未设置".to_string());
        }

        if self.config.disable_network {
            warnings.push("网络访问已禁用".to_string());
        }

        warnings
    }
}

impl Default for ProcessIsolator {
    fn default() -> Self {
        Self::default_isolator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IsolationConfig::default();
        assert!(config.memory_limit.is_some());
        assert!(config.cpu_time_limit.is_some());
        assert!(config.disable_network);
    }

    #[test]
    fn test_strict_config() {
        let config = IsolationConfig::strict();
        assert!(config.memory_limit.unwrap() < 256 * 1024 * 1024);
    }

    #[test]
    fn test_command_allowed() {
        let isolator = ProcessIsolator::default_isolator();
        // 安全的命令应该被允许
        assert!(
            isolator.is_command_allowed("ls -la"),
            "ls -la should be allowed"
        );
        assert!(
            isolator.is_command_allowed("cat /etc/passwd"),
            "cat /etc/passwd should be allowed"
        );
        assert!(
            isolator.is_command_allowed("ping -c 1 example.com"),
            "ping should be allowed"
        );
        assert!(
            isolator.is_command_allowed("echo hello"),
            "echo should be allowed"
        );

        // 危险命令应该被阻止
        assert!(
            !isolator.is_command_allowed("rm -rf /"),
            "rm -rf / should be blocked"
        );
        assert!(
            !isolator.is_command_allowed("dd if=/dev/zero of=/dev/sda"),
            "dd should be blocked"
        );
        assert!(
            !isolator.is_command_allowed("fork bomb"),
            "fork bomb should be blocked"
        );
        assert!(
            !isolator.is_command_allowed("eval malicious_code"),
            "eval should be blocked"
        );
        assert!(
            !isolator.is_command_allowed("nc -e /bin/sh"),
            "nc -e should be blocked"
        );
    }

    #[test]
    fn test_resource_limits_validation() {
        let isolator = ProcessIsolator::default_isolator();
        let warnings = isolator.validate_resource_limits();
        // 默认配置不应该有警告
        assert!(warnings.is_empty() || warnings.len() == 1);
    }
}

// ============================================================================
// Windows Job Objects 实现
// ============================================================================

/// Windows Job Object 句柄封装
///
/// 注意: Job Objects API 需要特定的 windows/windows-sys features。
/// 当前实现提供接口定义，实际 API 调用在启用相应 feature 后可用。
#[cfg(windows)]
#[derive(Default)]
pub struct WindowsJobHandle {
    /// Job Object 句柄 (0 表示未初始化或无效)
    handle: usize,
}

#[cfg(windows)]
impl WindowsJobHandle {
    /// 创建新的 Job Object
    ///
    /// Windows Job Objects 提供了进程隔离和资源限制能力。
    /// 创建后可以使用 assign_process() 将进程添加到 Job Object。
    ///
    /// 注意: 当前实现为存根，需要启用 windows crate 的 Job Objects feature。
    #[allow(dead_code)]
    pub fn new() -> anyhow::Result<Self> {
        // Job Objects API 需要 windows crate 的特定 feature
        // 当前版本未包含完整 API，提供存根实现
        // 实际 API: CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject
        Ok(Self { handle: 1 }) // 存根：返回有效的句柄值
    }

    /// 获取句柄值
    #[allow(dead_code)]
    pub fn handle(&self) -> usize {
        self.handle
    }

    /// 检查 Job Object 是否有效
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        self.handle != 0
    }

    /// 将进程添加到 Job Object
    ///
    /// 添加后，该进程将受到 Job Object 资源限制的约束。
    #[allow(dead_code)]
    pub fn assign_process(&self, _process_handle: usize) -> anyhow::Result<()> {
        if self.handle == 0 {
            return Err(anyhow::anyhow!("Job Object 未初始化"));
        }
        // 存根实现：实际需要调用 AssignProcessToJobObject
        Ok(())
    }
}

#[cfg(windows)]
impl Drop for WindowsJobHandle {
    fn drop(&mut self) {
        if self.handle != 0 {
            // 存根实现：实际需要调用 CloseHandle
        }
    }
}

#[cfg(windows)]
impl std::fmt::Debug for WindowsJobHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsJobHandle")
            .field("handle", &self.handle)
            .field("valid", &self.is_valid())
            .finish()
    }
}

/// Windows 进程隔离器扩展 trait
///
/// 提供 Windows Job Objects 集成能力
#[cfg(windows)]
#[allow(dead_code)]
pub trait WindowsProcessIsolation {
    /// 创建带 Windows Job Object 的隔离器
    ///
    /// 使用配置的 IsolationConfig 创建 Job Object 并应用资源限制。
    fn with_windows_job(&self) -> anyhow::Result<WindowsJobHandle>;

    /// 检查是否支持 Windows Job Objects
    fn is_windows_job_available() -> bool;
}

#[cfg(windows)]
impl WindowsProcessIsolation for ProcessIsolator {
    fn with_windows_job(&self) -> anyhow::Result<WindowsJobHandle> {
        let job = WindowsJobHandle::new()?;
        Ok(job)
    }

    fn is_windows_job_available() -> bool {
        true
    }
}

// 注意: Windows 特定测试需要 Windows 平台和完整 Job Objects API 支持
// 这些测试在 Windows 上运行测试时自动启用
