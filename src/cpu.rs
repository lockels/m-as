use std::collections::VecDeque;

use sysinfo::System;

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
