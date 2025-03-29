use std::cmp;

use color_eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, List, ListItem, ListState,
    Paragraph,
};
use ratatui::{DefaultTerminal, Frame};
use sysinfo::ProcessStatus;

use crate::cpu::CpuInfo;
use crate::process::{self, get_all_processes, Process};

pub struct AppState {
    pub cpu_info: CpuInfo,
    pub processes: Vec<Process>,
    pub process_list_state: ListState,
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new() -> Self {
        let mut processes = process::get_all_processes();
        process::sort_by_memory(&mut processes);

        Self {
            cpu_info: CpuInfo::new(),
            processes,
            process_list_state: ListState::default(),
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
    let mut state = AppState::new();
    let result = run(terminal, &mut state);
    ratatui::restore();
    result
}

pub fn run(mut terminal: DefaultTerminal, state: &mut AppState) -> Result<()> {
    loop {
        state.cpu_info.update();
        state.update_processes();

        terminal.draw(|f| render(f, state))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.scroll_offset = state.scroll_offset.saturating_add(1);
                        let max_offset = state.processes.len().saturating_sub(1);
                        state.scroll_offset = cmp::min(state.scroll_offset, max_offset);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.scroll_offset = state.scroll_offset.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn render(frame: &mut Frame, state: &AppState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Top 40% for CPU
            Constraint::Percentage(60), // Bottom 60% (empty for now)
        ])
        .split(frame.area());

    render_cpu_section(frame, &state.cpu_info, main_layout[0]);
    render_process_section(frame, &state.processes, state.scroll_offset, main_layout[1]);

    // let bottom_section = Block::default().borders(Borders::ALL);
    //
    // frame.render_widget(bottom_section, main_layout[1]);
}

fn render_cpu_section(frame: &mut Frame, cpu_info: &CpuInfo, area: Rect) {
    let cpu_block = Block::default()
        .title("CPU usage")
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
    scroll_offset: usize,
    area: Rect,
) {
    let block = Block::default()
        .title(" Process Information ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightMagenta));

    let inner_area = block.inner(area);
    let max_items = inner_area.height as usize - 2; // -2 for borders kek

    // Max lengths for each column
    const PID_WIDTH: usize = 6;
    const NAME_WIDTH: usize = 15;
    const CPU_WIDTH: usize = 5;
    const MEM_WIDTH: usize = 6;
    const STATUS_WIDTH: usize = 8;
    const PARENT_WIDTH: usize = 6;

    // Create header with exact spacing
    let header = Line::from(vec![
        Span::styled(
            format!("{:>PID_WIDTH$}", "PID"),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:<NAME_WIDTH$}", "NAME"),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:>CPU_WIDTH$}", "CPU%"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:>MEM_WIDTH$}", "MEM"),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:<STATUS_WIDTH$}", "STATUS"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{:<PARENT_WIDTH$}", "PARENT"),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let items: Vec<ListItem> = processes
        .iter()
        .skip(scroll_offset)
        .take(max_items)
        .map(|process| {
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

            // Truncate strings that are too long
            let name = if process.name.len() > NAME_WIDTH {
                format!("{}...", &process.name[..NAME_WIDTH.saturating_sub(3)])
            } else {
                process.name.clone()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>PID_WIDTH$}", process.pid),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:<NAME_WIDTH$}", name),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:>CPU_WIDTH$.1}%", process.cpu_usage),
                    Style::default().fg(Color::Red),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:>MEM_WIDTH$.2}MB", process.memory_mb),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:<STATUS_WIDTH$}", status_str),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(
                    format!("{:<PARENT_WIDTH$}", parent_str),
                    Style::default().fg(Color::Magenta),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_widget(list, area);
}
