use crossterm::event::KeyEvent;
use ratatui::{prelude::Backend, Frame};

use crate::state_store::{action::Action, ServerConnectionStatus, State};

use self::{chat_page::ChatPage, connect_page::ConnectPage};

use super::components::{Component, ComponentRender};

mod chat_page;
mod connect_page;

enum ActivePage {
    ChatPage,
    ConnectPage,
}

struct Props {
    active_page: ActivePage,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Props {
            active_page: match state.server_connection_status {
                ServerConnectionStatus::Connected { .. } => ActivePage::ChatPage,
                _ => ActivePage::ConnectPage,
            },
        }
    }
}

pub struct AppRouter {
    props: Props,
    //
    chat_page: ChatPage,
    connect_page: ConnectPage,
}

impl AppRouter {
    pub(crate) fn new(state: &State) -> Self
    where
        Self: Sized,
    {
        AppRouter {
            props: Props::from(state),
            chat_page: ChatPage::new(state),
            connect_page: ConnectPage::new(state),
        }
    }

    fn get_active_page_component(&self) -> &dyn Component {
        match self.props.active_page {
            ActivePage::ChatPage => &self.chat_page,
            ActivePage::ConnectPage => &self.connect_page,
        }
    }

    fn get_active_page_component_mut(&mut self) -> &mut dyn Component {
        match self.props.active_page {
            ActivePage::ChatPage => &mut self.chat_page,
            ActivePage::ConnectPage => &mut self.connect_page,
        }
    }
}

impl Component for AppRouter {
    fn update_from_state(&mut self, state: &State) {
        self.props = Props::from(state);
        self.chat_page.update_from_state(state);
        self.connect_page.update_from_state(state);
    }

    // route all functions to the active page
    fn name(&self) -> &str {
        self.get_active_page_component().name()
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Option<Action> {
        self.get_active_page_component_mut().handle_key_event(key)
    }
}

impl ComponentRender<()> for AppRouter {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: ()) {
        match self.props.active_page {
            ActivePage::ChatPage => self.chat_page.render(frame, props),
            ActivePage::ConnectPage => self.connect_page.render(frame, props),
        }
    }
}
