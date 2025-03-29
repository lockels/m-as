use std::collections::VecDeque;
use sysinfo::System;

#[allow(dead_code)]
pub fn main() {
    let mut cpu_info = CpuInfo::new();

    loop {
        cpu_info.update();
        println!("Global CPU Usage: {:.2}%", cpu_info.global_usage);
        for core in &cpu_info.cores {
            println!("Core {}: {:.2}%", core.name, core.usage);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[derive(Debug, Clone)]
pub struct CpuCore {
    pub name: String,
    pub usage: f32,
    pub history: VecDeque<f32>, // For graphing historical usage
}

impl CpuCore {
    pub fn new(name: String) -> Self {
        Self {
            name,
            usage: 0.0,
            history: VecDeque::with_capacity(60), // Store 60 data points (1 minute at 1 update/sec)
        }
    }
}

#[derive(Debug)]
pub struct CpuInfo {
    pub global_usage: f32,
    pub cores: Vec<CpuCore>,
    pub history: VecDeque<f32>, // Global CPU history
    system: System,             // Keep the System instance as part of the struct
}

impl CpuInfo {
    /// Create a new CpuInfo struct with default value
    pub fn new() -> Self {
        let mut system = System::new_all();
        // Wait a bit to get accurate initial readings
        std::thread::sleep(std::time::Duration::from_millis(500));

        let cores = system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, _)| CpuCore::new(format!("Core {}", i + 1)))
            .collect();

        Self {
            global_usage: 0.0,
            cores,
            history: VecDeque::with_capacity(60),
            system,
        }
    }

    /// Update the CPU information
    pub fn update(&mut self) {
        // Refresh CPU information
        self.system.refresh_cpu_all();

        // Need to wait a bit between refreshes to get accurate CPU usage
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Update global usage
        self.global_usage = self.system.global_cpu_usage();
        self.history.push_back(self.global_usage);
        if self.history.len() > 60 {
            self.history.pop_front();
        }

        // Update each core's usage
        for (i, cpu) in self.system.cpus().iter().enumerate() {
            if let Some(core) = self.cores.get_mut(i) {
                core.usage = cpu.cpu_usage();
                core.history.push_back(core.usage);
                if core.history.len() > 60 {
                    core.history.pop_front();
                }
            }
        }
    }

    pub fn _core_graph_data(&self, _core_index: usize) -> Option<Vec<(f64, f64)>> {
        self.cores.get(_core_index).map(|core| {
            core.history
                .iter()
                .enumerate()
                .map(|(i, &usage)| (i as f64, usage as f64))
                .collect()
        })
    }
}
