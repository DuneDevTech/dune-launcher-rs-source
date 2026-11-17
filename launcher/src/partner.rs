//! Dune Client: upstream's sponsor placement (a third-party hosting advert) is removed, together
//! with its pictures. The small API stays, empty, so the pages that used to call it need no
//! change and upstream merges stay simple. Nothing here ever draws anything or opens a link.
use crate::theme::Theme;
use gpui::{div, prelude::*, Div, Stateful};

/// No sponsor link.
pub const URL: &str = "";

/// Kept for `Settings::partner_seen_secs`; no advert is ever shown.
pub const SEEN_BEFORE_HIDE: u64 = 180;

/// Height the sidebar card used to need; nothing is drawn now.
pub const TALL_HEIGHT: f32 = 0.;

pub fn hide_button(id: &'static str, _t: &Theme) -> Stateful<Div> {
    div().id(id)
}

pub fn banner(id: &'static str, _width: f32, _t: &Theme) -> Stateful<Div> {
    div().id(id)
}

pub fn banner_image(id: &'static str, _width: f32, _t: &Theme) -> Stateful<Div> {
    div().id(id)
}

pub fn card(id: &'static str, _width: f32, _compact: bool, _t: &Theme) -> Stateful<Div> {
    div().id(id)
}
