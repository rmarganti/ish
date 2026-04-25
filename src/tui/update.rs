#![allow(dead_code)]

use crate::tui::{Effect, Model, Msg};

pub fn update(model: Model, _msg: Msg) -> (Model, Vec<Effect>) {
    (model, Vec::new())
}
