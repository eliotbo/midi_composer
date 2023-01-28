use crate::note::midi_notes::{MidiNote, MidiNotes, NoteEdge, NoteIndex};
use crate::track::undoredo::{AddedNote, ResizedConflicts, TrackHistory};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TrackAction {
    AddNote {
        added_note: AddedNote,
        // note_index_after: NoteIndex,
        // note_to_add: MidiNote,
        // resized_notes: Vec<ResizedNote>,
        // removed_notes: Vec<DeletedNote>,
    },
    AddManyNotes {
        added_notes: Vec<AddedNote>,
        // note_indices_after: Vec<NoteIndex>,
        // notes_to_add: MidiNotes,
    },
    RemoveNote {
        note_index_before: NoteIndex,
        note_before: MidiNote,
    },
    RemoveSelectedNotes {
        deleted_notes: MidiNotes,
    },
    DraggedNotes {
        drag: crate::track::Drag,
        scale: crate::note::scale::Scale,
    },
    ResizedNotes {
        delta_time: f32,
        resize_end: NoteEdge,
        resized_conflicts: Vec<ResizedConflicts>,
    },
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
