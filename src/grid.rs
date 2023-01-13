// use iced::widget::canvas::event::Event;
use iced::widget::canvas::{Cache, Cursor, Frame, Geometry, Text};
use iced::{alignment, Color, Point, Rectangle, Size, Vector};
// use rustc_hash::{FxHashMap, FxHashSet};

use crate::midi_notes::NOTE_LABELS;
use crate::track::{BEAT_SIZE, NOTE_SIZE};
use crate::{INIT_GRID_SIZE, INIT_SCALING};
use std::ops::RangeInclusive;

// use crate::piano_theme::PianoTheme;
// use crate::track::TrackMessage;

#[derive(Clone)]
pub struct Grid {
    pub translation: Vector,
    pub scaling: Vector,
    pub max_beats: usize,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            translation: Vector::new(
                -INIT_GRID_SIZE.width / 2.0 - BEAT_SIZE,
                -INIT_GRID_SIZE.height / 2.0 - NOTE_SIZE * 67.0,
            ),
            scaling: INIT_SCALING,
            max_beats: 100,
        }
    }
}

impl Grid {
    pub fn to_track_axes(&self, point: Point, size: &Size) -> Point {
        let region = self.visible_region(*size);

        let projection =
            Point::new(point.x / self.scaling.x + region.x, point.y / self.scaling.y + region.y);

        let x = projection.x / BEAT_SIZE as f32;
        let y = 132.0 - projection.y / NOTE_SIZE as f32;

        Point::new(x, y)
    }
    pub fn draw_background(
        &self,
        bounds: Rectangle,
        // cursor: Cursor,
        grid_cache: &Cache,
    ) -> Geometry {
        let center = Vector::new(bounds.width / 2.0, bounds.height / 2.0);

        let grid = grid_cache.draw(bounds.size(), |frame| {
            frame.translate(center);
            frame.scale(self.scaling);
            frame.translate(self.translation);
            frame.scale(Vector::new(BEAT_SIZE, NOTE_SIZE));

            let region = self.visible_region(frame.size());

            // TODO: line width should be scaled with the zoom level?
            let rows = region.rows();

            let columns = region.columns();
            let (total_rows, total_columns) = (rows.clone().count(), columns.clone().count());
            let beat_linewidth = 2.0 / BEAT_SIZE;
            let note_linewidth = 2.0 / NOTE_SIZE;
            let color = Color::from_rgb8(70, 74, 83);

            let text_size = 14.0;

            let mut note_colors =
                vec![true, false, true, false, true, true, false, true, false, true, false, true];

            note_colors.reverse();

            let alpha = 0.25;

            for row in region.rows() {
                let pos = Point::new(*columns.start() as f32, row as f32);
                frame.fill_rectangle(pos, Size::new(total_columns as f32, note_linewidth), color);

                let note_color = if note_colors[row as usize % 12] {
                    Color::from_rgba8(100, 100, 100, alpha)
                } else {
                    Color::from_rgba8(10, 10, 10, alpha)
                };

                frame.fill_rectangle(pos, Size::new(total_columns as f32, 1.0), note_color);

                if row as i32 % 12 == 11 {
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

                    let note_name = NOTE_LABELS[11 - row as usize % 12];
                    frame.fill_text(Text {
                        content: format!("{}{}", note_name, 8.0 - (row as f32 / 12.0).floor()),
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

            // // show hovered cell with a transparent overlay
            // if let Some(cell) = hovered_cell.clone() {
            //     frame.with_save(|frame| {
            //         frame.translate(center);
            //         frame.scale(self.scaling);
            //         frame.translate(self.translation);
            //         frame.scale(Vector::new(BEAT_SIZE, NOTE_SIZE));

            //         frame.fill_rectangle(
            //             Point::new(cell.j as f32, cell.i as f32),
            //             Size::UNIT,
            //             Color {
            //                 a: 0.5,
            //                 ..Color::BLACK
            //             },
            //         );
            //     });
            // }

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
        // let beat_size = BEAT_SIZE / self.scaling.x;
        // let note_size = NOTE_SIZE / self.scaling.y;

        Region {
            x: -self.translation.x - width / 2.0,
            y: -self.translation.y - height / 2.0,
            width,
            height,
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
        let lower_pitch_bound = -bounds.height / 2.0 / scaling.y - NOTE_SIZE * 5.0;

        let higher_pitch_bound = bounds.height / 2.0 / scaling.y - NOTE_SIZE * 12.0 * 11.0; // 11 octaves

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
        let i = 132.0 - position.y / NOTE_SIZE as f32;
        let j = position.x / BEAT_SIZE as f32;

        Point::new(j, i)
    }
}

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
