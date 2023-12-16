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
