use color_eyre::Result;
use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Paragraph};
use ratatui::{DefaultTerminal, Frame};

use crate::cpu::CpuInfo;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut cpu_info = CpuInfo::new();
    let result = run(terminal, &mut cpu_info);
    ratatui::restore();
    result
}

pub fn run(mut terminal: DefaultTerminal, cpu_info: &mut CpuInfo) -> Result<()> {
    loop {
        cpu_info.update();
        terminal.draw(|f| render(f, cpu_info))?;

        // Handle input with timeout to allow for refresh
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break Ok(());
                }
            }
        }
    }
}

fn render(frame: &mut Frame, cpu_info: &CpuInfo) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Top 40% for CPU
            Constraint::Percentage(60), // Bottom 60% (empty for now)
        ])
        .split(frame.area());

    render_cpu_section(frame, cpu_info, main_layout[0]);

    let bottom_section = Block::default().borders(Borders::ALL);

    frame.render_widget(bottom_section, main_layout[1]);
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
