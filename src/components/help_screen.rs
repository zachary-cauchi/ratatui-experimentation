use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
  layout::{Constraint, Margin, Rect},
  style::*,
  text::*,
  widgets::*,
};

use crate::{
  actions::{Action, HomeAction},
  app::Mode,
  config::{key_event_to_string, Config},
  tui::Frame,
};

use super::Component;

#[derive(Default)]
pub struct HelpScreen {
  pub show_help: bool,
  watched_modes: Vec<Mode>,
  config: Config,
  state: TableState,
}

impl HelpScreen {
  pub fn new(watched_modes: Vec<Mode>) -> Self {
    Self { show_help: false, watched_modes, config: Config::default(), state: TableState::default() }
  }

  pub fn add_mode(&mut self, mode: Mode) {
    if !self.watched_modes.contains(&mode) {
      self.watched_modes.push(mode);
    }
  }

  fn draw_help(&mut self, f: &mut Frame, rect: &Rect) {
    let rect = rect.inner(&Margin { horizontal: 4, vertical: 4 });
    f.render_widget(Clear, rect);
    let block = Block::default()
      .title(Line::from(vec![Span::styled("Key Bindings", Style::default().add_modifier(Modifier::BOLD))]))
      .borders(Borders::ALL)
      .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, rect);

    // Map the keybindings to a vector of rows.
    // Each vector prints the key(s) and the action it performs.
    // TODO: Change Action printing to prettier format.
    let rows: Vec<Row> = self
      .watched_modes
      .iter()
      .map(|mode| (mode, self.config.keybindings.get(mode).unwrap().clone()))
      .flat_map(|(mode, bindings)| {
        let mut rows = vec![
          Row::new(vec![Cell::from("")]),
          Row::new(vec![Cell::from(format!("{mode:?}")).style(Style::default().underlined())]),
        ];

        bindings
          .iter()
          .map(|(key, val)| {
            Row::new(vec![
              key
                .iter()
                .map(key_event_to_string)
                .enumerate()
                .map(|(i, k)| match i {
                  0 => k,
                  _ => format!(", {}", k),
                })
                .collect(),
              format!("{val}"),
            ])
          })
          .for_each(|r| rows.push(r));

        rows
      })
      .collect();

    // Construct the final table.
    let table = Table::new(rows)
      .header(Row::new(vec!["Key", "Action"]).bottom_margin(1).style(Style::default().add_modifier(Modifier::BOLD)))
      .widths(&[Constraint::Percentage(10), Constraint::Percentage(90)])
      .column_spacing(1);

    let location = rect.inner(&Margin { vertical: 4, horizontal: 2 });
    f.render_widget(Clear, location);
    f.render_stateful_widget(table, location, &mut self.state);
  }
}

impl Component for HelpScreen {
  fn register_config_handler(&mut self, config: crate::config::Config) -> Result<()> {
    self.config = config;

    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if action == Action::Engine(crate::actions::engine_actions::EngineAction::ToggleShowHelp) {
      self.show_help = !self.show_help;
    }

    Ok(None)
  }

  fn draw(&mut self, f: &mut crate::tui::Frame<'_>, rect: ratatui::prelude::Rect) -> Result<()> {
    if self.show_help {
      self.draw_help(f, &rect)
    }

    Ok(())
  }
}
