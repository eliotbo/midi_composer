//! midi notes

use crate::grid::Grid;
use crate::track::TimingInfo;

use iced::widget::canvas::{Cache, Cursor, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, Vector};

use std::fmt;

use crate::config::{BEAT_SIZE, NOTE_LABELS, NOTE_SIZE, RESIZE_BOX_PIXEL_WIDTH};
use crate::scale::Scale;

#[derive(Clone, Default)]
pub struct MidiNotes {
    // organized by pitch and then by time
    pub notes: Vec<Vec<MidiNote>>,
}

fn is_sorted<I>(data: I) -> bool
where
    I: IntoIterator,
    I::Item: PartialOrd,
{
    let mut it = data.into_iter();
    match it.next() {
        None => true,
        Some(first) => it
            .scan(first, |state, next| {
                let cmp = *state <= next;
                *state = next;
                Some(cmp)
            })
            .all(|b| b),
    }
}

impl fmt::Debug for MidiNotes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut non_empty_pitch_vecs: Vec<Vec<MidiNote>> =
            self.notes.iter().filter(|v| !v.is_empty()).cloned().collect();
        non_empty_pitch_vecs.reverse();
        let mut debug_string = String::new();
        non_empty_pitch_vecs.iter().for_each(|v| {
            let starts = v.iter().map(|n| n.start).collect::<Vec<f32>>();
            //
            //
            let pitch = (v[0].pitch.get() as i16) as usize;
            let pitch_index = pitch % 12;
            let mut note_str = NOTE_LABELS[pitch_index].to_string();
            // insert octave number
            note_str.push_str(&(pitch as i16 / 12 - 2).to_string());
            debug_string.push_str(&format!(
                "pitch: {:?} => {:?} --> is sorted: {}\n",
                note_str,
                &starts,
                is_sorted(&starts)
            ));
        });

        write!(f, "MidiNotes [\n{}]", debug_string)
    }
}
// (Pitch, Time)
// pub type NoteIndex = (usize, usize);

impl MidiNotes {
    pub fn new() -> Self {
        let mut notes = Vec::with_capacity(128);
        for _ in 0..128 {
            notes.push(Vec::new());
        }
        Self { notes }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn get(&self, note_index: NoteIndex) -> MidiNote {
        self.notes[note_index.pitch_index][note_index.time_index].clone()
    }

    // pub fn get_with_scale(&self, note_index: NoteIndex, scale: Scale) -> MidiNote {
    //     self.notes[note_index.pitch_index][note_index.time_index].clone()
    // }

    // drain the notes from self into a recipient
    pub fn drain(&mut self, recipient: &mut MidiNotes) {
        // let drained: MidiNotes = MidiNotes { notes: self.notes.drain(..).collect() };
        let drained: MidiNotes = std::mem::replace(self, Self::new());
        recipient.add_midi_notes(drained);
        // _ = std::mem::replace(self, Self::new());
    }

    pub fn add_notes_vec(&mut self, notes: Vec<MidiNote>) {
        for note in notes {
            self.add(note);
        }
    }

    pub fn add_midi_notes(&mut self, midi_notes: MidiNotes) {
        for notes in midi_notes.notes {
            for note in notes {
                self.add(note);
            }
        }
    }

    pub fn sort(&mut self) {
        for notes in self.notes.iter_mut() {
            notes.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        }
    }

    pub fn keep_one(&mut self, note_index: NoteIndex) {
        *self = Self::from(vec![self.notes[note_index.pitch_index][note_index.time_index].clone()]);
    }

    // TODO: resolve the timing conflicts when two notes overlap
    //
    // Add a midi note to the MidiNotes. It guaranteese that if the MidiNotes are
    // already sorted, then the new MidiNotes will be sorted as well.
    //
    // possible improvements:
    // 1) binary search for the insertion point
    // 2) add a field to MidiNotes that keeps track of the last
    //      inserted note for each pitch.
    //
    pub fn add(&mut self, note: MidiNote) -> NoteIndex {
        // convert pitch to index
        // insert note into notes
        // sort notes by start time

        let pitch = note.pitch.get() as usize;
        // let index = pitch ;
        // TODO: insert at correct index

        let mut time_index: isize = -1;
        let mut found_index = false;
        let mut notes_to_remove = Vec::new();

        // resolve conflicts where a note ends after another note starts
        //
        for i in 0..self.notes[pitch].len() {
            let curr = self.notes[pitch][i].clone();

            // if the new note overlaps with the current note, shorten the current note
            if note.start < curr.start && note.end > curr.start {
                // self.notes[index][i].start = note.end;
                notes_to_remove.push(i);
            }
            if note.start > curr.start && note.start < curr.end {
                // self.notes[index][i].end = note.start;
                notes_to_remove.push(i);
            }

            if note.start < curr.start && !found_index {
                time_index = i as isize;
                found_index = true;
            }
        }

        notes_to_remove.reverse();

        for i in notes_to_remove {
            self.notes[pitch].remove(i);
            if (i as isize) < time_index {
                time_index -= 1;
            }
        }

        if time_index < 0 {
            time_index = self.notes[pitch].len() as isize;
        }

        self.notes[pitch].insert(time_index as usize, note);

        NoteIndex { pitch_index: pitch, time_index: time_index as usize }

        // self.notes[index].sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }

    pub fn remove(&mut self, note_index: NoteIndex) -> MidiNote {
        self.notes[note_index.pitch_index].remove(note_index.time_index)
    }

    // TODO: get the visible regions and filter the notes accordingly using overlaps and
    // then compute their exact positions using the start and end times, and
    // then render them with fill_rectangle

    pub fn draw_notes(
        &self,
        grid: &Grid,
        bounds: &Rectangle,
        cursor: &Cursor,
        notes_cache: &Cache,
        color: Color,
    ) -> Geometry {
        let notes = notes_cache.draw(bounds.size(), |frame| {
            let mut color = color;

            grid.adjust_frame(frame, &bounds.size());

            let region = grid.visible_region(frame.size());

            let maybe_projected_cursor =
            // beware: position_in(&bounds) returns a translated position relative to the bounds
                cursor.position_in(&bounds).map(|position| grid.to_track_axes(position, &frame.size()));

            let mut white_notes =
                vec![true, false, true, false, true, true, false, true, false, true, false, true];

            white_notes.reverse();



            for row in region.rows() {


                // let pitch_relative_to_grid = row as usize;
                let pitch_relative_to_grid =  grid.scale.midi_range[row as usize] as usize;

                let maybe_note_vec = self.notes.get(pitch_relative_to_grid);

                if maybe_note_vec.is_none() || maybe_note_vec.unwrap().is_empty() {
                    continue;
                }

                // println!("\n\nrow  : {}, \npitch: {}", row, pitch_relative_to_grid);

                for note in maybe_note_vec.unwrap().iter() {
                    //
                    let pos = Point::new(note.start as f32, row as f32);
                    let note_len = note.end - note.start;



                    let pos2 = Point::new(note.start as f32, pitch_relative_to_grid as f32);
                    let note_rect = Rectangle::new(pos2, Size::new(note_len as f32, 1.0));

                    // println!("projected_cursor: {:?}", maybe_projected_cursor);

                    if let Some(projected_cursor) = maybe_projected_cursor {

                        let scale_adjusted_proj_cursor = grid.adjust_to_music_scale(projected_cursor);

                        if note_rect.contains(scale_adjusted_proj_cursor) {
                            color.a = 0.5;
                        } else {
                            color.a = 1.0;
                        }
                    }

                    frame.fill_rectangle(pos, Size::new(note_len as f32, 1.0), color);

                    // frame.fill_rectangle(pos, Size::new(total_columns as f32, 1.0), note_color);

                    // if row as i32 % 12 == 11 {
                    //     let text_pos = Point::new(
                    //         region.x / BEAT_SIZE as f32 + text_size * 0.2 / grid.scaling.y / BEAT_SIZE,
                    //         row as f32 + 0.5,
                    //     );

                    //     let note_label = Text {
                    //         color: Color::WHITE,
                    //         size: text_size,
                    //         position: text_pos,
                    //         horizontal_alignment: alignment::Horizontal::Left,
                    //         vertical_alignment: alignment::Vertical::Center,
                    //         ..Text::default()
                    //     };

                    //     let note_name = NOTE_LABELS[row as usize % 12];
                    //     frame.fill_text(Text {
                    //         content: format!("{}{}", note_name, 8.0 - (row as f32 / 12.0).floor()),
                    //         ..note_label
                    //     });
                    // }
                }
            }
        });
        notes
    }

    // TODO: optimize this. Maybe get rid of it in favor of specific drag_all and resize_all functions
    pub fn modify_all_notes(&mut self, f: impl Fn(&mut MidiNote) -> ()) {
        // more efficient with for loop with a continue on empty vecs
        let mut note_with_minimum_start = self
            .notes
            .iter()
            .flatten()
            .min_by(|a, b| a.start.partial_cmp(&b.start).unwrap())
            .unwrap()
            .clone();

        // more efficient to reverse and break in a for loop
        let mut note_with_minimum_pitch = self
            .notes
            .iter()
            .flatten()
            .min_by(|a, b| a.pitch.get().partial_cmp(&b.pitch.get()).unwrap())
            .unwrap()
            .clone();

        // more efficient to break in a for loop
        let mut note_with_maximum_pitch = self
            .notes
            .iter()
            .flatten()
            .max_by(|a, b| a.pitch.get().partial_cmp(&b.pitch.get()).unwrap())
            .unwrap()
            .clone();

        let backup_min_start_note = note_with_minimum_start.clone();
        let backup_min_pitch_note = note_with_minimum_pitch.clone();
        let backup_max_pitch_note = note_with_maximum_pitch.clone();

        f(&mut note_with_minimum_start);
        f(&mut note_with_minimum_pitch);
        f(&mut note_with_maximum_pitch);

        //
        //
        // time
        let delta_len = (note_with_minimum_start.end - note_with_minimum_start.start)
            - (backup_min_start_note.end - backup_min_start_note.start);
        let delta_time = note_with_minimum_start.start - backup_min_start_note.start;

        let mut new_delta_time = 0.0;
        let mut overide_delta_time = false;

        // if the minimum start is moved below 1.0 (the start beat of the grid),
        // then block it there
        if note_with_minimum_start.start < 1.0 {
            overide_delta_time = true;
            new_delta_time = 1.0 - backup_min_start_note.start;
        }

        //
        //
        // min pitch
        let delta_pitch_min =
            note_with_minimum_pitch.pitch.0 - backup_min_pitch_note.pitch.get() as i16;

        let mut new_delta_pitch_min = 0;
        let mut overide_delta_pitch_min = false;

        // if the minimum pitch is moved below 0 (the lowest pitch of the grid),
        // then block it there
        if note_with_minimum_pitch.pitch.0 < 0 {
            overide_delta_pitch_min = true;
            new_delta_pitch_min = 0 - backup_min_pitch_note.pitch.get() as i16;
        }

        //
        //
        // max pitch
        let delta_pitch_max =
            note_with_maximum_pitch.pitch.0 - backup_max_pitch_note.pitch.get() as i16;

        let mut new_delta_pitch_max = 0;
        let mut overide_delta_pitch_max = false;

        // if the maximum pitch is moved above 127 (the highest pitch of the grid),
        // then block it there
        if note_with_maximum_pitch.pitch.0 > 127 {
            overide_delta_pitch_max = true;
            new_delta_pitch_max = 127 - backup_max_pitch_note.pitch.get() as i16;
        }

        for notes_in_pitch in self.notes.iter_mut() {
            for note in notes_in_pitch.iter_mut() {
                // apply transformation to all notes
                f(note);

                // revert transformation if it would move the minimum start below 1.0
                if overide_delta_time {
                    // for both dragging and resizing
                    note.start += new_delta_time - delta_time;

                    // only for case of dragging the whole notes to the left
                    if delta_len.abs() < 0.0000001 {
                        note.end += new_delta_time - delta_time;
                    }
                }

                // revert transformation if it would move the minimum pitch below 0
                if overide_delta_pitch_min {
                    let new_pitch = note.pitch.0 - delta_pitch_min + new_delta_pitch_min;

                    note.pitch = Pitch(new_pitch as i16);
                }

                // revert transformation if it would move the maximum pitch above 127
                if overide_delta_pitch_max {
                    let new_pitch = note.pitch.0 - delta_pitch_max + new_delta_pitch_max;

                    note.pitch = Pitch(new_pitch as i16);
                }
            }
        }
    }

    // TODO: optimize using only the notes that can be currently viewed
    pub fn get_note_under_cursor(&self, grid: &Grid, projected_cursor: Point) -> Option<OverNote> {
        let mut resize_end = NoteEdge::None;

        for (pitch_index, notes_in_pitch) in self.notes.iter().enumerate() {
            for (time_index, note) in notes_in_pitch.iter().enumerate() {
                //
                //
                let note_len = note.end - note.start;
                let pos2 = Point::new(note.start as f32, pitch_index as f32);

                // TODO: make this scale independent (fixed number of pixels away from edge)
                let resizing_rect_len = RESIZE_BOX_PIXEL_WIDTH * grid.scaling.x / BEAT_SIZE;
                let start_rect = Rectangle::new(pos2, Size::new(resizing_rect_len, 1.0));
                let end_rect = Rectangle::new(
                    pos2 + Vector::new(note_len as f32 - resizing_rect_len, 0.0),
                    Size::new(resizing_rect_len, 1.0),
                );

                if start_rect.contains(projected_cursor) {
                    resize_end = NoteEdge::Start;
                } else if end_rect.contains(projected_cursor) {
                    resize_end = NoteEdge::End;
                }

                // clicked an edge
                if resize_end != NoteEdge::None {
                    return Some(OverNote {
                        note_index: NoteIndex { pitch_index, time_index },
                        note_edge: resize_end,
                    });
                }

                // Start dragging if click is inside note
                let note_rect = Rectangle::new(pos2, Size::new(note_len as f32, 1.0));
                if note_rect.contains(projected_cursor) {
                    return Some(OverNote {
                        note_index: NoteIndex { pitch_index, time_index },
                        note_edge: resize_end,
                    });
                }
            }
        }
        None
    }

    pub fn get_notes_in_rect(&self, rect: Rectangle) -> Vec<NoteIndex> {
        // let mut notes_in_rect = Vec::new();

        // filter out the notes using the pitch range of the Rectangle
        let b0 = rect.y + rect.height;
        let b1 = rect.y;
        let potential_pitch_vec = b0.min(b1).floor() as usize..b0.max(b1).ceil() as usize;

        let notes_filtered_by_pitch: Vec<Vec<MidiNote>> = self.notes[potential_pitch_vec].to_vec();

        let a0 = rect.x;
        let a1 = rect.x + rect.width;
        let time_bounds = (a0.min(a1), a0.max(a1));

        let note_indices: Vec<NoteIndex> = notes_filtered_by_pitch
            .iter()
            .enumerate()
            .flat_map(|(_wrong_pitch_index, notes_in_pitch)| {
                // let mut rev_notes_in_pitch = notes_in_pitch.clone();
                //
                // the reverse makes it so that when the notes are drained out, they are being drawn
                // from higher indices to lower indices, which won't compromise the indices of the
                // notes that are being drawn later.
                // rev_notes_in_pitch.reverse();
                notes_in_pitch
                    .iter()
                    .enumerate()
                    .filter_map(move |(time_index, note)| {
                        if note.start < time_bounds.1 && note.end > time_bounds.0 {
                            Some(NoteIndex { pitch_index: note.pitch.get() as usize, time_index })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<NoteIndex>>()
            })
            .collect();

        // println!();
        // println!("rect: {:?}", rect);
        // println!("potential_pitch_vec: {:?}", potential_pitch_vec);
        // println!("notes_filtered_by_pitch: {:?}", notes_filtered_by_pitch);

        note_indices
    }

    // remove a set of notes using a Vec<(pitch_index, time_index)>,
    // beware that starting with the lowest index will not work because
    // the indices will change as the notes are removed
    pub fn remove_notes(&mut self, mut note_indices: Vec<NoteIndex>) -> MidiNotes {
        note_indices.sort_by(|a, b| b.time_index.cmp(&a.time_index));
        // note_indices.reverse();
        let mut removed_notes: MidiNotes = MidiNotes::new();
        for NoteIndex { pitch_index, time_index } in note_indices {
            // println!("removing note at pitch_index: {}, time_index: {}", pitch_index, time_index);

            let note = self.notes[pitch_index].remove(time_index);
            removed_notes.add(note);
        }
        removed_notes
    }
}

impl From<Vec<MidiNote>> for MidiNotes {
    fn from(notes: Vec<MidiNote>) -> Self {
        let mut midi_notes = Self::new();
        for note in notes {
            midi_notes.add(note);
        }
        midi_notes
    }
}

#[derive(Clone)]
pub struct MidiNote {
    // pub id: u32,
    pub start: f32,
    pub end: f32,
    pub pitch: Pitch,
    _velocity: u16,
    _automation: Automation,
    // dynamic: Dynamic,
    // expression: Expression,
    // vibrato: Vibrato,
}

impl fmt::Debug for MidiNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "start {}", self.start)
    }
}

// #[derive(Debug, Clone)]
// struct Dynamic;
// #[derive(Debug, Clone)]
// struct Expression;
// #[derive(Debug, Clone)]
// struct Vibrato;

impl MidiNote {
    pub fn new(start: f32, end: f32, pitch: Pitch) -> Self {
        Self { start, end, pitch, _velocity: u16::MAX, _automation: Automation::default() }
    }

    pub fn shorten(&mut self, amount: f32) {
        self.end -= amount;
    }

    pub fn shorten_from_start(&mut self, amount: f32) {
        self.start += amount;
    }

    pub fn shorten_with_percent(&mut self, percentage: f32) {
        let amount = (self.end - self.start) * percentage;
        self.end -= amount;
    }

    pub fn to_seconds(&self, timing: TimingInfo) -> (f32, f32) {
        // convert bpm to beats per second
        let bps = timing.bpm / 60.0;

        // convert start and end to beats
        (self.start / bps, self.end / bps)
    }

    pub fn overlaps_with(&self, rect: &Rectangle) -> bool {
        let (start, end) = (self.start as f32, self.end as f32);
        let (x0, y0) = (rect.x, rect.y);
        let (x1, y1) = (rect.x + rect.width, rect.y + rect.height);

        let (sx, sy) = (x0.min(x1), y0.min(y1));
        let (ex, ey) = (x0.max(x1), y0.max(y1));

        let pitch = self.pitch.0 as f32;

        if start >= sx && start <= ex && pitch >= sy && pitch <= ey {
            return true;
        }

        if end >= sx && end <= ex && pitch >= sy && pitch <= ey {
            return true;
        }

        if start <= sx && end >= ex && pitch >= sy && pitch <= ey {
            return true;
        }

        false
    }

    pub fn to_label(&self) -> String {
        format!("pitch label: {}", self.pitch.to_str())
    }
}

#[derive(Clone, Default)]
pub struct Selected {
    pub notes: MidiNotes,
    pub selecting_square: Option<Rectangle>,
    // the direct_selecting_square is not adjusted to the music scale,
    // thus making it useful only for drawing.
    pub direct_selecting_square: Option<Rectangle>,
}

impl Selected {
    pub fn draw_selecting_square(
        &self,
        bounds: Rectangle,
        grid: &Grid,
        selection_square_cache: &Cache,
    ) -> Geometry {
        selection_square_cache.draw(bounds.size(), |frame| {
            if let Some(select) = self.direct_selecting_square {
                grid.adjust_frame(frame, &bounds.size());

                frame.stroke(
                    &Path::rectangle(Point::new(select.x, select.y), select.size()),
                    Stroke::default().with_width(2.0),
                );
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OverNote {
    pub note_index: NoteIndex,
    pub note_edge: NoteEdge,
}

#[derive(Clone, Copy, Debug)]
pub struct NoteIndex {
    pub pitch_index: usize,
    pub time_index: usize,
}

// An enum describing the transition between the main MidiNotes and the Selected MidiNotes
#[derive(Clone, Debug)]
pub enum ChangeSelection {
    // Move all notes from Selected to the main MidiNotes
    DrainSelect,
    // Move all notes from the main MidiNotes to the Selected MidiNotes
    SelectAll,
    // Move one note from the main MidiNotes to the Selected MidiNotes
    AddOneToSelected { note_index: NoteIndex },
    // Move one note from the Selected MidiNotes to the main MidiNotes
    UnselectOne { note_index: NoteIndex },
    // Keep only one note in the Selected MidiNotes
    UnselectAllButOne { note_index: NoteIndex },
    // Drain the Selected notes and add one note from the main notes to the Selected note
    SelectOne { note_index: NoteIndex },
    // Add multiple notes from the main notes to the Selected notes
    SelectMany { note_indices: Vec<NoteIndex> },
}

#[derive(Debug, Clone, Copy)]
pub struct Pitch(pub i16);

impl Pitch {
    pub fn new(pitch: u8) -> Self {
        Self(pitch as i16)
    }

    pub fn get(&self) -> u8 {
        self.0 as u8
    }

    pub fn set(&mut self, pitch: u8) {
        self.0 = pitch as i16;
    }

    // pub fn from_midi(midi: u8) -> Self {
    //     unimplemented!("TODO: See MIDI specs");
    // }

    pub fn to_str(&self) -> String {
        let octave = self.0 / 12 - 2;
        let note = self.0 % 12;

        let note_name = match note {
            0 => "C",
            1 => "C#",
            2 => "D",
            3 => "D#",
            4 => "E",
            5 => "F",
            6 => "F#",
            7 => "G",
            8 => "G#",
            9 => "A",
            10 => "A#",
            11 => "B",
            _ => "C",
        };

        format!("{}{}", note_name, octave)
    }

    // parses the note name (ex: "C#6")
    pub fn from_str(s: &str) -> Self {
        let note_name =
            s.chars().take_while(|c| c.is_alphabetic() || *c == '#').collect::<String>();

        let octave_number =
            s.chars().skip_while(|c| c.is_alphabetic() || *c == '#').collect::<String>();

        // convert the note name to a midi number
        let midi_number = match note_name.as_str() {
            "C" => 0,
            "C#" => 1,
            "D" => 2,
            "D#" => 3,
            "E" => 4,
            "F" => 5,
            "F#" => 6,
            "G" => 7,
            "G#" => 8,
            "A" => 9,
            "A#" => 10,
            "B" => 11,
            _ => panic!("Invalid note name"),
        };

        // add the octave number to the midi number
        let midi_number = midi_number + (octave_number.parse::<i8>().unwrap() + 2) as u8 * 12;

        Pitch(midi_number as i16)
    }
}

#[derive(Clone)]
pub enum NoteInteraction {
    None,
    Dragging { initial_cursor_pos: Point, original_notes: MidiNotes },
    Delete { notes_to_delete: Vec<(usize, usize)> },

    Resizing { initial_cursor_pos: Point, original_notes: MidiNotes, resize_end: NoteEdge },
    ResizingHover,
    Selecting { initial_music_cursor: Point, initial_cursor_proj: Point },
}

impl Default for NoteInteraction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Default, Clone)]
pub struct Automation;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NoteEdge {
    Start,
    End,
    None,
}
