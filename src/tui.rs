use std::time::{Duration, Instant};

use crate::cpu::CpuInfo;
use crate::process::{self, get_all_processes, Process};
use color_eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, BorderType, Borders, Cell, Chart, Dataset, GraphType, Paragraph, Row, Table,
    TableState,
};
use ratatui::{DefaultTerminal, Frame};
use std::sync::{Arc, Mutex};
use std::thread;
use sysinfo::ProcessStatus;

const fn make_highlight_style() -> Style {
    Style::new()
        .bg(Color::Rgb(70, 70, 90))
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub struct AppState {
    pub cpu_info: CpuInfo,
    pub processes: Vec<Process>,
    pub selected_process: usize,
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new() -> Self {
        let mut processes = process::get_all_processes();
        process::sort_by_memory(&mut processes);

        Self {
            cpu_info: CpuInfo::new(),
            processes,
            selected_process: 0,
            scroll_offset: 0,
        }
    }

    pub fn update_processes(&mut self) {
        self.processes = get_all_processes();
        process::sort_by_memory(&mut self.processes);
    }
}

pub fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

pub fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // Shared state between threads
    let state = Arc::new(Mutex::new(AppState::new()));

    // Clone Arc for background thread
    let state_thread = Arc::clone(&state);

    // Spawn background thread for data updates
    thread::spawn(move || {
        let mut last_cpu_update = Instant::now();
        let cpu_update_interval = Duration::from_millis(1000);

        loop {
            let now = Instant::now();

            // Update processes more frequently (250ms)
            {
                let mut state = state_thread.lock().unwrap();
                state.update_processes();
            }

            // Update CPU less frequently (1s) since it's more expensive
            if now.duration_since(last_cpu_update) >= cpu_update_interval {
                let mut state = state_thread.lock().unwrap();
                state.cpu_info.update();
                last_cpu_update = now;
            }

            thread::sleep(Duration::from_millis(50)); // Small sleep to prevent busy-wait
        }
    });

    // Main thread handles only UI and input
    let mut visible_height = terminal.size()?.height as usize - 4;
    loop {
        // Non-blocking event processing
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => {
                        let mut state = state.lock().unwrap();
                        if state.selected_process < state.processes.len().saturating_sub(1) {
                            state.selected_process += 1;
                        }
                        if state.selected_process >= state.scroll_offset + visible_height {
                            state.scroll_offset = state.selected_process - visible_height + 1;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let mut state = state.lock().unwrap();
                        if state.selected_process > 0 {
                            state.selected_process -= 1;
                        }
                        if state.selected_process < state.scroll_offset {
                            state.scroll_offset = state.selected_process;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Smooth rendering at 60fps
        terminal.draw(|f| {
            visible_height = f.area().height as usize - 4;
            let state = state.lock().unwrap();
            render(f, &state)
        })?;

        // Small sleep to prevent 100% CPU usage on UI thread
        thread::sleep(Duration::from_millis(16)); // ~60fps
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Top 40% for CPU
            Constraint::Percentage(60), // Bottom 60% for processes, etc.
        ])
        .split(frame.area());

    render_cpu_section(frame, &state.cpu_info, main_layout[0]);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left 50% for processes
            Constraint::Percentage(50), // Right 50% for other info
        ])
        .split(main_layout[1]);

    render_process_section(
        frame,
        &state.processes,
        state.selected_process,
        state.scroll_offset,
        bottom_layout[0],
    );

    let right_side_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top 50% for memory
            Constraint::Percentage(50), // Bottom 50% for disk
        ])
        .split(bottom_layout[1]);

    render_memory_section(frame, right_side_layout[0]);
    render_disk_section(frame, right_side_layout[1]);
}

fn render_cpu_section(frame: &mut Frame, cpu_info: &CpuInfo, area: Rect) {
    let cpu_block = Block::default()
        .title("CPU Usage")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightCyan))
        .style(Style::default());

    let cpu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    render_cpu_cores_list(frame, cpu_info, cpu_layout[0]);
    render_cpu_graphs(frame, cpu_info, cpu_layout[1]);

    frame.render_widget(cpu_block, area);
}

fn render_cpu_cores_list(frame: &mut Frame, cpu_info: &CpuInfo, area: Rect) {
    let cores_list: Vec<Line> = cpu_info
        .cores
        .iter()
        .enumerate()
        .map(|(i, core)| {
            let color = CORE_COLORS[i % CORE_COLORS.len()];
            Line::from(vec![
                Span::styled(
                    format!("{:>6}: ", core.name),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!("{:>5.1}%", core.usage), Style::default().fg(color)),
            ])
        })
        .collect();

    // Vertical centering
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(cores_list.len() as u16),
            Constraint::Min(1),
        ])
        .split(area);

    // Horizontal centering
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(20),
            Constraint::Min(1),
        ])
        .split(vertical_layout[1]);

    let list_widget = Paragraph::new(cores_list)
        .block(Block::default())
        .alignment(Alignment::Left);

    frame.render_widget(list_widget, horizontal_layout[1]);
}

fn render_cpu_graphs(frame: &mut Frame, cpu_info: &CpuInfo, area: Rect) {
    // First collect all the graph data
    let core_data: Vec<(String, Vec<(f64, f64)>, Color)> = cpu_info
        .cores
        .iter()
        .enumerate()
        .map(|(i, core)| {
            let data = core
                .history
                .iter()
                .enumerate()
                .map(|(x, &y)| (x as f64, y as f64))
                .collect();
            (core.name.clone(), data, CORE_COLORS[i % CORE_COLORS.len()])
        })
        .collect();

    // Create the chart widget
    let chart = {
        let y_min = 0.0;
        let y_max = 50.0;
        let datasets = core_data
            .iter()
            .map(|(name, data, color)| {
                Dataset::default()
                    .name(name.as_str())
                    .data(data)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(*color))
                    .marker(Marker::Braille)
            })
            .collect();

        Chart::new(datasets)
            .block(Block::default().title("CPU Usage History (0-50%)"))
            .x_axis(
                Axis::default()
                    .bounds([0.0, 59.0])
                    .labels::<Vec<Span>>(vec![Span::raw("0"), Span::raw("30"), Span::raw("60")]),
            )
            .y_axis(
                Axis::default()
                    .bounds([y_min, y_max])
                    .labels::<Vec<Span>>(vec![
                        Span::raw("0"),
                        Span::raw(format!("{:.0}", y_max / 2.0)),
                        Span::raw(format!("{:.0}", y_max)),
                    ]),
            )
    };

    // Center the chart vertically and horizontally
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),     // Top padding
            Constraint::Length(12), // Increased chart height from 10 to 12
            Constraint::Min(1),     // Bottom padding
        ])
        .split(area);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),         // Left padding
            Constraint::Percentage(90), // Chart width
            Constraint::Min(1),         // Right padding
        ])
        .split(vertical_layout[1]);

    frame.render_widget(chart, horizontal_layout[1]);
}

const CORE_COLORS: &[Color] = &[
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Gray,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
];

fn render_process_section(
    frame: &mut Frame,
    processes: &[Process],
    selected_process: usize,
    scroll_offset: usize,
    area: Rect,
) {
    let block = Block::default()
        .title(" Process Information ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightMagenta));

    let inner_area = block.inner(area);
    let max_items = inner_area.height as usize - 2; // Account for header and border
    let scroll_offset = scroll_offset.min(processes.len().saturating_sub(max_items));
    let mut adjusted_scroll = scroll_offset;

    if selected_process < adjusted_scroll {
        adjusted_scroll = selected_process;
    } else if selected_process >= adjusted_scroll + max_items {
        adjusted_scroll = selected_process - max_items + 1;
    }

    // Define column constraints
    let widths = [
        Constraint::Length(6),  // PID
        Constraint::Length(15), // Name
        Constraint::Length(6),  // CPU%
        Constraint::Length(8),  // Memory
        Constraint::Length(8),  // Status
        Constraint::Length(6),  // Parent
    ];

    // Create header row
    let header = Row::new(vec![
        Cell::from(Span::styled(
            "PID",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "NAME",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "CPU%",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "MEMORY",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "STATUS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "PARENT",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .height(1)
    .bottom_margin(1);

    // Create table rows
    let rows = processes
        .iter()
        .enumerate()
        .skip(adjusted_scroll)
        .take(max_items)
        .map(|(i, process)| {
            let is_selected = i == selected_process;

            let style = if is_selected {
                make_highlight_style()
            } else {
                Style::default()
            };

            let status_str = match process.status {
                ProcessStatus::Run => "Running",
                ProcessStatus::Sleep => "Sleeping",
                ProcessStatus::Idle => "Idle",
                ProcessStatus::Zombie => "Zombie",
                ProcessStatus::Dead => "Dead",
                ProcessStatus::Stop => "Stopped",
                _ => "Unknown",
            };

            let parent_str = process
                .parent_pid
                .map_or("None".to_string(), |pid| pid.to_string());

            // Truncate name if needed
            let name = if process.name.len() > 15 {
                format!("{}...", &process.name[..12])
            } else {
                process.name.clone()
            };

            Row::new(vec![
                Cell::from(Span::styled(
                    process.pid.to_string(),
                    Style::default().fg(Color::Yellow),
                )),
                Cell::from(Span::styled(name, Style::default().fg(Color::Green))),
                Cell::from(Span::styled(
                    format!("{:.1}%", process.cpu_usage),
                    Style::default().fg(Color::Red),
                )),
                Cell::from(Span::styled(
                    format!("{:.2}MB", process.memory_mb),
                    Style::default().fg(Color::Blue),
                )),
                Cell::from(Span::styled(status_str, Style::default().fg(Color::Cyan))),
                Cell::from(Span::styled(
                    parent_str,
                    Style::default().fg(Color::Magenta),
                )),
            ])
            .style(style)
        });

    let table = Table::new(rows.collect::<Vec<_>>(), widths)
        .header(header)
        .block(block)
        .widths(&widths)
        .column_spacing(2)
        .row_highlight_style(make_highlight_style()) // Use your custom style here
        .highlight_symbol(">> ");

    let selected_position =
        if selected_process >= adjusted_scroll && selected_process < adjusted_scroll + max_items {
            Some(selected_process - adjusted_scroll)
        } else {
            None
        };

    frame.render_stateful_widget(
        table,
        area,
        &mut TableState::default().with_selected(selected_position),
    );
}

fn render_memory_section(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Memory Usage ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightYellow));

    frame.render_widget(block, area);
}

fn render_disk_section(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Disk Usage ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightBlue));

    frame.render_widget(block, area);
}
