use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
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
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, cpu_info: &CpuInfo) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Top 30% for CPU
            Constraint::Percentage(70), // Bottom 70% (will be empty for now)
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

pub fn _render_cpu_graphs(_frame: &mut Frame, _cpu_info: &CpuInfo, _area: Rect) {}

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
