use iced::Vector;

use crate::note::midi_notes::{MidiNote, MidiNotes, NoteEdge, NoteIndex};

use std::collections::HashMap;
// The only actions that matter are the ones that change the main MidiNotes
// it would be much easier

pub type TrackId = u32;

#[derive(Debug, Clone, Default)]
pub struct History {
    pub action_sequence: Vec<Action>,
    pub head_position: usize,
    pub current_size: usize,
}

impl History {
    pub fn undo(&mut self) -> Option<Action> {
        if self.head_position > 0 {
            self.head_position -= 1;
        } else {
            return None;
        }
        self.action_sequence.get(self.head_position).cloned()
    }

    // pub fn undo(&mut self) -> Vec<Action> {
    // println!("head position: {}", self.head_position);
    // let mut actions = vec![];
    // loop {
    //     if self.head_position > 0 {
    //         self.head_position -= 1;
    //     } else {
    //         break;
    //     }

    //     if let Some(action) = self.action_sequence.get(self.head_position) {
    //         actions.push(action.clone());

    //         if let Action::TrackAction(TrackId, crate::util::TrackAction::SelectionAction(_)) =
    //             action
    //         {
    //             break;
    //         }
    //     } else {
    //         break;
    //     }
    // }
    // actions.reverse();
    // actions
    // }
    pub fn redo(&mut self) -> Option<Action> {
        // println!("head position: {}", self.head_position);
        let action = self.action_sequence.get(self.head_position).cloned();
        if self.head_position < self.action_sequence.len() {
            self.head_position += 1;
        } else {
            return None;
        }
        action
    }

    pub fn add_track_action(&mut self, track_id: TrackId, action: TrackAction) {
        self.action_sequence.truncate(self.head_position);
        self.action_sequence.push(Action::TrackAction(track_id, action));
        self.head_position += 1;
    }

    pub fn add_selection(&mut self, track_id: TrackId, action: SelectionAction) {
        self.action_sequence.truncate(self.head_position);
        self.action_sequence
            .push(Action::TrackAction(track_id, TrackAction::SelectionAction(action)));
        self.head_position += 1;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    TrackAction(TrackId, TrackAction),
    None,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrackAction {
    AddNote { note_index_after: NoteIndex, note_to_add: MidiNote },
    AddManyNotes { note_indices_after: Vec<NoteIndex>, notes_to_add: MidiNotes },
    RemoveNote { note_index_before: NoteIndex, note_before: MidiNote },
    RemoveManyNotes { note_indices_before: Vec<NoteIndex>, notes_before_deletion: MidiNotes },
    DraggedNotes { drag: crate::track::Drag, scale: crate::note::scale::Scale },
    ResizedNotes { delta_time: f32, resize_end: NoteEdge },
    SelectionAction(SelectionAction),
}

#[derive(Debug, Clone)]
pub enum SelectionAction {
    DrainSelect {
        new_indices: Vec<NoteIndex>,
    },
    AddOneToSelected {
        note_index: NoteIndex,
        new_index: NoteIndex,
    },
    UnselectOne {
        note_index: NoteIndex,
        new_index: NoteIndex,
    },
    SelectManyNotes {
        note_indices: Vec<NoteIndex>,
        new_indices: Vec<NoteIndex>,
    },
    // DeselectManyNotes {
    //     note_indices: Vec<NoteIndex>,
    //     new_indices: Vec<NoteIndex>,
    // },
    SelectAllNotes {
        new_indices: Vec<NoteIndex>,
    },

    UnselectAllButOne {
        note_index: NoteIndex,
        new_indices: Vec<NoteIndex>,
        new_note_index: NoteIndex,
    },
    SelectOne {
        note_index: NoteIndex,
        new_indices: Vec<NoteIndex>,
        new_note_index: NoteIndex,
    },
}

#[allow(dead_code)]
struct ClipBoard {
    notes: MidiNotes,
}
