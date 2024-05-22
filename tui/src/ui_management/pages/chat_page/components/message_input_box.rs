use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::{Backend, Rect},
    style::Color,
    Frame,
};

use super::super::section::usage::{HasUsageInfo, UsageInfo, UsageInfoLine};
use crate::ui_management::components::{
    input_box::{self, InputBox},
    Component, ComponentRender,
};
use crate::{
    state_store::{action::Action, State},
    ui_management::pages::chat_page::section::SectionActivation,
};

struct Props {
    /// Active room that the user is chatting in
    active_room: Option<String>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Self {
            active_room: state.active_room.clone(),
        }
    }
}

pub struct MessageInputBox {
    /// State Mapped MessageInputBox Props
    props: Props,
    // Internal State for the Component
    pub input_box: InputBox,
}

impl MessageInputBox {
    pub(crate) fn new(state: &State) -> Self {
        Self {
            props: Props::from(state),
            input_box: InputBox::new(),
        }
    }

    fn submit_message(&mut self) -> Action {
        let mut ret = Action::None;
        if !self.input_box.is_empty() {
            ret = Action::SendMessage {
                content: String::from(self.input_box.text()),
            };
            self.input_box.reset();
        }
        ret
    }
}

impl Component for MessageInputBox {
    fn update_from_state(&mut self, state: &State) {
        self.props = Props::from(state);
    }

    fn name(&self) -> &str {
        "Message Input"
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if key.kind == KeyEventKind::Press && self.props.active_room.is_some() {
            let action = self.input_box.handle_key_event(key);
            assert_eq!(action, Action::None);

            if key.code == KeyCode::Enter {
                return self.submit_message();
            }
        }
        Action::None
    }
}

impl SectionActivation for MessageInputBox {
    fn activate(&mut self) {}

    fn deactivate(&mut self) {
        self.input_box.reset();
    }
}

pub struct RenderProps {
    pub area: Rect,
    pub border_color: Color,
    pub show_cursor: bool,
}

impl ComponentRender<RenderProps> for MessageInputBox {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, props: RenderProps) {
        self.input_box.render(
            frame,
            input_box::RenderProps {
                title: "Message Input".into(),
                area: props.area,
                border_color: props.border_color,
                show_cursor: props.show_cursor,
            },
        )
    }
}

impl HasUsageInfo for MessageInputBox {
    fn usage_info(&self) -> UsageInfo {
        if self.props.active_room.is_none() {
            UsageInfo {
                description: Some("You can not send a message until you enter a room.".into()),
                lines: vec![UsageInfoLine {
                    keys: vec!["Esc".into()],
                    description: "to cancel".into(),
                }],
            }
        } else {
            UsageInfo {
                description: Some("Type your message to send a message to the active room".into()),
                lines: vec![
                    UsageInfoLine {
                        keys: vec!["Esc".into()],
                        description: "to cancel".into(),
                    },
                    UsageInfoLine {
                        keys: vec!["Enter".into()],
                        description: "to send your message".into(),
                    },
                ],
            }
        }
    }
}
