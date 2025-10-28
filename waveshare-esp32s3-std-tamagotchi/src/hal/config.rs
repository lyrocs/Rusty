// Hardware configuration for ESP32-S3

/// CPU configuration
#[derive(Debug, Clone)]
pub struct CpuConfig {
    pub frequency_mhz: u32,
    pub dual_core: bool,
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            frequency_mhz: 240,
            dual_core: true,
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub use_psram: bool,
    pub psram_speed_mhz: u32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            use_psram: true,
            psram_speed_mhz: 80,
        }
    }
}

/// Thread configuration
#[derive(Debug, Clone)]
pub struct ThreadConfig {
    pub input_thread_stack_size: usize,
    pub render_thread_stack_size: usize,
    pub io_thread_stack_size: usize,
    pub input_thread_priority: u8,
    pub render_thread_priority: u8,
    pub io_thread_priority: u8,
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            input_thread_stack_size: 4096,
            render_thread_stack_size: 8192,
            io_thread_stack_size: 4096,
            input_thread_priority: 10,
            render_thread_priority: 8,
            io_thread_priority: 6,
        }
    }
}

/// Performance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceMode {
    /// Maximum performance, both cores at full speed
    HighPerformance,
    /// Balanced mode with dynamic frequency scaling
    Balanced,
    /// Power saving mode, reduced frequency
    PowerSaving,
}

/// Overall system configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub cpu: CpuConfig,
    pub memory: MemoryConfig,
    pub threads: ThreadConfig,
    pub performance_mode: PerformanceMode,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            threads: ThreadConfig::default(),
            performance_mode: PerformanceMode::HighPerformance,
        }
    }
}
