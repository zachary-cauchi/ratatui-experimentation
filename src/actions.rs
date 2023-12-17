use std::fmt::{self, Display};

use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};

pub use crate::actions::home_action::ListNavDirection;

pub use self::{engine_actions::EngineAction, home_action::HomeAction};

pub mod engine_actions;
pub mod home_action;

macro_rules! extend_action {
  ( $x:ty, $y:ident ) => {
    impl From<$x> for Action {
      fn from(value: $x) -> Self {
        Action::$y(value)
      }
    }
  };
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
  Engine(EngineAction),
  Home(HomeAction),
}

impl Display for Action {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Engine(x) => write!(f, "Engine.{x}"),
      Self::Home(x) => write!(f, "Home.{x}"),
    }
  }
}

extend_action!(EngineAction, Engine);
extend_action!(HomeAction, Home);

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
              "Tick" => Ok(EngineAction::Tick.into()),
              "Render" => Ok(EngineAction::Render.into()),
              "Suspend" => Ok(EngineAction::Suspend.into()),
              "Resume" => Ok(EngineAction::Resume.into()),
              "Quit" => Ok(EngineAction::Quit.into()),
              "Refresh" => Ok(EngineAction::Refresh.into()),
              "ToggleShowHelp" => Ok(EngineAction::ToggleShowHelp.into()),
              data if substr.starts_with("Error(") => {
                let error_msg = data.trim_start_matches("Error(").trim_end_matches(')');
                Ok(EngineAction::Error(error_msg.to_string()).into())
              },
              data if substr.starts_with("Resize(") => {
                let parts: Vec<&str> = data.trim_start_matches("Resize(").trim_end_matches(')').split(',').collect();
                if parts.len() == 2 {
                  let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
                  let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
                  Ok(EngineAction::Resize(width, height).into())
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
              "Help" => Ok(HomeAction::Help.into()),
              "ScheduleIncrement" => Ok(HomeAction::ScheduleIncrement.into()),
              "ScheduleDecrement" => Ok(HomeAction::ScheduleDecrement.into()),
              "ToggleShowHelp" => Ok(HomeAction::ToggleShowHelp.into()),
              "EnterInsert" => Ok(HomeAction::EnterInsert.into()),
              "EnterNormal" => Ok(HomeAction::EnterNormal.into()),
              data if data.starts_with("NavigateList") => {
                let parts: Vec<&str> = data.split(&['(', ')']).collect();

                match parts[1] {
                  "Left" => Ok(HomeAction::NavigateList(ListNavDirection::Left).into()),
                  "Right" => Ok(HomeAction::NavigateList(ListNavDirection::Right).into()),
                  "Up" => Ok(HomeAction::NavigateList(ListNavDirection::Up).into()),
                  "Down" => Ok(HomeAction::NavigateList(ListNavDirection::Down).into()),
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
