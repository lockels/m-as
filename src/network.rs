use std::collections::VecDeque;
use sysinfo::System;

#[derive(Debug)]
pub struct NetworkMonitor {
    system: System,
    rx_history: VecDeque<u64>,
    tx_history: VecDeque<u64>,
    history_capacity: usize,
    last_rx: u64,
    last_tx: u64,
    max_bandwidth: u64,
}
