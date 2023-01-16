// The only actions that matter are the ones that change the main MidiNotes
// it would be much easier
pub struct History {
    actions: Vec<Action>,
}

enum Action {
    AddNote,
    RemoveNote,
    ChangeNote,
}
