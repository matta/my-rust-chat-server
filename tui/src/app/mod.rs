use std::{collections::HashMap, sync::Arc, time::Duration};

use comms::{command, event};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tokio::{
    net::tcp::OwnedWriteHalf,
    sync::{broadcast, RwLock},
};
use tokio_stream::StreamExt;

use crate::client::{BoxedStream, CommandWriter};

use self::termination::{Interrupted, Terminator};

pub(crate) mod termination;

pub(crate) enum InputMode {
    Normal,
    Editing,
}

pub(crate) struct RoomState {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) joined: bool,
}

impl RoomState {
    pub(crate) fn new(name: String, description: String, joined: bool) -> RoomState {
        RoomState {
            name,
            description,
            joined,
        }
    }
}

pub(crate) enum MessageBoxItem {
    Message { username: String, content: String },
    Notification(String),
}

/// App holds the state of the application
pub(crate) struct App {
    /// Command Writer is used to send commands
    command_writer: CommandWriter<OwnedWriteHalf>,
    /// Terminator is used to send the kill signal to the application
    terminator: Terminator,
    /// The name of the user
    pub(crate) username: String,
    /// The list of rooms the user can participate in and their status
    pub(crate) rooms: Vec<RoomState>,
    // The active room which the user has selected
    pub(crate) active_room: String,
    /// Current value of the input box
    pub(crate) input: String,
    /// Position of cursor in the editor area.
    pub(crate) cursor_position: usize,
    /// Current input mode
    pub(crate) input_mode: InputMode,
    /// History of recorded messages
    pub(crate) messages: HashMap<String, Vec<MessageBoxItem>>,
    /// Timer since app was open
    pub(crate) timer: usize,
}

impl App {
    pub fn new(command_writer: CommandWriter<OwnedWriteHalf>, terminator: Terminator) -> App {
        App {
            command_writer,
            terminator,
            username: String::new(),
            active_room: String::from("general"),
            rooms: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: HashMap::new(),
            cursor_position: 0,
            timer: 0,
        }
    }

    pub(crate) async fn handle_key_event(&mut self, key: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('e') => {
                    self.input_mode = InputMode::Editing;
                }
                KeyCode::Char('q') => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let _ = self.terminator.terminate(Interrupted::UserInt);
                }
                _ => {}
            },
            InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => self.submit_message().await,
                KeyCode::Char(to_insert) => {
                    self.enter_char(to_insert);
                }
                KeyCode::Backspace => {
                    self.delete_char();
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_server_event(&mut self, event: &event::Event) {
        match event {
            event::Event::LoginSuccessful(event) => {
                self.username = event.username.clone();
                self.rooms = event
                    .rooms
                    .clone()
                    .into_iter()
                    .map(|r| RoomState::new(r.name, r.description, false))
                    .collect();
                self.messages = event
                    .rooms
                    .clone()
                    .into_iter()
                    .map(|r| (r.name, Vec::new()))
                    .collect();
            }
            event::Event::RoomParticipation(event) => {
                if event.username == self.username {
                    let room = self
                        .rooms
                        .iter_mut()
                        .find(|r| r.name == event.room)
                        .expect("room not found");

                    room.joined = match event.status {
                        event::RoomParticipationStatus::Joined => true,
                        event::RoomParticipationStatus::Left => false,
                    };
                }

                self.messages
                    .get_mut(&event.room)
                    .unwrap()
                    .push(MessageBoxItem::Notification(format!(
                        "{} has {} the room",
                        event.username,
                        match event.status {
                            event::RoomParticipationStatus::Joined => "joined",
                            event::RoomParticipationStatus::Left => "left",
                        }
                    )));
            }
            event::Event::UserMessage(event) => {
                self.messages
                    .get_mut(&event.room)
                    .unwrap()
                    .push(MessageBoxItem::Message {
                        username: event.username.clone(),
                        content: event.content.clone(),
                    });
            }
        }
    }

    fn increment_timer(&mut self) {
        self.timer += 1;
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    async fn submit_message(&mut self) {
        // TODO: handle the promise
        let _ = self
            .command_writer
            .write(&command::UserCommand::SendMessage(
                command::SendMessageCommand {
                    room: "general".to_string(),
                    content: self.input.clone(),
                },
            ))
            .await;

        self.input.clear();
        self.reset_cursor();
    }
}

pub(crate) async fn main_loop(
    mut interrupt_rx: broadcast::Receiver<Interrupted>,
    mut event_stream: BoxedStream<anyhow::Result<event::Event>>,
    app: Arc<RwLock<App>>,
) -> anyhow::Result<Interrupted> {
    let mut ticker = tokio::time::interval(Duration::from_secs(1));

    let result = loop {
        tokio::select! {
            Some(Ok(event)) = event_stream.next() => {
                let mut app = app.write().await;

                app.handle_server_event(&event);
            }
            // Tick to terminate the select every N milliseconds
            _ = ticker.tick() => {
                let mut app = app.write().await;

                app.increment_timer();
            },
            // Catch and handle interrupt signal to gracefully shutdown
            Ok(interrupted) = interrupt_rx.recv() => {
                break interrupted;
            }
        }
    };

    Ok(result)
}