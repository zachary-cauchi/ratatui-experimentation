use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::lazy_static;
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{
  action::{Action, ListNavDirection},
  config::{key_event_to_string, KeyBindings},
};

lazy_static! {
  pub static ref LIST_OPS: HashMap<&'static str, Action> = HashMap::from([
    ("List", Action::ScheduleIncrement),
    ("Add", Action::ScheduleDecrement),
    ("Edit", Action::ScheduleIncrement),
    ("Delete", Action::ScheduleDecrement),
  ]);
}

#[derive(Default)]
pub struct MainMenu {
  pub show_help: bool,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<Vec<KeyEvent>, Action>,
  pub todo_op_index: usize,
}

impl MainMenu {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_keymap(&mut self, keymap: HashMap<Vec<KeyEvent>, Action>) {
    self.keymap = keymap;
  }

  pub fn navigate_list(&mut self, dir: ListNavDirection) {
    match (dir, self.todo_op_index) {
      (ListNavDirection::Left, 0) => self.todo_op_index = LIST_OPS.len() - 1,
      (ListNavDirection::Left, _) => self.todo_op_index -= 1,
      (ListNavDirection::Right, _) => {
        self.todo_op_index = if self.todo_op_index == LIST_OPS.len() - 1 { 0 } else { self.todo_op_index + 1 }
      },
      _ => {},
    }
  }

  fn draw_menu(&self, f: &mut Frame) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .margin(1)
      .constraints([Constraint::Min(0), Constraint::Length(3)])
      .split(f.size());

    let tabs = Tabs::new(vec!["List", "View", "Edit", "Delete"])
      .block(Block::default().title("List operations").borders(Borders::TOP))
      .style(Style::default().white())
      .highlight_style(Style::default().yellow().on_blue().underlined())
      .select(self.todo_op_index)
      .divider(symbols::DOT);

    f.render_widget(tabs, chunks[0]);
  }
}

impl Component for MainMenu {
  fn register_config_handler(&mut self, config: crate::config::Config) -> Result<()> {
    self.set_keymap(config.keybindings.get(&crate::app::Mode::MainMenu).unwrap().clone());

    Ok(())
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::NavigateList(dir) => {
        self.navigate_list(dir);
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref()).split(rect);

    self.draw_menu(f);

    Ok(())
  }
}
