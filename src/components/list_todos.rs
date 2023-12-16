use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

pub struct Todo {
  id: u32,
  title: &'static str,
  is_completed: bool,
}

#[derive(Default)]
pub struct TodosLister {
  selected_index: usize,
}

const TODOS_LIST: &[Todo] = &[
  Todo { id: 1, title: "Hello World!", is_completed: false },
  Todo { id: 1, title: "Already completed", is_completed: true },
];

impl TodosLister {
  pub fn new(selected_index: usize) -> Self {
    Self { selected_index }
  }

  pub fn todos_to_list(&self) -> List<'_> {
    let title_width = TODOS_LIST.iter().map(|t| t.title.width()).max().unwrap_or_default();

    let todos_list_items: Vec<ListItem<'_>> = TODOS_LIST
      .iter()
      .map(|t| {
        let title = format!("{:width$}", t.title, width = title_width).to_string();
        ListItem::new(match t.is_completed {
          true => Line::styled(title, Style::default().crossed_out()),
          false => Line::raw(title),
        })
      })
      .collect();

    List::new(todos_list_items)
  }
}

impl Widget for TodosLister {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let list = self.todos_to_list();
    let mut state = ListState::default().with_selected(Some(self.selected_index));

    StatefulWidget::render(list, area, buf, &mut state);
  }
}
