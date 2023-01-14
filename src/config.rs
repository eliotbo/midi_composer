use iced::{Size, Vector};

pub const RESIZE_BOX_PIXEL_WIDTH: f32 = 5.0;
pub static NOTE_LABELS: [&'static str; 12] =
    ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

pub const BEAT_SIZE: f32 = 60.0;
pub const NOTE_SIZE: f32 = 15.0;

pub const MIN_SCALING: Vector = Vector::new(0.1, 0.1);
pub const MAX_SCALING: Vector = Vector::new(2.0, 2.0);

pub const INIT_SCALING: Vector = Vector::new(1.0, 1.0);
pub const INIT_GRID_SIZE: Size = Size::new(500.0, 300.0);
