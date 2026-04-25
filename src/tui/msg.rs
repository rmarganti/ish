#![allow(dead_code)]

use crate::model::ish::Ish;
use crate::tui::effect::Effect;
use crate::tui::model::Status;

pub type MsgResult<T> = Result<T, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldEdit {
    Insert(char),
    Backspace,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveFailure {
    Conflict { id: String },
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSuccess {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorRequest {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    JumpTop,
    JumpBottom,
    HalfPageUp,
    HalfPageDown,
    OpenDetail,
    OpenStatusPicker,
    OpenCreateForm,
    OpenHelp,
    PopScreen,
    ConfirmDiscardCreateForm,
    CancelDiscardCreateForm,
    SubmitStatusChange,
    SubmitCreateForm,
    SubmitCreateAndEdit,
    EditCurrentIssue,
    RequestRefresh,
    FocusNextField,
    FocusPreviousField,
    CreateFormInput(FormFieldEdit),
    CreateFormCycleType(i32),
    CreateFormCyclePriority(i32),
    SubmitCreateFormWithStatus(Status),
    IssuesLoaded(MsgResult<Vec<Ish>>),
    SaveCompleted(SaveSuccess),
    SaveFailed(SaveFailure),
    EditorRequested(EditorRequest),
    EditorReturned(MsgResult<()>),
    Followup(Effect),
    Tick,
    Resize(u16, u16),
    Quit,
    DismissStatusLine,
}
