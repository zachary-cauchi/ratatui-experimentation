use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ListNavDirection {
  Left,
  Right,
  Up,
  Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum HomeAction {
  Help,
  ToggleShowHelp,
  ScheduleIncrement,
  ScheduleDecrement,
  Increment(usize),
  Decrement(usize),
  CompleteInput(String),
  EnterNormal,
  EnterInsert,
  EnterProcessing,
  ExitProcessing,
  Update,
  NavigateList(ListNavDirection),
}

impl Display for ListNavDirection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "NavigateList({})",
      match self {
        Self::Up => "🞁",
        Self::Down => "🞃",
        Self::Left => "🞀",
        Self::Right => "🞂",
      }
    )
  }
}

impl Display for HomeAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Increment(x) => write!(f, "Increment({x})"),
      Self::Decrement(x) => write!(f, "Decrement({x})"),
      Self::CompleteInput(x) => write!(f, "CompleteInput({x})"),
      Self::NavigateList(x) => write!(f, "NavigateList.{x:?}"),
      x => write!(f, "{:?}", x),
    }
  }
}
