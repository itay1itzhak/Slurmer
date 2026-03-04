use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::Path;

use crate::config::{save_config, SlurmerConfig};

pub struct SettingsPopup {
    pub visible: bool,
    pub input_mode: bool,
    pub slurm_logs_dir: String,
    pub valid: Option<bool>,
    pub status: String,
}

pub enum SettingsAction {
    None,
    Close,
    Saved,
}

impl SettingsPopup {
    pub fn new() -> Self {
        Self {
            visible: false,
            input_mode: false,
            slurm_logs_dir: String::new(),
            valid: None,
            status: String::new(),
        }
    }

    pub fn initialize(&mut self, current: Option<&str>) {
        self.slurm_logs_dir = current.unwrap_or("").to_string();
        self.validate();
        self.status.clear();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(Line::from("Settings").centered())
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::Black));
        frame.render_widget(block.clone(), area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        let title = "Slurm logs dir";
        let title = match self.valid {
            Some(true) => format!("{} ✓", title),
            Some(false) => format!("{} ✗ Invalid", title),
            None => title.to_string(),
        };
        let style = match (self.input_mode, self.valid) {
            (true, _) => Style::default().fg(Color::Cyan),
            (false, Some(false)) => Style::default().fg(Color::Red),
            _ => Style::default(),
        };

        let field = Paragraph::new(self.slurm_logs_dir.clone()).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(style),
        );
        frame.render_widget(field, chunks[0]);

        let status = Paragraph::new(self.status.clone())
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().title("Status").borders(Borders::ALL));
        frame.render_widget(status, chunks[1]);

        let help = Paragraph::new(
            "Enter: Edit | Ctrl+a: Save | Esc: Close"
        )
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(help, chunks[3]);

        if self.input_mode {
            frame.set_cursor_position(Position {
                x: chunks[0].x + 1 + self.slurm_logs_dir.len() as u16,
                y: chunks[0].y + 1,
            });
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SettingsAction {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => {
                self.visible = false;
                self.input_mode = false;
                return SettingsAction::Close;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                if self.valid == Some(true) {
                    let cfg = SlurmerConfig {
                        slurm_logs_dir: Some(self.slurm_logs_dir.trim().to_string()),
                    };
                    match save_config(&cfg) {
                        Ok(()) => {
                            self.status = "Saved".to_string();
                            return SettingsAction::Saved;
                        }
                        Err(e) => {
                            self.status = format!("Save failed: {}", e);
                        }
                    }
                } else {
                    self.status = "Not saved: invalid path".to_string();
                }
                return SettingsAction::None;
            }
            (_, KeyCode::Enter) => {
                self.input_mode = !self.input_mode;
                return SettingsAction::None;
            }
            _ => {}
        }

        if !self.input_mode {
            return SettingsAction::None;
        }

        match key.code {
            KeyCode::Char(c) => {
                self.slurm_logs_dir.push(c);
                self.validate();
            }
            KeyCode::Backspace => {
                let _ = self.slurm_logs_dir.pop();
                self.validate();
            }
            _ => {}
        }

        SettingsAction::None
    }

    pub fn current_value(&self) -> Option<&str> {
        let v = self.slurm_logs_dir.trim();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }

    fn validate(&mut self) {
        let v = self.slurm_logs_dir.trim();
        if v.is_empty() {
            self.valid = None;
            return;
        }
        self.valid = Some(Path::new(v).is_dir());
    }
}

