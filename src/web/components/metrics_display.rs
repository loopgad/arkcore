//! 指标显示组件
//!
//! 显示系统资源使用情况（CPU、内存、磁盘、网络等）。

use super::super::MetricsData;

/// 指标显示属性
#[derive(Debug, Clone)]
pub struct MetricsDisplayProps {
    /// 指标数据
    pub metrics: MetricsData,
    /// 是否显示详细信息
    pub detailed: bool,
}

/// 获取使用率对应的颜色类
fn usage_color(usage: f32) -> &'static str {
    if usage >= 90.0 {
        "usage-critical"
    } else if usage >= 70.0 {
        "usage-warning"
    } else {
        "usage-normal"
    }
}

/// 获取延迟等级
fn latency_level(latency_ms: u64) -> &'static str {
    if latency_ms < 50 {
        "latency-excellent"
    } else if latency_ms < 100 {
        "latency-good"
    } else if latency_ms < 300 {
        "latency-fair"
    } else {
        "latency-poor"
    }
}

/// 指标显示组件
#[derive(Debug, Clone)]
pub struct MetricsDisplay;

impl MetricsDisplay {
    /// 渲染简略指标
    pub fn render_compact(props: &MetricsDisplayProps) -> String {
        let cpu_class = usage_color(props.metrics.cpu);
        let mem_class = usage_color(props.metrics.memory);
        let disk_class = usage_color(props.metrics.disk);
        let latency_class = latency_level(props.metrics.latency_ms);

        format!(
            r#"
            <div class="metrics-display metrics-compact">
                <div class="metric-item">
                    <span class="metric-icon">💻</span>
                    <span class="metric-label">CPU</span>
                    <span class="metric-bar-container">
                        <span class="metric-bar {}" style="width: {}%"></span>
                    </span>
                    <span class="metric-value {}">{}%</span>
                </div>
                <div class="metric-item">
                    <span class="metric-icon">🧠</span>
                    <span class="metric-label">MEM</span>
                    <span class="metric-bar-container">
                        <span class="metric-bar {}" style="width: {}%"></span>
                    </span>
                    <span class="metric-value {}">{}%</span>
                </div>
                <div class="metric-item">
                    <span class="metric-icon">💾</span>
                    <span class="metric-label">DISK</span>
                    <span class="metric-bar-container">
                        <span class="metric-bar {}" style="width: {}%"></span>
                    </span>
                    <span class="metric-value {}">{}%</span>
                </div>
                <div class="metric-item">
                    <span class="metric-icon">🌐</span>
                    <span class="metric-label">LAT</span>
                    <span class="metric-value {}">{}ms</span>
                </div>
            </div>
            "#,
            cpu_class, props.metrics.cpu as i32, cpu_class, props.metrics.cpu as i32,
            mem_class, props.metrics.memory as i32, mem_class, props.metrics.memory as i32,
            disk_class, props.metrics.disk as i32, disk_class, props.metrics.disk as i32,
            latency_class, props.metrics.latency_ms
        )
    }

    /// 渲染详细指标
    pub fn render_detailed(props: &MetricsDisplayProps) -> String {
        let compact = Self::render_compact(props);

        format!(
            r#"
            <div class="metrics-display metrics-detailed">
                {}
                <div class="metrics-extra">
                    <div class="extra-item">
                        <span class="extra-label">CPU Cores:</span>
                        <span class="extra-value">{}</span>
                    </div>
                    <div class="extra-item">
                        <span class="extra-label">Total Memory:</span>
                        <span class="extra-value">{} GB</span>
                    </div>
                    <div class="extra-item">
                        <span class="extra-label">Available Memory:</span>
                        <span class="extra-value">{} GB</span>
                    </div>
                    <div class="extra-item">
                        <span class="extra-label">Disk I/O:</span>
                        <span class="extra-value">{} MB/s</span>
                    </div>
                </div>
            </div>
            "#,
            compact,
            // 这些值应该是动态的，这里用假数据演示
            num_cpus(),
            total_memory_gb(),
            available_memory_gb(props.metrics.memory),
            "125" // 假数据
        )
    }

    /// 根据 props 渲染
    pub fn render(props: &MetricsDisplayProps) -> String {
        if props.detailed {
            Self::render_detailed(props)
        } else {
            Self::render_compact(props)
        }
    }
}

/// 获取 CPU 核心数（示例）
fn num_cpus() -> usize {
    // 这应该从系统获取
    8
}

/// 获取总内存（GB，示例）
fn total_memory_gb() -> f32 {
    // 这应该从系统获取
    32.0
}

/// 根据使用率计算可用内存
fn available_memory_gb(memory_usage: f32) -> f32 {
    let total = total_memory_gb();
    total * (100.0 - memory_usage) / 100.0
}

/// 获取指标显示的 CSS 样式
pub fn get_css() -> &'static str {
    r#"
    .metrics-display {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-sm);
        padding: var(--spacing);
        background-color: var(--color-surface);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
    }
    .metrics-compact {
        flex-direction: row;
        flex-wrap: wrap;
    }
    .metric-item {
        display: flex;
        align-items: center;
        gap: var(--spacing-sm);
        flex: 1;
        min-width: 120px;
    }
    .metric-icon {
        font-size: 1rem;
    }
    .metric-label {
        font-size: var(--font-size-caption);
        color: var(--color-text-secondary);
        width: 35px;
    }
    .metric-bar-container {
        flex: 1;
        height: 6px;
        background-color: var(--color-border);
        border-radius: 3px;
        overflow: hidden;
        min-width: 60px;
    }
    .metric-bar {
        height: 100%;
        border-radius: 3px;
        transition: width var(--transition);
    }
    .metric-bar.usage-normal {
        background-color: var(--color-success);
    }
    .metric-bar.usage-warning {
        background-color: var(--color-warning);
    }
    .metric-bar.usage-critical {
        background-color: var(--color-error);
    }
    .metric-value {
        font-size: var(--font-size-small);
        font-family: monospace;
        min-width: 45px;
        text-align: right;
    }
    .metric-value.usage-normal {
        color: var(--color-success);
    }
    .metric-value.usage-warning {
        color: var(--color-warning);
    }
    .metric-value.usage-critical {
        color: var(--color-error);
    }
    /* 延迟等级 */
    .latency-excellent { color: var(--color-success); }
    .latency-good { color: var(--color-info); }
    .latency-fair { color: var(--color-warning); }
    .latency-poor { color: var(--color-error); }
    /* 详细信息样式 */
    .metrics-detailed {
        gap: var(--spacing);
    }
    .metrics-extra {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: var(--spacing-sm);
        padding-top: var(--spacing);
        border-top: 1px solid var(--color-border);
    }
    .extra-item {
        display: flex;
        justify-content: space-between;
        font-size: var(--font-size-small);
    }
    .extra-label {
        color: var(--color-text-secondary);
    }
    .extra-value {
        color: var(--color-text-primary);
        font-family: monospace;
    }
    "#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_color() {
        assert_eq!(usage_color(50.0), "usage-normal");
        assert_eq!(usage_color(75.0), "usage-warning");
        assert_eq!(usage_color(95.0), "usage-critical");
    }

    #[test]
    fn test_latency_level() {
        assert_eq!(latency_level(30), "latency-excellent");
        assert_eq!(latency_level(80), "latency-good");
        assert_eq!(latency_level(200), "latency-fair");
        assert_eq!(latency_level(500), "latency-poor");
    }
}
