use crossterm::event::KeyEvent;
use ratatui::{prelude::Backend, Frame};

use crate::state_store::State;

pub trait Component {
    fn move_with_state(self, state: &State) -> Self
    where
        Self: Sized;

    // Returns a name used to describe the component in the UI.
    fn name(&self) -> &str;

    fn handle_key_event(&mut self, key: KeyEvent);
}

pub trait ComponentRender<Props> {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: Props);
}
