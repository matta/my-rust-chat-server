/// The set of actions that can be performed in the application's state store.
///
/// This enum represents the different types of actions that can be dispatched to the
/// state store, which will then update the application's state accordingly.
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// No action.
    /// 
    /// This is used when no action is needed. While we could model this as an
    /// `Option<Action>`, this would preclude marking the type as #[must_use],
    /// and thus invite code that accidentally ignores actions returned from functions.
    /// 
    /// See https://github.com/rust-lang/rust/issues/71368 for a more detailed
    /// discussion of this issue.
    None,
    ConnectToServerRequest { addr: String },
    SendMessage { content: String },
    SelectRoom { room: String },
    Exit,
}
