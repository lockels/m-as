use std::fmt;
use sysinfo::{Pid, ProcessStatus, System};

#[derive(Debug)]
pub struct Process {
    pid: Pid,
    name: String,
    cpu_usage: f32,
    memory_mb: f64,
    status: ProcessStatus,
    parent_pid: Option<Pid>,
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self.status {
            ProcessStatus::Run => "Running",
            ProcessStatus::Sleep => "Sleeping",
            ProcessStatus::Idle => "Idle",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Dead => "Dead",
            ProcessStatus::Stop => "Stopped",
            _ => "Unknown",
        };

        // Format parent PID
        let parent_str = match self.parent_pid {
            Some(pid) => pid.to_string(),
            None => "None".to_string(),
        };

        write!(
            f,
            "PID: {:<6} | Name: {:<20} | CPU: {:<5.1}% | Mem: {:<6.2}MB | Status: {:<8} | Parent: {}",
            self.pid,
            self.name,
            self.cpu_usage,
            self.memory_mb,
            status_str,
            parent_str
        )
    }
}

pub fn get_all_processes() -> Vec<Process> {
    let mut system = System::new_all();
    system.refresh_all();
    system
        .processes()
        .iter()
        .map(|(pid, process)| Process {
            pid: *pid,
            name: process.name().to_string_lossy().into_owned(),
            cpu_usage: process.cpu_usage(),
            memory_mb: (process.memory() as f64) / 1024.0 / 1024.0,
            status: process.status(),
            parent_pid: process.parent(),
        })
        .collect()
}

// == Functions for sorting processes ==

pub fn sort_by_cpu(processes: &mut [Process]) {
    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
}

pub fn sort_by_memory(processes: &mut [Process]) {
    processes.sort_by(|a, b| {
        b.memory_mb
            .partial_cmp(&a.memory_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}
