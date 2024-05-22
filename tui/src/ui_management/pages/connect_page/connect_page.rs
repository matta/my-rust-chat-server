use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{prelude::*, widgets::*, Frame};

use crate::{
    state_store::{action::Action, ServerConnectionStatus, State},
    ui_management::components::{
        input_box::{self, InputBox},
        Component, ComponentRender,
    },
};

struct Props {
    error_message: Option<String>,
}

impl From<&State> for Props {
    fn from(state: &State) -> Self {
        Props {
            error_message: if let ServerConnectionStatus::Errored { err } =
                &state.server_connection_status
            {
                Some(err.to_string())
            } else {
                None
            },
        }
    }
}

/// ConnectPage handles the connection to the server
pub struct ConnectPage {
    // Mapped Props from State
    props: Props,
    // Internal Components
    input_box: InputBox,
}

impl ConnectPage {
    pub(crate) fn new(state: &State) -> Self
    where
        Self: Sized,
    {
        let mut input_box = InputBox::new();
        input_box.set_text(DEFAULT_SERVER_ADDR);

        ConnectPage {
            props: Props::from(state),
            input_box,
        }
    }

    fn connect_to_server(&mut self) -> Action {
        if self.input_box.is_empty() {
            return Action::None;
        }

        Action::ConnectToServerRequest {
            addr: self.input_box.text().to_string(),
        }
    }
}

const DEFAULT_SERVER_ADDR: &str = "localhost:8080";

impl Component for ConnectPage {
    fn update_from_state(&mut self, state: &State) {
        self.props = Props::from(state);
    }

    fn name(&self) -> &str {
        "Connect Page"
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        let action = self.input_box.handle_key_event(key);
        assert_eq!(action, Action::None);

        if key.kind != KeyEventKind::Press {
            return Action::None;
        }

        match key.code {
            KeyCode::Enter => self.connect_to_server(),
            KeyCode::Char('q') => Action::Exit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Exit,
            _ => Action::None,
        }
    }
}

impl ComponentRender<()> for ConnectPage {
    fn render<B: Backend>(&self, frame: &mut Frame<B>, _props: ()) {
        let [_, vertical_centered, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Min(1),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(frame.size())
        else {
            panic!("The main layout should have 3 chunks")
        };

        let [_, both_centered, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Min(1),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(vertical_centered)
        else {
            panic!("The horizontal layout should have 3 chunks")
        };

        let [container_addr_input, container_help_text, container_error_message] =
            *Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(both_centered)
        else {
            panic!("The left layout should have 3 chunks")
        };

        self.input_box.render(
            frame,
            input_box::RenderProps {
                title: "Server Host and Port".into(),
                area: container_addr_input,
                border_color: Color::Yellow,
                show_cursor: true,
            },
        );

        let help_text = Paragraph::new(Text::from(Line::from(vec![
            "Press ".into(),
            "<Enter>".bold(),
            " to connect".into(),
        ])));
        frame.render_widget(help_text, container_help_text);

        let error_message = Paragraph::new(if let Some(err) = self.props.error_message.as_ref() {
            Text::from(format!("Error: {}", err.as_str()))
        } else {
            Text::from("")
        })
        .wrap(Wrap { trim: true })
        .style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::SLOW_BLINK | Modifier::ITALIC),
        );

        frame.render_widget(error_message, container_error_message);
    }
}
