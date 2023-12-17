use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum EngineAction {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  Refresh,
  ToggleShowHelp,
  Error(String),
}

impl Display for EngineAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Resize(x, y) => write!(f, "Resize({x}, {y})"),
      Self::Error(x) => write!(f, "Error({x:?})"),
      x => write!(f, "{:?}", x),
    }
  }
}
