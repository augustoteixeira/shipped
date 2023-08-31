use async_trait::async_trait;
use macroquad::prelude::*;

use super::ui::{split, trim_margins, Button, ButtonPanel, Input, Rect, Ui};

#[derive(Debug)]
pub enum ViewState {
  Paused,
}

#[derive(Debug)]
pub struct View {
  rect: Rect,
  state: ViewState,
  panel: ButtonPanel<Command>,
}

#[derive(Clone, Debug)]
pub enum Command {
  Exit,
}

impl View {
  fn build_panel(&self, rect: &Rect) -> ButtonPanel<Command> {
    let mut panel: ButtonPanel<Command> =
      ButtonPanel::new(self.rect.clone(), (vec![], vec![], vec![], vec![], vec![]));
    match &self.state {
      ViewState::Paused => {
        let rects: Vec<Rect> = split(
          &trim_margins(self.rect.clone(), 0.4, 0.4, 0.4, 0.4),
          vec![0.0, 1.0],
          vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0],
        );
        panel.push(Button::<Command>::new(
          rects[0].clone(),
          ("No levels".to_string(), Command::Exit, false, false),
        ));
      }
    }
    panel
  }
}

#[async_trait]
impl Ui for View {
  type Command = ();
  type Builder = ();

  fn new(rect: Rect, _: ()) -> Self {
    View {
      rect: rect.clone(),
      state: ViewState::Paused,
      panel: ButtonPanel::new(rect, (vec![], vec![], vec![], vec![], vec![])),
    }
  }
  async fn draw(&self) {
    match &self.state {
      ViewState::Paused => {
        self.panel.draw().await;
      }
    }
  }
  fn process_input(&mut self, input: Input) -> Option<()> {
    match &mut self.state {
      ViewState::Paused => {
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
          return Some(());
        }
        match self.panel.process_input(input) {
          Some(Command::Exit) => Some(()),
          _ => None,
        }
      }
    }
  }
}
