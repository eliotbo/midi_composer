use crate::note::midi_notes::{ChangeSelection, MidiNote, MidiNotes, NoteEdge, NoteIndex};
// use crate::track::undoredo::{AddedNote, ResizedConflicts, TrackHistory};
use crate::track::{AddMode, Track, TrackMessage};
use crate::util::History;

use std::fmt;
// use crate::note::midi_notes::{MidiNote, MidiNotes, NoteEdge, NoteIndex};
// use crate::track::actions::{SelectionAction, TrackAction};
// use crate::track::Track;

#[derive(Clone, Default)]
pub struct TrackHistory {
    pub action_sequence: Vec<TrackAction>,
    pub head_position: usize,
    pub current_size: usize,
}

impl fmt::Debug for TrackHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TrackHistory: {:#?}", self.action_sequence)
    }
}

impl TrackHistory {
    pub fn undo(&mut self) -> Option<TrackAction> {
        if self.head_position > 0 {
            self.head_position -= 1;
        } else {
            return None;
        }
        self.action_sequence.get(self.head_position).cloned()
    }

    pub fn redo(&mut self) -> Option<TrackAction> {
        let action = self.action_sequence.get(self.head_position).cloned();
        // println!("track history: {:?}", self);
        if self.head_position < self.action_sequence.len() {
            self.head_position += 1;
        } else {
            // println!("head position: {}", self.head_position);
            return None;
        }
        action
    }

    pub fn add_track_action(&mut self, action: TrackAction) {
        self.action_sequence.truncate(self.head_position);
        self.action_sequence.push(action);
        self.head_position += 1;
    }

    pub fn add_selection(&mut self, action: SelectionAction) {
        self.action_sequence.truncate(self.head_position);
        self.action_sequence.push(TrackAction::SelectionAction(action));
        self.head_position += 1;
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConflictHistory {
    pub resized: Vec<ResizedConflicts>,
    pub deleted: Vec<DeletedNote>,
}

impl ConflictHistory {
    pub fn add(&mut self, conflict: ConflictHistory) {
        self.resized.extend(conflict.resized);
        self.deleted.extend(conflict.deleted);
    }
}

#[derive(Debug, Clone)]
pub struct ResizedConflicts {
    pub note_index: NoteIndex,
    pub edge: NoteEdge,
    pub delta_time: f32,
}

#[derive(Debug, Clone)]
pub struct DeletedNote {
    pub note_index: NoteIndex,
    pub removed_note: MidiNote,
}

#[derive(Debug, Clone)]
pub struct AddedNote {
    pub note_index_after: NoteIndex,
    pub note_to_add: MidiNote,
    pub conflicts_with_selected: ConflictHistory,
    // pub resized_notes: Vec<ResizedConflicts>,
    // pub removed_notes: Vec<DeletedNote>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrackAction {
    AddNote {
        added_note: AddedNote,
        conflicts: ConflictHistory,
        message: TrackMessage,
    },
    AddManyNotes {
        added_notes: Vec<AddedNote>,
        message: TrackMessage,
    },
    RemoveNote {
        note_index_before: NoteIndex,
        note_before: MidiNote,
        is_selected: bool,
        message: TrackMessage,
    },
    RemoveSelectedNotes {
        deleted_notes: MidiNotes,
        message: TrackMessage,
    },
    DraggedNotes {
        // drag: crate::track::Drag,
        // scale: crate::note::scale::Scale,
        conflicts: ConflictHistory,
        message: TrackMessage,
    },
    ResizedNotes {
        // delta_time: f32,
        // resize_end: NoteEdge,
        resized_conflicts: Vec<ResizedConflicts>,
        conflicts: ConflictHistory,
        message: TrackMessage,
    },
    SelectionAction(SelectionAction),
}

#[derive(Debug, Clone)]
pub enum SelectionAction {
    DrainSelect {
        message: TrackMessage,
        new_indices: Vec<NoteIndex>,
    },
    SelectAllNotes {
        message: TrackMessage,
        new_indices: Vec<NoteIndex>,
    },
    UnselectOne {
        message: TrackMessage,
        new_index: NoteIndex,
    },
    UnselectAllButOne {
        message: TrackMessage,
        new_indices: Vec<NoteIndex>,
        new_note_index: NoteIndex,
    },

    AddOneToSelected {
        message: TrackMessage,
        new_index: NoteIndex,
    },

    SelectManyNotes {
        message: TrackMessage,
        new_indices: Vec<NoteIndex>,
    },

    SelectOne {
        message: TrackMessage,

        new_indices: Vec<NoteIndex>,
        new_note_index: NoteIndex,
    },
}

impl TrackAction {
    pub fn handle_conflicts(track: &mut Track, conflicts: &ConflictHistory) {
        for removed_note in conflicts.deleted.iter() {
            track.midi_notes.add(&removed_note.removed_note);
        }

        for resized_note in conflicts.resized.iter() {
            let note_index = resized_note.note_index;
            let note = &mut track.midi_notes.notes[note_index.pitch_index][note_index.time_index];

            note.resize(resized_note.edge, -resized_note.delta_time);
        }
    }

    pub fn handle_selection_conflicts(track: &mut Track, conflicts: &ConflictHistory) {
        for removed_note in conflicts.deleted.iter() {
            track.selected.notes.remove(&removed_note.note_index);
        }

        for resized_note in conflicts.resized.iter() {
            let note_index = resized_note.note_index;
            let note =
                &mut track.selected.notes.notes[note_index.pitch_index][note_index.time_index];

            note.resize(resized_note.edge, -resized_note.delta_time);
        }
    }

    pub fn handle_undo(&self, track: &mut Track) {
        match self {
            TrackAction::AddNote { added_note, conflicts, .. } => {
                track.selected.notes.remove(&added_note.note_index_after);
                Self::handle_selection_conflicts(track, &added_note.conflicts_with_selected);
                Self::handle_conflicts(track, &conflicts);

                track.notes_cache.clear();
                track.selected_notes_cache.clear();
            }

            TrackAction::AddManyNotes { added_notes, .. } => {
                track.remove_notes_with_conflicts(added_notes);
                track.notes_cache.clear();
                track.selected_notes_cache.clear();
            }

            TrackAction::RemoveNote { note_before, is_selected, .. } => {
                if !is_selected {
                    track.midi_notes.add(note_before);
                    track.notes_cache.clear();
                } else {
                    // println!("undo remove note (selected) ");
                    track.selected.notes.add(note_before);
                    track.selected_notes_cache.clear();
                }
            }

            TrackAction::RemoveSelectedNotes { deleted_notes, .. } => {
                track.selected.notes.add_midi_notes(deleted_notes);
                track.selected_notes_cache.clear();
            }

            TrackAction::DraggedNotes {
                message: TrackMessage::FinishDragging { drag, scale },
                conflicts,
                ..
            } => {
                let mut modified_notes: MidiNotes = track.selected.notes.clone();

                for v in modified_notes.notes.iter_mut() {
                    for note in v.iter_mut() {
                        note.reposition(-drag.delta_pitch, -drag.delta_time, &scale);
                    }
                }

                Self::handle_conflicts(track, &conflicts);

                track.selected.notes.clear();
                track.selected.notes.add_midi_notes(&modified_notes);
                track.selected_notes_cache.clear();
                track.notes_cache.clear();
            }

            TrackAction::ResizedNotes {
                message: TrackMessage::FinishResizingNotes { delta_time, resize_end },
                resized_conflicts,
                conflicts,
            } => {
                for notes_in_pitch in track.selected.notes.notes.iter_mut() {
                    for note in notes_in_pitch.iter_mut() {
                        note.resize(*resize_end, -delta_time);
                    }
                }

                // handle conflicts within the selected notes
                for conflict in resized_conflicts.iter() {
                    let note_index = conflict.note_index;
                    let note = &mut track.selected.notes.notes[note_index.pitch_index]
                        [note_index.time_index];

                    note.resize(conflict.edge, -conflict.delta_time);
                }

                // handle conflicts between selected notes and non-selected notes
                Self::handle_conflicts(track, &conflicts);

                track.selected_notes_cache.clear();
                track.notes_cache.clear();
            }

            TrackAction::SelectionAction(selection_action) => {
                track.selected_notes_cache.clear();
                track.notes_cache.clear();
                match selection_action {
                    SelectionAction::DrainSelect { new_indices, .. } => {
                        // println!("undo drain select");
                        let notes = track.midi_notes.remove_notes(new_indices);
                        track.selected.notes.add_midi_notes(&notes);
                    }
                    SelectionAction::UnselectOne { new_index, .. } => {
                        let note = track.midi_notes.remove(new_index);
                        track.selected.notes.add(&note);
                    }
                    SelectionAction::UnselectAllButOne { new_indices, new_note_index, .. } => {
                        let note = track.selected.notes.remove(new_note_index);
                        let removed_notes = track.midi_notes.remove_notes(new_indices);
                        track.selected.notes.add_midi_notes(&removed_notes);
                        track.selected.notes.add(&note);
                    }

                    SelectionAction::AddOneToSelected { new_index, .. } => {
                        let note = track.selected.notes.remove(new_index);
                        track.midi_notes.add(&note);
                    }
                    SelectionAction::SelectAllNotes { new_indices, .. } => {
                        let notes = track.selected.notes.remove_notes(new_indices);
                        track.midi_notes.add_midi_notes(&notes);
                    }
                    SelectionAction::SelectManyNotes { new_indices, .. } => {
                        let notes = track.selected.notes.remove_notes(new_indices);
                        track.midi_notes.add_midi_notes(&notes);
                    }
                    SelectionAction::SelectOne { new_indices, new_note_index, .. } => {
                        let note = track.selected.notes.remove(new_note_index);
                        let removed_notes = track.midi_notes.remove_notes(new_indices);
                        track.selected.notes.add_midi_notes(&removed_notes);
                        track.midi_notes.add(&note);
                    }
                }
            }
            _ => {
                panic!("undo not implemented for this action: {:?}", self);
            }
        }
    }

    pub fn handle_redo(&self, track: &mut Track) {
        //
        // a redo action should not be recorded in the history. so
        // we pass a dummy history to the update function
        let mut dummy_history = &mut History::default();
        dummy_history.is_dummy = true;
        track.notes_cache.clear();
        track.selected_notes_cache.clear();
        println!("redo self: {:?}", self);
        match self {
            TrackAction::AddNote { message, .. } => track.update(message, dummy_history),
            TrackAction::AddManyNotes { message, .. } => track.update(message, dummy_history),
            TrackAction::RemoveNote { message, .. } => track.update(message, dummy_history),
            TrackAction::RemoveSelectedNotes { message, .. } => {
                track.update(message, dummy_history)
            }
            TrackAction::DraggedNotes { message, .. } => {
                // track.update(message, dummy_history)
                if let TrackMessage::FinishDragging { drag, scale } = message {
                    let mut modified_notes: MidiNotes = track.selected.notes.clone();

                    for v in modified_notes.notes.iter_mut() {
                        for note in v.iter_mut() {
                            note.reposition(drag.delta_pitch, drag.delta_time, &scale);
                        }
                    }

                    track.midi_notes.resolve_conflicts(&track.selected.notes);

                    track.selected.notes.clear();
                    track.selected.notes.add_midi_notes(&modified_notes);

                    track.selected_notes_cache.clear();
                    track.notes_cache.clear();
                }
            }
            TrackAction::ResizedNotes { message, .. } => {
                // track.update(message, dummy_history)
                if let TrackMessage::FinishResizingNotes { delta_time, resize_end } = message {
                    for notes_in_pitch in track.selected.notes.notes.iter_mut() {
                        for note in notes_in_pitch.iter_mut() {
                            note.resize(*resize_end, *delta_time);
                        }
                    }
                    track.selected.notes.resolve_self_resize_conflicts();
                    track.midi_notes.resolve_conflicts(&track.selected.notes);

                    track.selected_notes_cache.clear();
                    track.notes_cache.clear();
                }
            }

            TrackAction::SelectionAction(selection_action) => match selection_action {
                SelectionAction::DrainSelect { message, .. } => {
                    track.update(&message, dummy_history);
                }
                SelectionAction::SelectAllNotes { message, .. } => {
                    track.update(&message, dummy_history);
                }
                SelectionAction::UnselectOne { message, .. } => {
                    track.update(&message, dummy_history);
                }

                SelectionAction::UnselectAllButOne { message, .. } => {
                    track.update(&message, dummy_history);
                }

                SelectionAction::AddOneToSelected { message, .. } => {
                    track.update(&message, dummy_history);
                }
                SelectionAction::SelectOne { message, .. } => {
                    track.update(&message, dummy_history);
                }

                SelectionAction::SelectManyNotes { message, .. } => {
                    track.update(&message, dummy_history);
                }
            },
        };
    }
}
// }
