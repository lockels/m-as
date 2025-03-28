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
    let result = run(terminal);
    ratatui::restore();
    result
}

pub fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame, cpu_info: &CpuInfo) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4), // Top 25% for CPU
            Constraint::Ratio(3, 4), // Bottom 75% (will be empty for now)
        ])
        .split(frame.area());

    render_cpu_section(frame, cpu_info, main_layout[1]);

    let bottom_section = Block::default().borders(Borders::ALL);

    frame.render_widget(bottom_section, main_layout[1]);
}

fn render_cpu_section(frame: &mut Frame, cpu_info: &CpuInfo, area: Rect) {
    let cpu_section = Block::default()
        .title("CPU usage")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightCyan))
        .style(Style::default());

    let cpu_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_cpu_cores_list(frame, cpu_info, cpu_layout[0]);
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

    let list_widget = Paragraph::new(cores_list)
        .block(Block::default().style(Style::default().bg(Color::Black)))
        .alignment(Alignment::Left);

    frame.render_widget(list_widget, area);
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
