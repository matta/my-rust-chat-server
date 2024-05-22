use crossterm::event::KeyEvent;
use ratatui::{prelude::Backend, Frame};

use crate::state_store::{action::Action, State};

pub trait Component {
    fn update_from_state(&mut self, state: &State);

    // Returns a name used to describe the component in the UI.
    fn name(&self) -> &str;

    fn handle_key_event(&mut self, key: KeyEvent) -> Action;
}

pub trait ComponentRender<Props> {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: Props);
}
