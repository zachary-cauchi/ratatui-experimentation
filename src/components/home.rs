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
  actions::{Action, EngineAction, HomeAction, ListNavDirection},
  config::{key_event_to_string, KeyBindings},
};

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
}

lazy_static! {
  pub static ref LIST_OPS: HashMap<&'static str, Action> = HashMap::from([
    ("List", HomeAction::ScheduleIncrement.into()),
    ("Add", HomeAction::ScheduleDecrement.into()),
    ("Edit", HomeAction::ScheduleIncrement.into()),
    ("Delete", HomeAction::ScheduleDecrement.into()),
  ]);
}

#[derive(Default)]
pub struct Home {
  pub counter: usize,
  pub app_ticker: usize,
  pub render_ticker: usize,
  pub mode: Mode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<Vec<KeyEvent>, Action>,
  pub text: Vec<String>,
  pub last_events: Vec<KeyEvent>,
  pub todo_op_index: usize,
}

impl Home {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_keymap(&mut self, keymap: HashMap<Vec<KeyEvent>, Action>) {
    self.keymap = keymap;
  }

  pub fn tick(&mut self) {
    log::info!("Tick");
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    log::debug!("Render Tick");
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn add(&mut self, s: String) {
    self.text.push(s)
  }

  pub fn schedule_increment(&mut self, i: usize) {
    let tx = self.action_tx.clone().unwrap();
    tokio::spawn(async move {
      tx.send(HomeAction::EnterProcessing.into()).unwrap();
      tx.send(HomeAction::Increment(i).into()).unwrap();
      tx.send(HomeAction::ExitProcessing.into()).unwrap();
    });
  }

  pub fn schedule_decrement(&mut self, i: usize) {
    let tx = self.action_tx.clone().unwrap();
    tokio::spawn(async move {
      tx.send(HomeAction::EnterProcessing.into()).unwrap();
      tx.send(HomeAction::Decrement(i).into()).unwrap();
      tx.send(HomeAction::ExitProcessing.into()).unwrap();
    });
  }

  pub fn increment(&mut self, i: usize) {
    self.counter = self.counter.saturating_add(i);
  }

  pub fn decrement(&mut self, i: usize) {
    self.counter = self.counter.saturating_sub(i);
  }

  pub fn navigate_list(&mut self, dir: ListNavDirection) {
    if self.mode == Mode::Normal {
      match (dir, self.todo_op_index) {
        (ListNavDirection::Left, 0) => self.todo_op_index = LIST_OPS.len() - 1,
        (ListNavDirection::Left, _) => self.todo_op_index -= 1,
        (ListNavDirection::Right, _) => {
          self.todo_op_index = if self.todo_op_index == LIST_OPS.len() - 1 { 0 } else { self.todo_op_index + 1 }
        },
        _ => {},
      };
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

impl Component for Home {
  fn register_config_handler(&mut self, config: crate::config::Config) -> Result<()> {
    self.set_keymap(config.keybindings.get(&crate::app::Mode::Home).unwrap().clone());

    Ok(())
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key);
    let action = match self.mode {
      Mode::Normal | Mode::Processing => return Ok(None),
      Mode::Insert => match key.code {
        KeyCode::Esc => HomeAction::EnterNormal.into(),
        KeyCode::Enter => {
          if let Some(sender) = &self.action_tx {
            if let Err(e) = sender.send(HomeAction::CompleteInput(self.input.value().to_string()).into()) {
              error!("Failed to send action: {:?}", e);
            }
          }
          HomeAction::EnterNormal.into()
        },
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          HomeAction::Update.into()
        },
      },
    };
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Engine(e) => match e {
        EngineAction::Tick => self.tick(),
        EngineAction::Render => self.render_tick(),
        _ => (),
      },
      Action::Home(h) => match h {
        HomeAction::ScheduleIncrement => self.schedule_increment(1),
        HomeAction::ScheduleDecrement => self.schedule_decrement(1),
        HomeAction::Increment(i) => self.increment(i),
        HomeAction::Decrement(i) => self.decrement(i),
        HomeAction::CompleteInput(s) => self.add(s),
        HomeAction::EnterNormal => {
          self.mode = Mode::Normal;
        },
        HomeAction::EnterInsert => {
          self.mode = Mode::Insert;
        },
        HomeAction::EnterProcessing => {
          self.mode = Mode::Processing;
        },
        HomeAction::NavigateList(dir) => {
          self.navigate_list(dir);
        },
        HomeAction::ExitProcessing => {
          // TODO: Make this go to previous mode instead
          self.mode = Mode::Normal;
        },
        _ => (),
      },
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {
    let rects = Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)].as_ref()).split(rect);

    let mut text: Vec<Line> = self.text.clone().iter().map(|l| Line::from(l.clone())).collect();
    text.insert(0, "".into());
    text.insert(0, "Type into input and hit enter to display here".dim().into());
    text.insert(0, "".into());
    text.insert(0, format!("Render Ticker: {}", self.render_ticker).into());
    text.insert(0, format!("App Ticker: {}", self.app_ticker).into());
    text.insert(0, format!("Counter: {}", self.counter).into());
    text.insert(0, "".into());
    text.insert(
      0,
      Line::from(vec![
        "Press ".into(),
        Span::styled("j", Style::default().fg(Color::Red)),
        " or ".into(),
        Span::styled("k", Style::default().fg(Color::Red)),
        " to ".into(),
        Span::styled("increment", Style::default().fg(Color::Yellow)),
        " or ".into(),
        Span::styled("decrement", Style::default().fg(Color::Yellow)),
        ".".into(),
      ]),
    );
    text.insert(0, "".into());

    f.render_widget(
      Paragraph::new(text)
        .block(
          Block::default()
            .title("ratatui async template")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(match self.mode {
              Mode::Processing => Style::default().fg(Color::Yellow),
              _ => Style::default(),
            })
            .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center),
      rects[0],
    );
    let width = rects[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = self.input.visual_scroll(width as usize);
    let input = Paragraph::new(self.input.value())
      .style(match self.mode {
        Mode::Insert => Style::default().fg(Color::Yellow),
        _ => Style::default(),
      })
      .scroll((0, scroll as u16))
      .block(Block::default().borders(Borders::ALL).title(Line::from(vec![
        Span::raw("Enter Input Mode "),
        Span::styled("(Press ", Style::default().fg(Color::DarkGray)),
        Span::styled("/", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to start, ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to save and exit, ", Style::default().fg(Color::DarkGray)),
        Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD).fg(Color::Gray)),
        Span::styled(" to exit without saving)", Style::default().fg(Color::DarkGray)),
      ])));
    f.render_widget(input, rects[1]);
    if self.mode == Mode::Insert {
      f.set_cursor((rects[1].x + 1 + self.input.cursor() as u16).min(rects[1].x + rects[1].width - 2), rects[1].y + 1)
    }

    f.render_widget(
      Block::default()
        .title(
          ratatui::widgets::block::Title::from(format!(
            "{:?}",
            &self.last_events.iter().map(key_event_to_string).collect::<Vec<_>>()
          ))
          .alignment(Alignment::Right),
        )
        .title_style(Style::default().add_modifier(Modifier::BOLD)),
      Rect { x: rect.x + 1, y: rect.height.saturating_sub(1), width: rect.width.saturating_sub(2), height: 1 },
    );

    self.draw_menu(f);

    Ok(())
  }
}
