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

#[derive(Default, Clone, Copy)]
struct MainMenuTabs {
  pub item_index: usize,
  pub is_item_selected: bool,
}

impl MainMenuTabs {
  pub fn navigate_list(&mut self, dir: ListNavDirection) {
    match (dir, self.item_index) {
      (ListNavDirection::Left, 0) => self.item_index = LIST_OPS.len() - 1,
      (ListNavDirection::Left, _) => self.item_index -= 1,
      (ListNavDirection::Right, _) => {
        self.item_index = if self.item_index == LIST_OPS.len() - 1 { 0 } else { self.item_index + 1 }
      },
      _ => {},
    }
  }
}

impl Widget for MainMenuTabs {
  fn render(self, area: Rect, buf: &mut Buffer) {
    Tabs::new(vec!["List", "Add", "Edit", "Delete"])
      .style(Style::default().white())
      .highlight_style(Style::default().yellow().on_blue().underlined())
      .select(self.item_index)
      .divider(symbols::DOT)
      .render(area, buf);
  }
}

#[derive(Default)]
pub struct MainMenu {
  pub show_help: bool,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<Vec<KeyEvent>, Action>,
  main_menu_tabs: MainMenuTabs,
}

impl MainMenu {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_keymap(&mut self, keymap: HashMap<Vec<KeyEvent>, Action>) {
    self.keymap = keymap;
  }

  pub fn render_main_menu_border(&mut self, buf: &mut Buffer, area: Rect) {
    Block::default().title("Main Menu").border_set(symbols::border::DOUBLE).borders(Borders::ALL).render(area, buf);
    Block::default()
      .title("Use 🞀 / 🞂 to navigate the top menu tabs.")
      .title_alignment(Alignment::Left)
      .title_position(block::Position::Bottom)
      .render(area.inner(&Margin::new(1, 0)), buf);
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
        self.main_menu_tabs.navigate_list(dir);
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let main_menu_inner = rect.inner(&Margin::new(1, 1));
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Length(1), Constraint::Min(0)])
      .split(main_menu_inner);

    self.render_main_menu_border(f.buffer_mut(), rect);
    f.render_widget(self.main_menu_tabs, chunks[0]);


    Ok(())
  }
}
