use crate::note::midi_notes::MidiNotes;

// use std::collections::HashMap;
// The only actions that matter are the ones that change the main MidiNotes
// it would be much easier

pub type TrackId = u32;

#[derive(Debug, Clone, Default)]
pub struct History {
    pub action_sequence: Vec<Action>,
    pub head_position: usize,
    pub current_size: usize,
    pub is_dummy: bool,
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

    pub fn redo(&mut self) -> Option<Action> {
        // println!("head position: {}", self.head_position);
        let action = self.action_sequence.get(self.head_position).cloned();
        if self.head_position < self.action_sequence.len() {
            self.head_position += 1;
        } else {
            println!("head position: {}", self.head_position);
            return None;
        }
        action
    }

    pub fn add_action_from_track(&mut self, track_id: TrackId) {
        self.action_sequence.truncate(self.head_position);
        self.action_sequence.push(Action::FromTrackId(track_id));
        self.head_position += 1;
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    FromTrackId(TrackId),
    None,
}

#[allow(dead_code)]
pub enum ClipBoard {
    Notes { notes: MidiNotes, player_head: f32 },
    None,
}
