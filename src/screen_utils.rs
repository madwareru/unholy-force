use macroquad::prelude::*;
use crate::screen_utils;

pub const TARGET_WIDTH: f32 = 640f32;
pub const TARGET_HEIGHT: f32 = 360f32;

pub fn screen_scaling_factor() -> f32 {
    let scaling_factor = (screen_width() / TARGET_WIDTH).trunc();
    let scaling_factor = scaling_factor.min((screen_height() / TARGET_HEIGHT).trunc());
    scaling_factor.max(1.0)
}

pub fn screen_origin_pos() -> (f32, f32) {
    let scaling_factor = screen_scaling_factor();
    let expected_width = TARGET_WIDTH * scaling_factor;
    let expected_height = TARGET_HEIGHT * scaling_factor;
    let origin_pos_x = (screen_width() - expected_width) / 2.0;
    let origin_pos_y = (screen_height() - expected_height) / 2.0;
    (origin_pos_x, origin_pos_y)
}

pub fn scaled_mouse_position() -> (f32, f32) {
    let scaling_factor = screen_scaling_factor();
    let (origin_pos_x, origin_pos_y) = screen_utils::screen_origin_pos();
    let mouse_x = (mouse_position().0 - origin_pos_x) / scaling_factor;
    let mouse_y = (mouse_position().1 - origin_pos_y) / scaling_factor;
    (mouse_x, mouse_y)
}