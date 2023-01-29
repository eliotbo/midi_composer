//! midi notes

use crate::grid::Grid;
use crate::track::TimingInfo;

use iced::widget::canvas::event::{self};
use iced::widget::canvas::{Cache, Cursor, Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Size, Vector};

use std::fmt;

use super::scale::Scale;
use crate::config::{BEAT_SIZE, NOTE_LABELS, RESIZE_BOX_PIXEL_WIDTH};
use crate::track::TrackMessage;

use crate::track::actions::{AddedNote, ConflictHistory, DeletedNote, ResizedConflicts};

#[derive(Clone)]
pub struct MidiNotes {
    // organized by pitch and then by time
    pub notes: Vec<Vec<MidiNote>>,
    pub number_of_notes: usize,
    pub start_time: f32, // TODO: keep track of start and end time when adding/deleting notes
    pub end_time: f32,
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

impl Default for MidiNotes {
    fn default() -> Self {
        Self::new()
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

// type Resized = Vec<(usize, NoteEdge, f32)>;
// type Deleted = Vec<(usize, MidiNote)>;

impl MidiNotes {
    pub fn new() -> Self {
        let mut notes = Vec::with_capacity(128);
        for _ in 0..128 {
            notes.push(Vec::new());
        }
        Self { notes, start_time: 1.0, end_time: 5.0, number_of_notes: 0 }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    // TODO: return Option
    pub fn get(&self, note_index: NoteIndex) -> &MidiNote {
        &self.notes[note_index.pitch_index][note_index.time_index]
    }

    pub fn get_mut(&mut self, note_index: NoteIndex) -> &mut MidiNote {
        &mut self.notes[note_index.pitch_index][note_index.time_index]
    }

    // pub fn get_with_scale(&self, note_index: NoteIndex, scale: Scale) -> MidiNote {
    //     self.notes[note_index.pitch_index][note_index.time_index].clone()
    // }

    // drain the notes from self into a recipient
    pub fn drain(&mut self, recipient: &mut MidiNotes) -> Vec<AddedNote> {
        // let drained: MidiNotes = MidiNotes { notes: self.notes.drain(..).collect() };
        // let all_note_indices = self.get_all_note_indices();
        let drained: MidiNotes = std::mem::replace(self, Self::new());
        self.number_of_notes = 0;

        recipient.add_midi_notes(&drained)
    }

    pub fn delete_all(&mut self) -> Self {
        let drained: MidiNotes = std::mem::replace(self, Self::new());
        self.number_of_notes = 0;

        drained
    }

    // pub fn delete(&mut self, note_index: NoteIndex) {
    //     self.notes[note_index.pitch_index].remove(note_index.time_index);
    //     self.number_of_notes -= 1;
    // }

    pub fn get_all_note_indices(&self) -> Vec<NoteIndex> {
        let mut note_indices = Vec::new();
        for (pitch_index, notes) in self.notes.iter().enumerate() {
            for (time_index, _) in notes.iter().enumerate() {
                note_indices.push(NoteIndex { pitch_index, time_index });
            }
        }
        note_indices
    }

    pub fn add_notes_vec(&mut self, notes: Vec<MidiNote>) {
        for note in notes {
            self.add(&note);
        }
    }

    pub fn add_midi_notes(&mut self, midi_notes: &MidiNotes) -> Vec<AddedNote> {
        let mut added_notes = Vec::new();
        // let mut conflict_history = ConflictHistory::default();
        for notes in midi_notes.notes.iter() {
            for note in notes {
                // let added_note_and_conflicts = ;
                added_notes.push(self.add(&note));
                // conflict_history.add(conflicts);
            }
        }
        added_notes
    }

    pub fn sort(&mut self) {
        for notes in self.notes.iter_mut() {
            notes.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        }
    }

    pub fn keep_one(&mut self, note_index: NoteIndex) {
        *self = Self::from(vec![self.notes[note_index.pitch_index][note_index.time_index].clone()]);
        self.number_of_notes = 1;
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
    //
    // // returns (the index of the added note, the notes that were resized, the notes that were deleted )
    // pub fn add(&mut self, note: &MidiNote) -> AddedNote {
    //     // convert pitch to index
    //     // insert note into notes
    //     // sort notes by start time

    //     let pitch = note.pitch.get() as usize;
    //     // let index = pitch ;
    //     // TODO: insert at correct index

    //     let mut time_index: isize = -1;
    //     // let mut found_index = false;
    //     let mut notes_to_remove = Vec::new();
    //     let mut removed_notes: Vec<DeletedNote> = Vec::new();
    //     let mut resized_notes: Vec<ResizedConflicts> = Vec::new();
    //     self.number_of_notes += 1;

    //     for i in 0..self.notes[pitch].len() {
    //         let curr = self.notes[pitch][i].clone();

    //         // if the start time of the new note is before the start time of the current note,
    //         // insert the new note here.
    //         if note.start < curr.start {
    //             time_index = i as isize;
    //         }

    //         // if the end of the new note is before the start of the current note, insert the new note
    //         // at the current index. No side effects needed.
    //         if note.end <= curr.start {
    //             break;
    //         }

    //         // if the new note partially overlaps with the current note start, shorten the
    //         // current note using its start point.
    //         if note.start < curr.start && note.end < curr.end {
    //             let delta_time = curr.start - note.end;
    //             resized_notes.push(ResizedConflicts {
    //                 note_index: NoteIndex { pitch_index: pitch, time_index: i },
    //                 edge: NoteEdge::Start,
    //                 delta_time,
    //             });
    //             self.notes[pitch][i].start = note.end;
    //             break;
    //         }

    //         // if the new note completely overlaps with the current note, delete the current note,
    //         // and continue
    //         if note.start <= curr.start && note.end >= curr.end {
    //             notes_to_remove.push(i);
    //             continue;
    //         }

    //         // if the new note fits within the current note, shorten the end of the current note
    //         if note.start > curr.start && note.start < curr.end {
    //             self.notes[pitch][i].end = note.start;
    //             break;
    //         }

    //         // // if the new note completely overlaps the current note, remove the current note
    //         // if (note.start >= curr.start && note.end <= curr.end)
    //         //     || (note.start <= curr.start && note.end >= curr.end)
    //         // {
    //         //     notes_to_remove.push(i);
    //         // }
    //     }

    //     // remove notes that completely overlap with the new note
    //     notes_to_remove.reverse();
    //     for i in notes_to_remove {
    //         self.number_of_notes -= 1;
    //         let removed_note = self.notes[pitch].remove(i);
    //         removed_notes.push(DeletedNote {
    //             note_index: NoteIndex { pitch_index: pitch, time_index: i },
    //             removed_note,
    //         });
    //         if (i as isize) < time_index {
    //             time_index -= 1;
    //         }
    //     }

    //     // insert note
    //     if time_index < 0 {
    //         time_index = self.notes[pitch].len() as isize;
    //     }

    //     self.notes[pitch].insert(time_index as usize, note.clone());

    //     // let mut notes_to_remove = self.resolve_conflicts(note);

    //     AddedNote {
    //         note_index_after: NoteIndex { pitch_index: pitch, time_index: time_index as usize },
    //         note_to_add: note.clone(),
    //         resized_notes,
    //         removed_notes,
    //     }

    //     // self.notes[index].sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    // }

    pub fn add(&mut self, note: &MidiNote) -> AddedNote {
        let conflicts = self.resolve_conflicts_single(&note);
        let time_index = self.find_time_index(note);
        let pitch = note.pitch.get() as usize;
        self.number_of_notes += 1;

        let added_note = AddedNote {
            note_index_after: NoteIndex { pitch_index: pitch, time_index: time_index as usize },
            note_to_add: note.clone(),
            conflicts,
            // resized_notes: conflicts.resized,
            // removed_notes: conflicts.deleted,
        };

        self.notes[pitch].insert(time_index as usize, note.clone());
        added_note
        // (added_note, conflicts)
    }
    // After resize a set of notes, the notes may overlap with each other. This function
    // resolves the conflicts by resizing the notes. The note that starts later has
    // priority over the note that starts earlier, which means that the end of a note
    // is modified if there is a conflict with the next note.
    pub fn resolve_self_resize_conflicts(&mut self) -> Vec<ResizedConflicts> {
        let mut all_conflicts = Vec::new();

        for pitch_vec in self.notes.iter_mut() {
            let pitch_vec_clone = pitch_vec.clone();
            let pitch_vec_len = pitch_vec.len();
            // let mut double_break = false;

            // for each note, check if it overlaps with any other following note
            //
            for (i, note) in pitch_vec.iter_mut().enumerate() {
                let pitch_index = note.pitch.get() as usize;
                if i < pitch_vec_len - 1 {
                    let next_note = &pitch_vec_clone[i + 1];

                    // if the current note ends before the next note starts, no conflict
                    if next_note.start >= note.end {
                        break;
                    } else {
                        // add local conflict
                        let local_conflict = ResizedConflicts {
                            note_index: NoteIndex { pitch_index: pitch_index, time_index: i },
                            edge: NoteEdge::End,
                            delta_time: next_note.start - note.end, // delta time should be negative
                        };
                        note.end = next_note.start;
                        all_conflicts.push(local_conflict);
                    }
                }
            }
        }
        all_conflicts
    }

    // resolves overlapping notes between midi_notes and selected.notes.
    // The selected notes have priority since they are the ones that are being edited.
    pub fn resolve_conflicts(&mut self, notes: &MidiNotes) -> ConflictHistory {
        let mut all_conflicts = ConflictHistory::default();
        for pitch_vec in notes.notes.iter() {
            for note in pitch_vec.iter() {
                let local_conflict = self.resolve_conflicts_single(note);
                all_conflicts.add(local_conflict);
            }
        }
        all_conflicts
    }

    // find the time_index of an external note using binary search
    //
    // Copilot, can you tell me if the following code is correct?
    pub fn find_time_index(&self, note: &MidiNote) -> usize {
        let pitch = note.pitch.get() as usize;

        let mut time_index = -1;
        let mut min = 0;
        let mut max = self.notes[pitch].len();

        while min < max {
            let mid = (min + max) / 2;
            if note.start < self.notes[pitch][mid].start {
                max = mid;
            } else if note.start > self.notes[pitch][mid].start {
                min = mid + 1;
            } else {
                time_index = mid as isize;
                break;
            }
        }

        if time_index < 0 {
            time_index = min as isize;
        }

        time_index as usize
    }

    pub fn resolve_conflicts_single(&mut self, note: &MidiNote) -> ConflictHistory {
        let pitch = note.pitch.get() as usize;
        let mut notes_to_remove = Vec::new();
        let mut removed_notes: Vec<DeletedNote> = Vec::new();
        let mut resized_notes: Vec<ResizedConflicts> = Vec::new();

        let time_index = self.find_time_index(note);

        // if the note overlaps with a note that starts before it, resize the note before it
        if time_index > 0 {
            let curr = &mut self.notes[pitch][time_index - 1];
            if note.start < curr.end {
                let delta_time = note.start - curr.end;
                resized_notes.push(ResizedConflicts {
                    note_index: NoteIndex {
                        pitch_index: pitch,
                        time_index: (time_index - 1) as usize,
                    },
                    edge: NoteEdge::End,
                    delta_time,
                });
                curr.end = note.start;
            }
        }

        // if the note overlaps with a note after it, apply a resize or a delete depending
        // on whether it overlaps partly or completely, respectively

        for i in time_index..self.notes[pitch].len() {
            let curr = &mut self.notes[pitch][i];
            // if no overlap, then break
            if note.end < curr.start {
                break;
            }

            // overlaps partly
            if note.end > curr.start && note.end < curr.end {
                let delta_time = note.end - curr.start;
                resized_notes.push(ResizedConflicts {
                    note_index: NoteIndex { pitch_index: pitch, time_index: i },
                    edge: NoteEdge::Start,
                    delta_time,
                });
                curr.start = note.end;
                break;
            }

            // check for overlaps completely
            if note.end > curr.start && note.end >= curr.end {
                notes_to_remove.push(i);
            }
        }

        // remove notes that completely overlap with the new note
        notes_to_remove.reverse();
        for i in notes_to_remove {
            self.number_of_notes -= 1;
            let removed_note = self.notes[pitch].remove(i);
            removed_notes.push(DeletedNote {
                note_index: NoteIndex { pitch_index: pitch, time_index: i },
                removed_note,
            });
        }

        ConflictHistory { deleted: removed_notes, resized: resized_notes }

        // remove notes that completely overlap with the new note
    }

    pub fn remove(&mut self, note_index: &NoteIndex) -> MidiNote {
        self.notes[note_index.pitch_index].remove(note_index.time_index)
    }

    // pub fn remove_all(&mut self) -> Vec<MidiNote> {
    //     let mut removed_notes = Vec::new();
    //     for pitch in self.notes.iter_mut() {
    //         for note in pitch.iter() {
    //             removed_notes.push(note.clone());
    //         }
    //     }
    //     notes
    // }

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


                for note in maybe_note_vec.unwrap().iter() {
                    //
                    let pos = Point::new(note.start as f32, row as f32);
                    let note_len = note.end - note.start;



                    let pos2 = Point::new(note.start as f32, pitch_relative_to_grid as f32);
                    let note_rect = Rectangle::new(pos2, Size::new(note_len as f32, 1.0));


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

    // // TODO: optimize this. Maybe get rid of it in favor of specific drag_all and resize_all functions
    // pub fn modify_all_notes(&mut self, f: impl Fn(&mut MidiNote) -> ()) {
    //     // more efficient with for loop with a continue on empty vecs
    //     let mut note_with_minimum_start = self
    //         .notes
    //         .iter()
    //         .flatten()
    //         .min_by(|a, b| a.start.partial_cmp(&b.start).unwrap())
    //         .unwrap()
    //         .clone();

    //     // more efficient to reverse and break in a for loop
    //     let mut note_with_minimum_pitch = self
    //         .notes
    //         .iter()
    //         .flatten()
    //         .min_by(|a, b| a.pitch.get().partial_cmp(&b.pitch.get()).unwrap())
    //         .unwrap()
    //         .clone();

    //     // more efficient to break in a for loop
    //     let mut note_with_maximum_pitch = self
    //         .notes
    //         .iter()
    //         .flatten()
    //         .max_by(|a, b| a.pitch.get().partial_cmp(&b.pitch.get()).unwrap())
    //         .unwrap()
    //         .clone();

    //     let backup_min_start_note = note_with_minimum_start.clone();
    //     let backup_min_pitch_note = note_with_minimum_pitch.clone();
    //     let backup_max_pitch_note = note_with_maximum_pitch.clone();

    //     f(&mut note_with_minimum_start);
    //     f(&mut note_with_minimum_pitch);
    //     f(&mut note_with_maximum_pitch);

    //     //
    //     //
    //     // time
    //     let delta_len = (note_with_minimum_start.end - note_with_minimum_start.start)
    //         - (backup_min_start_note.end - backup_min_start_note.start);
    //     let delta_time = note_with_minimum_start.start - backup_min_start_note.start;

    //     let mut new_delta_time = 0.0;
    //     let mut overide_delta_time = false;

    //     // if the minimum start is moved below 1.0 (the start beat of the grid),
    //     // then block it there
    //     if note_with_minimum_start.start < 1.0 {
    //         overide_delta_time = true;
    //         new_delta_time = 1.0 - backup_min_start_note.start;
    //     }

    //     //
    //     //
    //     // min pitch
    //     let delta_pitch_min =
    //         note_with_minimum_pitch.pitch.0 - backup_min_pitch_note.pitch.get() as i16;

    //     let mut new_delta_pitch_min = 0;
    //     let mut overide_delta_pitch_min = false;

    //     // if the minimum pitch is moved below 0 (the lowest pitch of the grid),
    //     // then block it there
    //     if note_with_minimum_pitch.pitch.0 < 0 {
    //         overide_delta_pitch_min = true;
    //         new_delta_pitch_min = 0 - backup_min_pitch_note.pitch.get() as i16;
    //     }

    //     //
    //     //
    //     // max pitch
    //     let delta_pitch_max =
    //         note_with_maximum_pitch.pitch.0 - backup_max_pitch_note.pitch.get() as i16;

    //     let mut new_delta_pitch_max = 0;
    //     let mut overide_delta_pitch_max = false;

    //     // if the maximum pitch is moved above 127 (the highest pitch of the grid),
    //     // then block it there
    //     if note_with_maximum_pitch.pitch.0 > 127 {
    //         overide_delta_pitch_max = true;
    //         new_delta_pitch_max = 127 - backup_max_pitch_note.pitch.get() as i16;
    //     }

    //     for notes_in_pitch in self.notes.iter_mut() {
    //         for note in notes_in_pitch.iter_mut() {
    //             // apply transformation to all notes
    //             f(note);

    //             // revert transformation if it would move the minimum start below 1.0
    //             if overide_delta_time {
    //                 // for both dragging and resizing
    //                 note.start += new_delta_time - delta_time;

    //                 // only for case of dragging the whole notes to the left
    //                 if delta_len.abs() < 0.0000001 {
    //                     note.end += new_delta_time - delta_time;
    //                 }
    //             }

    //             // revert transformation if it would move the minimum pitch below 0
    //             if overide_delta_pitch_min {
    //                 let new_pitch = note.pitch.0 - delta_pitch_min + new_delta_pitch_min;

    //                 note.pitch = Pitch(new_pitch as i16);
    //             }

    //             // revert transformation if it would move the maximum pitch above 127
    //             if overide_delta_pitch_max {
    //                 let new_pitch = note.pitch.0 - delta_pitch_max + new_delta_pitch_max;

    //                 note.pitch = Pitch(new_pitch as i16);
    //             }
    //         }
    //     }
    // }

    // returns the delta time and delta pitch
    pub fn drag_all_notes(&mut self, delta_cursor: Vector, grid: &Grid) -> (f32, i8) {
        // Maybe we should keep track of the minimum start time somewhere to avoid this big O(n) search
        let note_with_minimum_start = self
            .notes
            .iter()
            .flatten()
            .min_by(|a, b| a.start.partial_cmp(&b.start).unwrap())
            .unwrap()
            .clone();

        // find the minimum pitch by finding the first non-empty vector
        let mut note_with_minimum_pitch =
            self.notes.iter().find(|v| !v.is_empty()).unwrap().first().unwrap().clone();

        // find the minimum pitch by finding the first non-empty vector from the top
        let mut note_with_maximum_pitch =
            self.notes.iter().rev().find(|v| !v.is_empty()).unwrap().first().unwrap().clone();

        let new_start = note_with_minimum_start.get_new_start(delta_cursor);

        // get the pitch index relative to the current music scale
        let min_scaled_pitch = note_with_minimum_pitch.get_scaled_pitch(&grid.scale);
        let max_scaled_pitch = note_with_maximum_pitch.get_scaled_pitch(&grid.scale);

        // the grid is scaled such that a unit in the x direction is equal to a beat
        // and a unit in the y direction is equal to a pitch
        let mut delta_pitch = delta_cursor.y as i8;
        let mut delta_time = delta_cursor.x;

        // time
        //
        // if the minimum start is moved below 1.0 (the start beat of the grid),
        // then block it there
        if new_start < 1.0 {
            delta_time = 1.0 - note_with_minimum_start.start;
        }

        // min pitch
        //
        // if the minimum pitch is moved below 0 (the lowest pitch of the grid),
        // then block it there
        //
        let new_min_pitch = min_scaled_pitch as i16 + delta_cursor.y.floor() as i16;
        if new_min_pitch < 0 {
            delta_pitch = 0 - min_scaled_pitch;
        }

        //
        //
        // // max pitch
        // if the maximum pitch is moved above grid.scale.midi_size() (the highest pitch of the grid),
        // then block it there.
        // Note sure why we need to subtract 2 here, but it works.
        let new_max_pitch = max_scaled_pitch as i16 + delta_pitch as i16;
        if new_max_pitch > grid.scale.midi_size() as i16 - 2 {
            delta_pitch = (grid.scale.midi_size() as i16 - (max_scaled_pitch as i16)) as i8 - 2;
        }

        // let mut absolute_delta_pitch = 0;
        for notes_in_pitch in self.notes.iter_mut() {
            for note in notes_in_pitch.iter_mut() {
                //
                // apply transformation to all notes

                note.reposition(delta_pitch, delta_time, &grid.scale);
            }
        }

        return (delta_time, delta_pitch);
    }

    // TODO: optimize this.
    pub fn resize_all_notes(&mut self, resize_end: NoteEdge, delta_time: f32) -> f32 {
        //
        for notes_in_pitch in self.notes.iter_mut() {
            for note in notes_in_pitch.iter_mut() {
                //
                // apply transformation to all notes
                note.resize(resize_end, delta_time);
            }
        }
        delta_time
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

        note_indices
    }

    // remove a set of notes using a Vec<(pitch_index, time_index)>,
    // beware that starting with the lowest index will not work because
    // the indices will change as the notes are removed
    pub fn remove_notes(&mut self, note_indices: &Vec<NoteIndex>) -> MidiNotes {
        let note_indices = &mut note_indices.clone();
        note_indices.sort_by(|a, b| b.time_index.cmp(&a.time_index));
        // note_indices.reverse();
        let mut removed_notes: MidiNotes = MidiNotes::new();
        for NoteIndex { pitch_index, time_index } in note_indices.clone() {
            // println!("removing note at pitch_index: {}, time_index: {}", pitch_index, time_index);

            let note = self.notes[pitch_index].remove(time_index);
            removed_notes.add(&note);
        }
        removed_notes
    }
}

impl From<Vec<MidiNote>> for MidiNotes {
    fn from(notes: Vec<MidiNote>) -> Self {
        let mut midi_notes = Self::new();
        for note in notes {
            midi_notes.add(&note);
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

    pub fn get_pitch(&self) -> u8 {
        self.pitch.get()
    }

    pub fn get_scaled_pitch(&mut self, scale: &Scale) -> i8 {
        let chromatic_starting_pitch = self.pitch.get();

        let scale_starting_pitch =
            scale.from_chromatic_index_to_scale_index(chromatic_starting_pitch) as i8;

        scale_starting_pitch

        // scale_starting_pitch as i16 + cursor_delta.y as i16
    }

    pub fn get_new_start(&self, cursor_delta: Vector) -> f32 {
        self.start + cursor_delta.x
    }

    // pub fn reposition(&mut self, cursor_delta: Vector, grid: &Grid) {
    pub fn reposition(&mut self, delta_pitch: i8, delta_time: f32, scale: &Scale) -> i8 {
        let chromatic_starting_pitch = self.pitch.get();

        let scale_starting_pitch =
            scale.from_chromatic_index_to_scale_index(chromatic_starting_pitch) as i8;

        let new_pitch_index = (scale_starting_pitch + delta_pitch) as usize;

        // // if the new_pitch_index is out of bounds, cancel the move
        // if new_pitch_index >= grid.scale.midi_size() {
        //     new_pitch_index = scale_starting_pitch as usize;
        // }

        // println!("new_pitch_index: {}", new_pitch_index);

        let new_pitch = scale.midi_range[new_pitch_index] as i16;

        // let delta_time = cursor_delta.x;
        self.pitch = Pitch(new_pitch);

        self.start = self.start + delta_time;
        self.end = self.end + delta_time;

        return new_pitch as i8 - chromatic_starting_pitch as i8;
    }

    pub fn resize(&mut self, resize_end: NoteEdge, delta_time: f32) {
        let mut new_end_time = self.end;
        let mut new_start_time = self.start;
        match resize_end {
            NoteEdge::Start => {
                new_start_time = self.start + delta_time;
            }
            NoteEdge::End => {
                new_end_time = self.end + delta_time;
            }
            _ => {}
        }

        let start_time = new_start_time.min(new_end_time).max(1.0);
        let end_time = new_start_time.max(new_end_time);

        *self = MidiNote::new(start_time, end_time, self.pitch);
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
    // pub original_note_indices: Vec<NoteIndex>,
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
    WriteNoteMode(bool), // adding notes if mouse is pressed
}

impl Default for NoteInteraction {
    fn default() -> Self {
        Self::None
    }
}

impl NoteInteraction {
    pub fn toggle_write_mode(&mut self) {
        match self {
            Self::WriteNoteMode(_) => *self = Self::None,
            _ => *self = Self::WriteNoteMode(false),
        }
    }

    pub fn is_write_mode(&self) -> bool {
        match self {
            Self::WriteNoteMode(_) => true,
            _ => false,
        }
    }

    pub fn is_writing(&self) -> bool {
        match self {
            Self::WriteNoteMode(true) => true,
            _ => false,
        }
    }

    pub fn handle_resizing(
        &self,
        music_scale_cursor: Point,
        track: &crate::track::Track,
    ) -> (event::Status, Option<crate::track::TrackMessage>) {
        if let NoteInteraction::Resizing { initial_cursor_pos, original_notes, resize_end } = self {
            let cursor_delta = music_scale_cursor - *initial_cursor_pos;

            if cursor_delta.x == track.last_delta_time {
                return (event::Status::Ignored, None);
            }

            return (
                event::Status::Captured,
                Some(TrackMessage::ResizedNotes {
                    delta_time: cursor_delta.x,
                    original_notes: original_notes.clone(),
                    resize_end: resize_end.clone(),
                }),
            );
        } else {
            return (event::Status::Ignored, None);
        }
    }

    pub fn handle_selecting(
        &self,
        projected_cursor: Point,
        music_scale_cursor: Point,
    ) -> (event::Status, Option<crate::track::TrackMessage>) {
        if let NoteInteraction::Selecting { initial_music_cursor, initial_cursor_proj } = self {
            let cursor_delta = music_scale_cursor - *initial_music_cursor;

            let selecting_square =
                Rectangle::new(*initial_music_cursor, Size::new(cursor_delta.x, cursor_delta.y));

            let direct_cursor_delta = projected_cursor - *initial_cursor_proj;
            let direct_selecting_square = Rectangle::new(
                *initial_cursor_proj,
                Size::new(direct_cursor_delta.x, direct_cursor_delta.y),
            );

            return (
                event::Status::Captured,
                Some(TrackMessage::Selecting { selecting_square, direct_selecting_square }),
            );
        } else {
            return (event::Status::Ignored, None);
        }
    }

    pub fn handle_dragging(
        &self,
        projected_cursor: Point,
        track: &crate::track::Track,
    ) -> (event::Status, Option<crate::track::TrackMessage>) {
        if let NoteInteraction::Dragging { initial_cursor_pos, original_notes } = self {
            let mut floor_cursor = Vector::new(projected_cursor.x, projected_cursor.y.floor());
            let mut floor_initial_cursor =
                Vector::new(initial_cursor_pos.x, initial_cursor_pos.y.floor());

            let mut cursor_delta: Vector = (projected_cursor - *initial_cursor_pos).into();

            // snap to beat
            if !track.modifiers.alt() {
                // music_floor_cursor.x = music_floor_cursor.x.floor();
                // music_floor_initial_cursor.x = music_floor_initial_cursor.x.floor();
                // music_cursor_delta =
                //     (music_floor_cursor - music_floor_initial_cursor).into();

                floor_cursor.x = floor_cursor.x.floor();
                floor_initial_cursor.x = floor_initial_cursor.x.floor();
                cursor_delta = (floor_cursor - floor_initial_cursor).into();
            }

            // TODO: check if the notes actually changed position,
            // if they didn't, don't send a message!!!

            if cursor_delta == track.last_cursor_delta {
                return (event::Status::Ignored, None);
            }

            return (
                event::Status::Captured,
                Some(TrackMessage::Dragged {
                    cursor_delta: cursor_delta,
                    original_notes: original_notes.clone(),
                }),
            );
        } else {
            return (event::Status::Ignored, None);
        }
    }

    pub fn handle_note_writing(
        &self,
        music_scale_cursor: Point,
        track: &crate::track::Track,
    ) -> (event::Status, Option<crate::track::TrackMessage>) {
        if self.is_writing() {
            // if there is already a note under the cursor, ignore the mouse press
            if let Some(OverNote { .. }) =
                track.midi_notes.get_note_under_cursor(&track.grid, music_scale_cursor)
            {
                return (event::Status::Ignored, None);
            }
            if let Some(OverNote { .. }) =
                track.selected.notes.get_note_under_cursor(&track.grid, music_scale_cursor)
            {
                return (event::Status::Ignored, None);
            }

            let pitch = Pitch(music_scale_cursor.y.floor() as i16);
            let start = music_scale_cursor.x.floor();
            let end = start + 1.0;

            let note = MidiNote::new(start, end, pitch);

            return (event::Status::Captured, Some(TrackMessage::AddNote(note)));
        } else {
            return (event::Status::Ignored, None);
        }
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
