use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
  actions::{Action, EngineAction},
  components::{
    fps::FpsCounter, help_screen::HelpScreen, home::Home, main_menu::MainMenu, mode_switcher::ModeSwitcher, Component,
  },
  config::Config,
  tui,
};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
  #[default]
  MainMenu,
  Home,
}

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let mode = Mode::MainMenu;
    let main_menu = MainMenu::new();
    let home = Home::new();
    let fps = FpsCounter::new();
    let config = Config::new()?;
    let help_screen = HelpScreen::new(vec![mode]);
    let mode_switcher = ModeSwitcher::new(mode);

    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(main_menu), Box::new(help_screen), Box::new(mode_switcher)],
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);

    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init()?;
    }

    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(EngineAction::Quit.into())?,
          tui::Event::Tick => action_tx.send(EngineAction::Tick.into())?,
          tui::Event::Render => action_tx.send(EngineAction::Render.into())?,
          tui::Event::Resize(x, y) => action_tx.send(EngineAction::Resize(x, y).into())?,
          tui::Event::Key(key) => {
            if let Some(keymap) = self.config.keybindings.get(&self.mode) {
              if let Some(action) = keymap.get(&vec![key]) {
                log::info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
              } else {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                }
              }
            };
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.handle_events(Some(e.clone()))? {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != EngineAction::Tick.into() && action != EngineAction::Render.into() {
          log::debug!("{action:?}");
        }
        if let Action::Engine(engine_action) = &action {
          match engine_action {
            EngineAction::Tick => {
              self.last_tick_key_events.drain(..);
            },
            EngineAction::ChangeMode(m) => self.mode = *m,
            EngineAction::Quit => self.should_quit = true,
            EngineAction::Suspend => self.should_suspend = true,
            EngineAction::Resume => self.should_suspend = false,
            EngineAction::Resize(w, h) => {
              tui.resize(Rect::new(0, 0, *w, *h))?;
              tui.draw(|f| {
                for component in self.components.iter_mut() {
                  let r = component.draw(f, f.size());
                  if let Err(e) = r {
                    action_tx.send(EngineAction::Error(format!("Failed to draw: {:?}", e)).into()).unwrap();
                  }
                }
              })?;
            },
            EngineAction::Render => {
              tui.draw(|f| {
                for component in self.components.iter_mut() {
                  let r = component.draw(f, f.size());
                  if let Err(e) = r {
                    action_tx.send(EngineAction::Error(format!("Failed to draw: {:?}", e)).into()).unwrap();
                  }
                }
              })?;
            },
            _ => {},
          }
        }

        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(EngineAction::Resume.into())?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
