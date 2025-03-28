use color_eyre::Result;
use ratatui::crossterm::event::{self, Event};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::{DefaultTerminal, Frame};

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

fn render(frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4), // Top 25% for CPU
            Constraint::Ratio(3, 4), // Bottom 75% (will be empty for now)
        ])
        .split(frame.area());

    let cpu_section = Block::default()
        .title("CPU usage")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightCyan))
        .style(Style::default());

    frame.render_widget(cpu_section, layout[0]);

    let bottom_section = Block::default().borders(Borders::ALL);

    frame.render_widget(bottom_section, layout[1]);
}

