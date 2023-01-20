use iced::widget::canvas::{Cache, Cursor, Frame, Geometry, Text};
use iced::{alignment, Color, Point, Rectangle, Size, Vector};

use std::ops::RangeInclusive;

use crate::config::{
    BEAT_SIZE, INIT_GRID_SIZE, INIT_PITCH_POS, INIT_SCALING, NOTE_LABELS, NOTE_SIZE,
};
use crate::note::scale::Scale;

pub const IS_WHITE_KEY: [bool; 12] =
    [true, false, true, false, true, true, false, true, false, true, false, true];

#[derive(Clone)]
pub struct Grid {
    pub translation: Vector,
    pub scaling: Vector, // geometrical scale
    pub max_beats: usize,
    pub scale: Scale, // musical scale
}

impl Default for Grid {
    fn default() -> Self {
        let scale = Scale::default();

        Self {
            translation: Vector::new(
                -INIT_GRID_SIZE.width / 2.0 - BEAT_SIZE,
                // -1.0 * INIT_GRID_SIZE.height / 2.0 - NOTE_SIZE * 1.0,
                -1.0 * INIT_GRID_SIZE.height / 2.0 - NOTE_SIZE * INIT_PITCH_POS,
            ),
            scaling: INIT_SCALING,
            max_beats: 100,
            scale,
        }
    }
}

impl Grid {
    // pub fn to_track_axes(&self, point: Point, size: &Size) -> Point {
    //     let region = self.visible_region(*size);

    //     let projection =
    //         Point::new(point.x / self.scaling.x + region.x, point.y / self.scaling.y + region.y);

    //     let x = projection.x / BEAT_SIZE as f32;
    //     let y = 127.0 - projection.y / NOTE_SIZE as f32;

    //     Point::new(x, y)
    // }

    // pub fn to_track_axes(&self, point: Point, region: &Region) -> Point {
    // pub fn to_track_axes(&self, point: Point, size: &Size) -> Point {

    pub fn to_track_axes(&self, point: Point, frame_size: &Size) -> Point {
        let region = self.visible_region(*frame_size);

        // between 0 and 1.
        // The y axis is inverted, so the 0 value is at the bottom of the screen
        let mut cursor_normalized =
            Point::new(point.x / frame_size.width, 1.0 - point.y / frame_size.height);

        // relative to frame axes
        cursor_normalized.x = region.x + cursor_normalized.x * frame_size.width / self.scaling.x;
        cursor_normalized.y = region.y + cursor_normalized.y * frame_size.height / self.scaling.y;

        // relative to grid units
        cursor_normalized.x /= BEAT_SIZE as f32;
        cursor_normalized.y /= NOTE_SIZE as f32;

        cursor_normalized
    }

    // // Takes in a point (ex: cursor position) and modifies it accodring to the music scale.
    // // If the scale is chromatic, nothing changes, but if the scale is anything else, the
    // // y value will skip over notes that are not in the scale.
    // pub fn adjust_to_music_scale(&self, mut point: Point) -> Point {
    //     let y_whole = point.y.floor();
    //     let y_frac = point.y - y_whole;
    //     point.y = self.scale.midi_range[y_whole as usize] as f32 + y_frac;
    //     point
    // }

    // Takes in a point (ex: cursor position) and modifies it accodring to the music scale.
    // If the scale is chromatic, nothing changes, but if the scale is anything else, the
    // y value will skip over notes that are not in the scale.
    pub fn adjust_to_music_scale(&self, mut point: Point) -> Point {
        println!("adjust_to_music_scale: {:?}", point);

        let y_whole = point.y.abs().floor();

        let scale_size = self.scale.midi_size() as i16;

        let a = (y_whole as i16 / scale_size) * scale_size;
        let b = y_whole as i16 % scale_size;

        let y_frac = point.y.abs() - y_whole;

        let p_sign = if point.y >= 0.0 { 1.0 } else { -1.0 };
        point.y = p_sign * (a as f32 + self.scale.midi_range[b as usize] as f32 + y_frac);
        point
    }

    pub fn adjust_frame(&self, frame: &mut Frame, size: &Size) {
        let negative_translation = Vector::new(self.translation.x, -self.translation.y);
        let center = Vector::new(size.width / 2.0, size.height / 2.0);
        frame.translate(center);
        frame.scale(self.scaling);
        frame.translate(negative_translation);
        frame.scale(Vector::new(BEAT_SIZE, -NOTE_SIZE));
    }

    pub fn draw_background(&self, bounds: Rectangle, grid_cache: &Cache) -> Geometry {
        let grid = grid_cache.draw(bounds.size(), |frame| {
            self.adjust_frame(frame, &bounds.size());

            let region = self.visible_region(frame.size());

            // TODO: line width should be scaled with the zoom level?
            let rows = region.rows();

            let columns = region.columns();
            let (total_rows, total_columns) = (rows.clone().count(), columns.clone().count());
            let beat_linewidth = 2.0 / BEAT_SIZE;
            let note_linewidth = 2.0 / NOTE_SIZE;
            let color = Color::from_rgb8(70, 74, 83);

            let text_size = 14.0;

            let alpha = 0.25;

            for row in region.rows() {
                let note_index = self.scale.midi_range[row as usize];

                let pos = Point::new(*columns.start() as f32, row as f32);
                frame.fill_rectangle(pos, Size::new(total_columns as f32, note_linewidth), color);

                let mut note_color = Color::from_rgba8(100, 100, 100, alpha);

                if !IS_WHITE_KEY[note_index as usize % 12] {
                    note_color = Color::from_rgba8(10, 10, 10, alpha);
                };

                frame.fill_rectangle(pos, Size::new(total_columns as f32, 1.0), note_color);

                if (note_index as i32) % 12 == 0 {
                    // if true {
                    let text_pos = Point::new(
                        region.x / BEAT_SIZE as f32 + text_size * 0.2 / self.scaling.y / BEAT_SIZE,
                        row as f32 + 0.5,
                    );

                    let note_label = Text {
                        color: Color::WHITE,
                        size: text_size,
                        position: text_pos,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Center,
                        ..Text::default()
                    };

                    let note_name = NOTE_LABELS[note_index as usize % 12];
                    frame.fill_text(Text {
                        content: format!(
                            "{}{}",
                            note_name,
                            -2.0 + (note_index as f32 / 12 as f32).floor()
                        ),
                        ..note_label
                    });
                }
            }

            for column in region.columns() {
                let pos = Point::new(column as f32, *rows.start() as f32);
                frame.fill_rectangle(pos, Size::new(beat_linewidth, total_rows as f32), color);

                if column as i32 % 2 == 1 {
                    frame.fill_rectangle(
                        pos,
                        Size::new(1.0, total_rows as f32),
                        Color::from_rgba8(100, 74, 83, alpha),
                    );
                }

                let text_pos = Point::new(
                    column as f32 + 0.07 / self.scaling.x,
                    region.y / NOTE_SIZE as f32 + text_size * 0.0 / NOTE_SIZE / self.scaling.x,
                );

                let beat_label = Text {
                    color: Color::WHITE,
                    size: text_size,
                    position: text_pos,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    ..Text::default()
                };

                frame.fill_text(Text { content: format!("{:?}", column), ..beat_label });
            }
        });

        grid
    }

    pub fn draw_text_and_hover_overlay(&self, bounds: Rectangle, cursor: Cursor) -> Geometry {
        let overlay = {
            let mut frame = Frame::new(bounds.size());

            let hovered_cell = cursor
                .position_in(&bounds)
                .map(|position| Cell::at(self.project(position, frame.size())));

            let text = Text {
                color: Color::WHITE,
                size: 14.0,
                position: Point::new(frame.width(), frame.height()),
                horizontal_alignment: alignment::Horizontal::Right,
                vertical_alignment: alignment::Vertical::Bottom,
                ..Text::default()
            };

            if let Some(cell) = hovered_cell {
                frame.fill_text(Text {
                    content: format!("({}\t, {})", cell.note_name, cell.j,),
                    position: text.position - Vector::new(0.0, 16.0),
                    ..text
                });
            }

            frame.into_geometry()
        };

        overlay
    }

    pub fn visible_region(&self, size: Size) -> Region {
        let width = size.width / self.scaling.x;
        let height = size.height / self.scaling.y;

        Region {
            x: -self.translation.x - width / 2.0,
            y: -self.translation.y - height / 2.0,
            width,
            height: height,
        }
    }

    pub fn project(&self, position: Point, size: Size) -> Point {
        let region = self.visible_region(size);

        Point::new(position.x / self.scaling.x + region.x, position.y / self.scaling.y + region.y)
    }

    pub fn limit_to_bounds(
        &self,
        new_translation: &mut Vector,
        bounds: Rectangle,

        scaling: Vector, // NOTE: cannot use self.scaling due to the WheelScrolled case
    ) {
        //
        // can go between 1 and self.max_beats
        let beat_bounds = (1.0, self.max_beats as f32);

        let lower_beat_bound = -bounds.width / 2.0 / scaling.x - BEAT_SIZE * beat_bounds.0;

        let higher_beat_bound = bounds.width / 2.0 / scaling.x - BEAT_SIZE * beat_bounds.1;

        if new_translation.x > lower_beat_bound {
            new_translation.x = lower_beat_bound;
        }

        if new_translation.x < higher_beat_bound {
            new_translation.x = higher_beat_bound;
        }

        //
        // can go between C-2 and C8 + 8 notes

        // Because MIDI is represented with 7 bits (0-127),
        // there are 5 missing notes in the 11th octabe
        //
        let lower_pitch_bound = -bounds.height / 2.0 / scaling.y - NOTE_SIZE * 0.0;

        let higher_pitch_bound = bounds.height / 2.0 / scaling.y
            - NOTE_SIZE * (self.scale.midi_range.len() as f32 - 1.0);

        if new_translation.y < higher_pitch_bound {
            new_translation.y = higher_pitch_bound;
        }

        if new_translation.y > lower_pitch_bound {
            new_translation.y = lower_pitch_bound;
        }
    }

    //     pub fn snap_to_beat(&self, x: f32) -> f32 {
    //         let scaled_beat_size = BEAT_SIZE as f32 / self.scaling.x;
    //         (x / scaled_beat_size).round() * scaled_beat_size
    //     }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cell {
    note_name: String,
    i: isize,
    j: isize,
}

impl Cell {
    pub fn at(position: Point) -> Cell {
        let i = (position.y / NOTE_SIZE as f32).ceil() as isize;
        // convert to note
        let note_name = NOTE_LABELS[((i - 1) as usize) % 12].to_string();

        let j = (position.x / BEAT_SIZE as f32).ceil() as isize;

        Cell { note_name, i: i.saturating_sub(1), j: j.saturating_sub(1) }
    }

    pub fn to_editor_axes(position: Point) -> Point {
        let i = position.y / NOTE_SIZE as f32;
        let j = position.x / BEAT_SIZE as f32;

        Point::new(j, i)
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Region {
    pub fn rows(&self) -> RangeInclusive<isize> {
        let first_row = (self.y / NOTE_SIZE as f32).floor() as isize;
        let visible_rows = (self.height / NOTE_SIZE as f32).ceil() as isize;
        first_row..=first_row + visible_rows
    }

    pub fn columns(&self) -> RangeInclusive<isize> {
        let first_column = (self.x / BEAT_SIZE as f32).floor() as isize;
        let visible_columns = (self.width / BEAT_SIZE as f32).ceil() as isize;
        first_column..=first_column + visible_columns
    }

    // fn cull<'a>(
    //     &self,
    //     cells: impl Iterator<Item = &'a Cell>,
    // ) -> impl Iterator<Item = &'a Cell> {
    //     let rows = self.rows();
    //     let columns = self.columns();

    //     cells.filter(move |cell| {
    //         rows.contains(&cell.i) && columns.contains(&cell.j)
    //     })
    // }
}

pub enum GridInteraction {
    None,
    Panning { translation: Vector, start: Point },
}

impl Default for GridInteraction {
    fn default() -> Self {
        Self::None
    }
}
