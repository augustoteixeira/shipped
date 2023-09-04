use async_trait::async_trait;
use macroquad::prelude::*;

use super::load_bf::LoadBF;
use super::new_bf::NewBF;
use super::ui::{trim_margins, Button, Grid, Input, Rect, Ui};

#[derive(Clone, Debug)]
enum Selection {
  LoadBF,
  NewBF,
  UploadCode,
  Credits,
  Quit,
}

pub struct LandingSelection {
  buttons: Grid<1, 5, Button<Selection>>,
}

pub enum LandingState {
  Selection(LandingSelection),
  NewBF(NewBF),
  LoadBF(LoadBF),
  Credits(Credits),
}

pub struct Landing {
  rect: Rect,
  state: LandingState,
}

#[derive(Clone)]
pub enum LandingCommand {
  Exit,
}

fn button_grid(rect: Rect) -> Grid<1, 5, Button<Selection>> {
  Grid::new(
    trim_margins(rect, 0.3, 0.3, 0.3, 0.3),
    [
      [(
        "Load Battlefield".to_string(),
        Selection::LoadBF,
        true,
        false,
      )],
      [(
        "Create Battlefield".to_string(),
        Selection::NewBF,
        true,
        false,
      )],
      [(
        "Upload Code".to_string(),
        Selection::UploadCode,
        true,
        false,
      )],
      [("Credits".to_string(), Selection::Credits, true, false)],
      [("Quit".to_string(), Selection::Quit, true, false)],
    ],
  )
}

#[async_trait]
impl Ui for Landing {
  type Command = LandingCommand;
  type Builder = ();

  fn new(rect: Rect, _: ()) -> Self {
    Landing {
      rect: rect.clone(),
      //state: LandingState::NewBF(NewBF::new(rect.clone(), ())),
      state: LandingState::LoadBF(LoadBF::new(rect.clone(), ())),
      //state: LandingState::Selection(LandingSelection {
      //  buttons: button_grid(rect),
      //}),
    }
  }
  async fn draw(&self) {
    match &self.state {
      LandingState::Selection(s) => s.buttons.draw().await,
      LandingState::Credits(c) => c.draw().await,
      LandingState::NewBF(n) => n.draw().await,
      LandingState::LoadBF(l) => l.draw().await,
    }
  }
  fn process_input(&mut self, input: Input) -> Option<LandingCommand> {
    match &mut self.state {
      LandingState::Selection(s) => {
        if let Input::Key(KeyCode::Escape) | Input::Key(KeyCode::Q) = input {
          return Some(LandingCommand::Exit);
        }
        match s.buttons.process_input(input) {
          Some(Selection::Quit) => Some(LandingCommand::Exit),
          Some(Selection::Credits) => {
            self.state =
              LandingState::Credits(Credits::new(self.rect.clone(), "Hi there!".to_string()));
            None
          }
          Some(Selection::NewBF) => {
            self.state = LandingState::NewBF(NewBF::new(self.rect.clone(), None));
            None
          }
          Some(Selection::LoadBF) => {
            self.state = LandingState::LoadBF(LoadBF::new(self.rect.clone(), ()));
            None
          }
          _ => None,
        }
      }
      LandingState::Credits(c) => {
        match c.process_input(input) {
          Some(()) => {
            self.state = LandingState::Selection(LandingSelection {
              buttons: button_grid(self.rect.clone()),
            })
          }
          _ => {}
        }
        None
      }
      LandingState::NewBF(n) => {
        match n.process_input(input) {
          Some(()) => {
            self.state = LandingState::Selection(LandingSelection {
              buttons: button_grid(self.rect.clone()),
            })
          }
          _ => {}
        }
        None
      }
      LandingState::LoadBF(l) => {
        match l.process_input(input) {
          Some(()) => {
            self.state = LandingState::Selection(LandingSelection {
              buttons: button_grid(self.rect.clone()),
            })
          }
          _ => {}
        }
        None
      }
    }
  }
}

pub struct Credits {
  text: String,
}

#[async_trait]
impl Ui for Credits {
  type Command = ();
  type Builder = String;

  fn new(_: Rect, string: String) -> Self {
    Credits { text: string }
  }
  async fn draw(&self) {
    draw_text(self.text.as_str(), 200.0, 200.0, 40.0, DARKGREEN);
  }
  fn process_input(&mut self, _: Input) -> Option<()> {
    Some(())
  }
}
