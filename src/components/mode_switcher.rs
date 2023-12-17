use std::collections::HashMap;

use crate::actions::Action;
use crate::actions::EngineAction;
use crate::actions::HomeAction;
use crate::actions::ListNavDirection;
use crate::app::Mode;

use super::Component;
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use ratatui::prelude::*;
use ratatui::widgets::*;

lazy_static! {
  pub static ref MODES: Vec<(&'static str, Mode)> = vec![("Main Menu", Mode::MainMenu), ("Home", Mode::Home),];
}

#[derive(Default)]
pub struct ModeSwitcher {
  show_menu: bool,
  current_index: usize,
  mode_list_state: ListState,
}

impl ModeSwitcher {
  pub fn new(active_mode: Mode) -> Self {
    let index = MODES.iter().map(|(s, m)| m).enumerate().find(|(i, m)| **m == active_mode).map(|(i, m)| i).unwrap();

    Self { show_menu: false, current_index: index, mode_list_state: ListState::default().with_selected(Some(index)) }
  }

  fn select_mode(&mut self, offset: isize) -> Option<Action> {
    let new_index_option = self.current_index.checked_add_signed(offset).map(|ni| ni.clamp(0, MODES.len() - 1));

    match new_index_option {
      Some(ni) => self.current_index = ni,
      None => return None,
    };

    self.mode_list_state.select(Some(self.current_index));

    MODES.get(self.current_index).map(|(s, m)| EngineAction::ChangeMode(*m).into())
  }

  fn draw_menu(&mut self, f: &mut Frame, rect: Rect) {
    let location = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(20), Constraint::Ratio(1, 5), Constraint::Min(0)])
      .split(rect.inner(&Margin::new(1, 1)))[0];
    let location = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(10), Constraint::Min(MODES.len() as u16 + 5)])
      .split(location)[0];

    let background = Block::new()
      .light_blue()
      .on_black()
      .title("Select Mode")
      .borders(Borders::ALL)
      .title_alignment(Alignment::Left)
      .title_position(block::Position::Top);

    let mode_listitems: Vec<ListItem> = MODES.iter().map(|(s, m)| ListItem::new(*s)).collect();
    let list = List::new(mode_listitems)
      .style(Style::default())
      .highlight_style(Style::default().underlined())
      .highlight_symbol(">>")
      .block(background);

    f.render_widget(Clear, location);
    f.render_stateful_widget(list, location, &mut self.mode_list_state);
  }
}

impl Component for ModeSwitcher {
  fn update(&mut self, action: crate::actions::Action) -> Result<Option<crate::actions::Action>> {
    let new_action = match action {
      Action::Home(h) => match h {
        HomeAction::NavigateList(ListNavDirection::Up) => self.select_mode(-1),
        HomeAction::NavigateList(ListNavDirection::Down) => self.select_mode(1),
        _ => None,
      },
      Action::Engine(EngineAction::ToggleShowModeSwitcher) => {
        self.show_menu = !self.show_menu;
        None
      },
      _ => None,
    };

    Ok(new_action)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    if self.show_menu {
      self.draw_menu(f, rect);
    }

    Ok(())
  }
}
