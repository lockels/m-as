use std::collections::VecDeque;

use sysinfo::System;

#[derive(Debug)]
pub struct MemoryInfo {
    system: System,
    // Memory
    pub total_memory: u64,
    pub used_memory: u64,
    pub memory_history: VecDeque<f32>,
    // Swap
    pub total_swap: u64,
    pub used_swap: u64,
    pub swap_history: VecDeque<f32>,
}

#[allow(dead_code)]
pub fn main() {
    let memory_info = MemoryInfo::new();

    println!("{}", memory_info.memory_usage_text());
    println!("{}", memory_info.swap_usage_text());
}

impl MemoryInfo {
    pub fn new() -> MemoryInfo {
        let mut system = System::new_all();
        system.refresh_memory();

        Self {
            total_memory: system.total_memory(),
            used_memory: system.used_memory(),
            memory_history: VecDeque::with_capacity(60),
            total_swap: system.total_swap(),
            used_swap: system.used_swap(),
            swap_history: VecDeque::with_capacity(60),
            system,
        }
    }

    pub fn update(&mut self) {
        self.system.refresh_memory();

        // Get raw values in bytes
        self.total_memory = self.system.total_memory();
        self.used_memory = self.system.used_memory();
        self.total_swap = self.system.total_swap();
        self.used_swap = self.system.used_swap();

        // Calculate memory percentage using available memory instead of used_memory
        let available_memory = self.system.available_memory();
        let actual_used_memory = self.total_memory.saturating_sub(available_memory);
        let memory_percent = (actual_used_memory as f32 / self.total_memory as f32) * 100.0;

        // Calculate swap percentage
        let swap_percent = if self.total_swap > 0 {
            (self.used_swap as f32 / self.total_swap as f32) * 100.0
        } else {
            0.0
        };

        // Update histories
        self.memory_history.push_back(memory_percent);
        self.swap_history.push_back(swap_percent);

        if self.memory_history.len() > 60 {
            self.memory_history.pop_front();
        }
        if self.swap_history.len() > 60 {
            self.swap_history.pop_front();
        }
    }

    pub fn _memory_graph_data(&self) -> Vec<(f64, f64)> {
        self.memory_history
            .iter()
            .enumerate()
            .map(|(i, &usage)| (i as f64, usage as f64))
            .collect()
    }

    pub fn _swap_graph_data(&self) -> Vec<(f64, f64)> {
        self.swap_history
            .iter()
            .enumerate()
            .map(|(i, &usage)| (i as f64, usage as f64))
            .collect()
    }

    pub fn memory_usage_text(&self) -> String {
        let available = self.system.available_memory();
        let used = self.total_memory.saturating_sub(available);
        format!(
            "Memory: {:.1}% ({:.1}GB / {:.1}GB)",
            (used as f32 / self.total_memory as f32) * 100.0,
            bytes_to_gb(used),
            bytes_to_gb(self.total_memory)
        )
    }

    pub fn swap_usage_text(&self) -> String {
        if self.total_swap > 0 {
            format!(
                "Swap: {:.1}% ({:.1}GB / {:.1}GB)",
                (self.used_swap as f32 / self.total_swap as f32) * 100.0,
                bytes_to_gb(self.used_swap),
                bytes_to_gb(self.total_swap)
            )
        } else {
            "Swap: Not Available".to_string()
        }
    }

    pub fn current_memory_percent(&self) -> f32 {
        (self.used_memory as f32 / self.total_memory as f32) * 100.0
    }

    pub fn current_swap_percent(&self) -> f32 {
        if self.total_swap > 0 {
            (self.used_swap as f32 / self.total_swap as f32) * 100.0
        } else {
            0.0
        }
    }
}

fn bytes_to_gb(bytes: u64) -> f32 {
    bytes as f32 / 1024.0 / 1024.0 / 1024.0
}
