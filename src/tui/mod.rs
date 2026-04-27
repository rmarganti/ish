pub mod editor;
pub mod effect;
pub mod keymap;
pub mod model;
pub mod msg;
pub mod runtime;
pub mod theme;
pub mod update;
pub mod view;

#[cfg(test)]
mod effect_integration;

#[allow(unused_imports)]
pub use effect::{Effect, IssueDraft, IssuePatch};
#[allow(unused_imports)]
pub use model::{
    BOARD_COLUMNS, BoardState, ConfigHandle, CreateFormState, DetailState, HelpState, IshType,
    Model, PickerState, Priority, PriorityPickerState, Screen, Severity, Status, StatusLine,
};
#[allow(unused_imports)]
pub use msg::{EditorRequest, FormFieldEdit, Msg, MsgResult, SaveFailure, SaveSuccess};

use crate::app::{AppContext, AppError};

pub fn run(ctx: AppContext) -> Result<(), AppError> {
    runtime::run(ctx)
}
