use crate::note::midi_notes::{MidiNote, MidiNotes, NoteEdge, NoteIndex};
use crate::track::actions::{SelectionAction, TrackAction};

#[derive(Debug, Clone, Default)]
pub struct TrackHistory {
    pub action_sequence: Vec<TrackAction>,
    pub head_position: usize,
    pub current_size: usize,
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
        if self.head_position <= self.action_sequence.len() {
            self.head_position += 1;
        } else {
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
    pub resized_notes: Vec<ResizedConflicts>,
    pub removed_notes: Vec<DeletedNote>,
}
