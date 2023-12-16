use std::fmt;

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};

use crate::actions::home_action::ListNavDirection;

use self::{engine_actions::EngineAction, home_action::HomeAction};

pub mod engine_actions;
pub mod home_action;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
  Engine(EngineAction),
  Home(HomeAction),
}

impl From<EngineAction> for Action {
  fn from(value: EngineAction) -> Self {
    Action::Engine(value)
  }
}

impl<'de> Deserialize<'de> for Action {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
      type Value = Action;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string (in the format \"<ActionName>[<param_1> ...]\") representation of Action")
      }

      fn visit_str<E>(self, value: &str) -> Result<Action, E>
      where
        E: de::Error,
      {
        match value {
          data if data.starts_with("Engine.") => {
            let substr: &str = data.split("Engine.").nth(1).unwrap_or_default();

            match substr {
              "Tick" => Ok(Action::Engine(EngineAction::Tick)),
              "Render" => Ok(Action::Engine(EngineAction::Render)),
              "Suspend" => Ok(Action::Engine(EngineAction::Suspend)),
              "Resume" => Ok(Action::Engine(EngineAction::Resume)),
              "Quit" => Ok(Action::Engine(EngineAction::Quit)),
              "Refresh" => Ok(Action::Engine(EngineAction::Refresh)),
              data if substr.starts_with("Error(") => {
                let error_msg = data.trim_start_matches("Error(").trim_end_matches(')');
                Ok(Action::Engine(EngineAction::Error(error_msg.to_string())))
              },
              data if substr.starts_with("Resize(") => {
                let parts: Vec<&str> = data.trim_start_matches("Resize(").trim_end_matches(')').split(',').collect();
                if parts.len() == 2 {
                  let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
                  let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
                  Ok(Action::Engine(EngineAction::Resize(width, height)))
                } else {
                  Err(E::custom(format!("Invalid Resize format: {}", value)))
                }
              },
              _ => Err(E::custom(format!("Unknown EngineAction variant: {}", value))),
            }
          },
          data if data.starts_with("Home.") => {
            let substr: &str = data.split("Home.").nth(1).unwrap_or_default();

            match substr {
              "Help" => Ok(Action::Home(HomeAction::Help)),
              "ScheduleIncrement" => Ok(Action::Home(HomeAction::ScheduleIncrement)),
              "ScheduleDecrement" => Ok(Action::Home(HomeAction::ScheduleDecrement)),
              "ToggleShowHelp" => Ok(Action::Home(HomeAction::ToggleShowHelp)),
              "EnterInsert" => Ok(Action::Home(HomeAction::EnterInsert)),
              "EnterNormal" => Ok(Action::Home(HomeAction::EnterNormal)),
              data if data.starts_with("NavigateList") => {
                let parts: Vec<&str> = data.split(&['(', ')']).collect();

                match parts[1] {
                  "Left" => Ok(Action::Home(HomeAction::NavigateList(ListNavDirection::Left))),
                  "Right" => Ok(Action::Home(HomeAction::NavigateList(ListNavDirection::Right))),
                  "Up" => Ok(Action::Home(HomeAction::NavigateList(ListNavDirection::Up))),
                  "Down" => Ok(Action::Home(HomeAction::NavigateList(ListNavDirection::Down))),
                  x => Err(E::custom(format!("Unexpected list navigation direction in config: {}", x))),
                }
              },
              _ => Err(E::custom(format!("Unknown HomeAction variant: {}", value))),
            }
          },
          _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
        }
      }
    }

    deserializer.deserialize_str(ActionVisitor)
  }
}
