pub mod effect;
pub mod keymap;
pub mod model;
pub mod msg;
pub mod runtime;
pub mod theme;
pub mod update;
pub mod view;

use crate::app::{AppContext, AppError};

pub fn run(ctx: &AppContext) -> Result<(), AppError> {
    runtime::run(ctx)
}
