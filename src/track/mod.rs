pub mod actions;
// pub mod undoredo;

pub use iced_native;

use iced::widget::canvas::event::{self, Event};

use iced::widget::canvas::{self};
use iced::widget::canvas::{Cache, Canvas, Cursor, Geometry};
use iced::{
    keyboard::{self, Modifiers},
    mouse, Color, Length, Point, Rectangle, Size, Vector,
};

use crate::grid::{Grid, GridInteraction};
use crate::note::midi_notes::{
    ChangeSelection, MidiNote, MidiNotes, NoteEdge, NoteIndex, NoteInteraction, OverNote, Pitch,
    Selected, WritingMode,
};
use crate::note::scale::{Scale, ScaleType};
use crate::piano_theme::PianoTheme;

use crate::config::{MAX_SCALING, MIN_SCALING};
use crate::track::actions::{SelectionAction, TrackAction, TrackHistory};
use crate::util::{History, TrackId};

pub type TrackElement<'a> = iced::Element<'a, TrackMessage, iced::Renderer<PianoTheme>>;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Drag {
    delta_pitch: i8,
    delta_time: f32,
}

impl Drag {
    const ZERO: Self = Self { delta_pitch: 0, delta_time: 0.0 };
}

pub struct Track {
    track_id: TrackId,
    notes_cache: Cache,
    grid_cache: Cache,
    selection_square_cache: Cache,
    selected_notes_cache: Cache,
    pub selected: Selected,
    pub midi_notes: MidiNotes,
    pub grid: Grid,
    pub meta: TrackMeta,
    pub channel: u8,
    pub timing_info: TimingInfo,
    pub is_active: bool,
    pub modifiers: keyboard::Modifiers,
    pub last_cursor_delta: Vector,
    pub last_delta_time: f32,
    pub drag: Drag,

    pub do_unlock: bool,
    track_history: TrackHistory,
}

#[derive(Debug, Clone)]
pub struct TrackMeta {
    pub name: String,
    pub bpm: f32,
}

impl Default for TrackMeta {
    fn default() -> Self {
        Self { name: "Track 1".to_string(), bpm: 120.0 }
    }
}

impl Track {
    pub fn new(track_id: TrackId) -> Self {
        let mut midi_notes = MidiNotes::new();

        let note0 = MidiNote::new(1.0, 2.5, Pitch::new(53));
        let note1 = MidiNote::new(2.0, 3.5, Pitch::new(55));
        let note2 = MidiNote::new(3.0, 4.5, Pitch::new(53));

        midi_notes.add(&note0);
        midi_notes.add(&note1);
        midi_notes.add(&note2);

        Self {
            track_id,
            grid_cache: Cache::default(),
            notes_cache: Cache::default(),
            selection_square_cache: Cache::default(),
            selected_notes_cache: Cache::default(),
            selected: Selected::default(),
            midi_notes,
            grid: Grid::default(),
            channel: 0,
            meta: TrackMeta::default(),
            timing_info: TimingInfo::default(),
            is_active: false,
            modifiers: keyboard::Modifiers::default(),
            last_cursor_delta: Vector::default(),
            last_delta_time: 0.0,
            drag: Drag::default(),
            track_history: TrackHistory::default(),
            do_unlock: false,
        }
    }

    pub fn view(&self) -> TrackElement {
        Canvas::new(self).width(Length::Fill).height(Length::Fill).into()
    }

    pub fn remove_notes_with_conflicts(
        &mut self,
        added_notes: &Vec<crate::track::actions::AddedNote>,
    ) -> MidiNotes {
        let added_notes2 = &mut added_notes.clone();
        added_notes2
            .sort_by(|a, b| b.note_index_after.time_index.cmp(&a.note_index_after.time_index));
        added_notes2.reverse();

        let mut removed_notes: MidiNotes = MidiNotes::new();
        for crate::track::actions::AddedNote {
            note_index_after: NoteIndex { pitch_index, time_index },
            conflicts_with_selected: conflicts,
            ..
        } in added_notes2.iter()
        {
            crate::track::actions::TrackAction::handle_conflicts(self, &conflicts);

            let note = self.selected.notes.notes[*pitch_index].remove(*time_index);
            removed_notes.add(&note);
        }
        removed_notes
    }

    // adds a note to the MidiNotes, and checks for the necessary sideeffect
    pub fn add_note(&mut self, message: &TrackMessage, history: &mut History) {
        if let TrackMessage::AddNote { note, add_mode } = message {
            if let AddMode::Drain = add_mode {
                self.update(
                    &TrackMessage::UpdateSelection {
                        change_selection: ChangeSelection::DrainSelect,
                    },
                    history,
                );
            }

            let conflicts = self.midi_notes.resolve_conflicts_single(&note);
            self.selected_notes_cache.clear();
            self.notes_cache.clear();

            let added_note = self.selected.notes.add(&note);

            // if !added_note.conflicts_with_selected.deleted.is_empty() {
            //     println!(
            //         "deleted selected notes: {:?}",
            //         added_note.conflicts_with_selected.deleted
            //     );
            // }

            // println!("added note: {:?}", added_note);
            if !history.is_dummy {
                history.add_action_from_track(self.track_id);
                self.track_history.add_track_action(TrackAction::AddNote {
                    added_note,
                    conflicts,
                    message: message.clone(),
                });
            }
        }
    }

    pub fn update(&mut self, message: &TrackMessage, history: &mut History) {
        let message = message.clone();
        match message {
            TrackMessage::Toggle => {
                // println!("Toggled");
                match self.grid.scale.scale_type {
                    ScaleType::Chromatic => self.grid.scale.set_scale_type(ScaleType::Minor),
                    ScaleType::Minor => self.grid.scale.set_scale_type(ScaleType::Chromatic),
                    _ => {}
                }
                self.notes_cache.clear();
                self.grid_cache.clear();
                self.selected_notes_cache.clear();
            }
            TrackMessage::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            TrackMessage::Translated { translation } => {
                self.grid.translation = translation;

                self.notes_cache.clear();
                self.grid_cache.clear();
                self.selected_notes_cache.clear();
            }

            TrackMessage::Scaled { scaling, translation } => {
                self.grid.scaling = scaling;

                if let Some(translation) = translation {
                    self.grid.translation = translation;
                }

                self.notes_cache.clear();
                self.grid_cache.clear();
                self.selected_notes_cache.clear();
            }
            // TrackMessage::LeftClick => {
            //     self.notes_cache.clear();
            // }
            TrackMessage::CursorMoved => {
                self.notes_cache.clear();
            }

            // locks messages that have effects on midi_notes and selected notes,
            // to avoid race conditions
            //
            TrackMessage::ResetLock => {
                self.do_unlock = false;
            }

            TrackMessage::UpdateSelection { change_selection } => {
                self.do_unlock = true;

                let msg =
                    TrackMessage::UpdateSelection { change_selection: change_selection.clone() };

                match change_selection {
                    ChangeSelection::DrainSelect => {
                        // println!("nbr note selected: {}", self.selected.notes.number_of_notes);
                        if !(self.selected.notes.number_of_notes == 0) {
                            // println!("Drain Select");
                            let added_notes = self.selected.notes.drain(&mut self.midi_notes);
                            let new_indices =
                                added_notes.iter().map(|x| x.note_index_after).collect();

                            if !history.is_dummy {
                                self.track_history.add_selection(SelectionAction::DrainSelect {
                                    message: msg,
                                    new_indices,
                                });
                            }
                        }
                    }

                    ChangeSelection::SelectAll => {
                        // println!("Select All");
                        let drained_notes = self.midi_notes.drain(&mut self.selected.notes);
                        let new_indices =
                            drained_notes.iter().map(|x| x.note_index_after).collect();
                        if !history.is_dummy {
                            self.track_history.add_selection(SelectionAction::SelectAllNotes {
                                message: msg,
                                new_indices,
                            });
                        }
                    }

                    ChangeSelection::UnselectOne { ref note_index } => {
                        // println!("Unselect One");
                        let note = self.selected.notes.remove(note_index);
                        let added_note = self.midi_notes.add(&note);

                        if !history.is_dummy {
                            self.track_history.add_selection(SelectionAction::UnselectOne {
                                message: msg,

                                new_index: added_note.note_index_after,
                            });
                        }
                    }

                    ChangeSelection::UnselectAllButOne { ref note_index } => {
                        // println!("Unselect All But One");
                        let selected_note = self.selected.notes.remove(note_index);
                        let drained_notes = self.selected.notes.drain(&mut self.midi_notes);
                        let new_indices =
                            drained_notes.iter().map(|x| x.note_index_after).collect();
                        let added_note = self.selected.notes.add(&selected_note);

                        if !history.is_dummy {
                            self.track_history.add_selection(SelectionAction::UnselectAllButOne {
                                message: msg,
                                new_indices,
                                new_note_index: added_note.note_index_after,
                            });
                        }
                    }

                    ChangeSelection::AddOneToSelected { ref note_index } => {
                        // println!("Add One To Selected");
                        let note = self.midi_notes.remove(note_index);
                        let added_note = self.selected.notes.add(&note);
                        if !history.is_dummy {
                            self.track_history.add_selection(SelectionAction::AddOneToSelected {
                                message: msg,
                                new_index: added_note.note_index_after,
                            });
                        }
                    }

                    ChangeSelection::SelectOne { ref note_index } => {
                        // println!("Select One");
                        let selected_note = self.midi_notes.remove(note_index);
                        let added_notes = self.selected.notes.drain(&mut self.midi_notes);
                        let new_indices = added_notes.iter().map(|x| x.note_index_after).collect();
                        let added_note = self.selected.notes.add(&selected_note);

                        if !history.is_dummy {
                            self.track_history.add_selection(SelectionAction::SelectOne {
                                message: msg,
                                new_indices,
                                new_note_index: added_note.note_index_after,
                            });
                        }
                    }

                    ChangeSelection::SelectMany { ref note_indices } => {
                        if !note_indices.is_empty() {
                            // println!("Select Many");
                            let removed_notes = self.midi_notes.remove_notes(&note_indices.clone());
                            let added_notes = self.selected.notes.add_midi_notes(&removed_notes);
                            let new_indices =
                                added_notes.iter().map(|x| x.note_index_after).collect();

                            // println!(
                            //     "number of selected notes: {}",
                            //     self.selected.notes.number_of_notes
                            // );
                            if !history.is_dummy {
                                self.track_history.add_selection(
                                    SelectionAction::SelectManyNotes { message: msg, new_indices },
                                );
                            }
                        }
                    }
                };
                self.selected.selecting_square = None;
                self.selected.direct_selecting_square = None;
                self.selected_notes_cache.clear();
                self.notes_cache.clear();
                self.selection_square_cache.clear();
            }

            TrackMessage::Dragged { cursor_delta, original_notes } => {
                self.last_cursor_delta = cursor_delta;

                let mut modified_notes: MidiNotes = original_notes.clone();
                let (delta_time, delta_pitch) =
                    modified_notes.drag_all_notes(cursor_delta, &self.grid);

                self.drag.delta_time = delta_time;
                self.drag.delta_pitch = delta_pitch;

                self.selected.notes.clear();
                self.selected.notes.add_midi_notes(&modified_notes);
                self.selected_notes_cache.clear();
                self.notes_cache.clear();
            }

            TrackMessage::FinishDragging { drag, scale } => {
                self.do_unlock = true;

                let conflicts = self.midi_notes.resolve_conflicts(&self.selected.notes);

                if !history.is_dummy {
                    history.add_action_from_track(self.track_id);
                    self.track_history.add_track_action(TrackAction::DraggedNotes {
                        // drag,
                        // scale,
                        conflicts,
                        message: TrackMessage::FinishDragging { drag, scale },
                    });
                }
            }

            TrackMessage::ResizedNotes { mut original_notes, delta_time, resize_end } => {
                self.last_delta_time = delta_time;
                let delta_time = original_notes.resize_all_notes(resize_end, delta_time);
                self.drag.delta_time = delta_time;
                self.selected.notes = original_notes;

                self.selected_notes_cache.clear();
            }

            TrackMessage::FinishResizingNote { delta_time, resize_end } => {
                self.do_unlock = true;

                let resized_conflicts = self.selected.notes.resolve_self_resize_conflicts();
                let conflicts = self.midi_notes.resolve_conflicts(&self.selected.notes);

                if !history.is_dummy {
                    history.add_action_from_track(self.track_id);
                    self.track_history.add_track_action(TrackAction::ResizedNotes {
                        // delta_time,
                        // resize_end,
                        resized_conflicts,
                        conflicts,
                        message: TrackMessage::FinishResizingNote { delta_time, resize_end },
                    });
                }
            }

            TrackMessage::Selecting { selecting_square, direct_selecting_square } => {
                self.selected.selecting_square = Some(selecting_square);
                self.selected.direct_selecting_square = Some(direct_selecting_square);
                self.selection_square_cache.clear();
            }

            m @ TrackMessage::DeleteSelectedNotes => {
                self.do_unlock = true;
                // let notes_before_deletion = self.midi_notes.remove_notes(&notes_to_delete);
                let deleted_notes = self.selected.notes.delete_all();
                // println!("deleted notes: {}", deleted_notes.number_of_notes);
                self.notes_cache.clear();
                self.selected_notes_cache.clear();

                if !history.is_dummy {
                    history.add_action_from_track(self.track_id);
                    self.track_history.add_track_action(TrackAction::RemoveSelectedNotes {
                        deleted_notes,
                        message: m,
                    });
                }
            }

            m @ TrackMessage::AddNote { .. } => {
                self.do_unlock = true;

                self.add_note(&m, history);
            }

            TrackMessage::DeleteOne { note_index_before, is_selected } => {
                self.do_unlock = true;

                let note_before = if !is_selected {
                    self.update(
                        &TrackMessage::UpdateSelection {
                            change_selection: ChangeSelection::DrainSelect,
                        },
                        history,
                    );
                    self.midi_notes.remove(&note_index_before)
                } else {
                    self.selected.notes.remove(&note_index_before)
                };
                self.notes_cache.clear();
                self.selected_notes_cache.clear();

                if !history.is_dummy {
                    history.add_action_from_track(self.track_id);
                    self.track_history.add_track_action(TrackAction::RemoveNote {
                        note_index_before,
                        note_before,
                        is_selected,
                        message: TrackMessage::DeleteOne { note_index_before, is_selected },
                    });
                }
            }

            TrackMessage::AddManyNotes { notes } => {
                self.do_unlock = true;

                let added_notes = self.selected.notes.add_midi_notes(&notes);
                self.selected_notes_cache.clear();
                if !history.is_dummy {
                    history.add_action_from_track(self.track_id);
                    self.track_history.add_track_action(TrackAction::AddManyNotes {
                        added_notes,
                        message: (TrackMessage::AddManyNotes { notes }),
                    });
                }
            }

            // TODO: mechanism for getting out of the loop in case of an unexpected state of History
            TrackMessage::Undo => loop {
                if let Some(track_action) = self.track_history.undo() {
                    // println!("Undoing track action: {:?}", track_action);
                    track_action.handle_undo(self);

                    match track_action {
                        TrackAction::SelectionAction { .. } => continue,
                        _ => break,
                    }
                } else {
                    break;
                }
            },

            TrackMessage::Redo => loop {
                if let Some(track_action) = self.track_history.redo() {
                    // println!("Redoing track action: {:?}", track_action);
                    track_action.handle_redo(self);

                    match track_action {
                        TrackAction::SelectionAction { .. } => continue,
                        _ => break,
                    }
                } else {
                    break;
                }
            },
        }
    }

    fn keyboard_key(
        &self,
        event: Event,
        track_state: &mut TrackState,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                Some((event::Status::Captured, Some(TrackMessage::ModifiersChanged(modifiers))))
            }

            // Debug
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::D, ..
            }) => {
                println!("");
                println!("");
                println!("len : {}", self.track_history.action_sequence.len());
                for act in &self.track_history.action_sequence {
                    println!("------------------");
                    println!("{:#?}", act);
                }

                Some((event::Status::Captured, None))
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::B, ..
            }) => {
                track_state.note_interaction.toggle_write_mode();
                Some((event::Status::Captured, None))
            }
            _ => None,
        }
    }

    fn start_pan(
        &self,
        event: Event,
        track_state: &mut TrackState,
        cursor_position: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) = event {
            track_state.grid_interaction = GridInteraction::Panning {
                translation: self.grid.translation,
                start: cursor_position,
            };

            return Some((event::Status::Captured, None));
        }
        None
    }

    fn pan_grid(
        &self,
        event: Event,
        track_state: &mut TrackState,
        cursor_position: Point,
        bounds: Rectangle,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
            if let GridInteraction::Panning { translation, start } = track_state.grid_interaction {
                let mut new_translation = Vector::new(
                    translation.x + (cursor_position.x - start.x) / self.grid.scaling.x,
                    translation.y - (cursor_position.y - start.y) / self.grid.scaling.y,
                );

                self.grid.limit_to_bounds(&mut new_translation, bounds, self.grid.scaling);

                return Some((
                    event::Status::Captured,
                    Some(TrackMessage::Translated { translation: new_translation }),
                ));
            };
        }
        return None;
    }

    fn end_pan(
        &self,
        event: Event,
        track_state: &mut TrackState,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) = event {
            if let GridInteraction::Panning { .. } = track_state.grid_interaction {
                track_state.grid_interaction = GridInteraction::None;
                return Some((event::Status::Captured, None));
            }

            return Some((event::Status::Ignored, None));
        }
        return None;
    }

    fn zoom(
        &self,
        event: Event,
        cursor: Cursor,
        bounds: Rectangle,
        cursor_in_bounds: bool,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    if !cursor_in_bounds {
                        return Some((event::Status::Ignored, None));
                    }

                    if y < 0.0 && self.grid.scaling.x <= MIN_SCALING.x
                        || y > 0.0 && self.grid.scaling.x >= MAX_SCALING.x
                        || y < 0.0 && self.grid.scaling.y <= MIN_SCALING.y
                        || y > 0.0 && self.grid.scaling.y >= MAX_SCALING.y
                    {
                        return Some((event::Status::Captured, None));
                    }

                    let old_scaling = self.grid.scaling;
                    let factor0 = Vector::new(
                        (1.0 + y / 30.0).max(MIN_SCALING.x).min(MAX_SCALING.x),
                        (1.0 + y / 30.0).max(MIN_SCALING.y).min(MAX_SCALING.y),
                    );

                    let scaling = Vector::new(
                        self.grid.scaling.x * factor0.x,
                        self.grid.scaling.y * factor0.y,
                    );

                    let translation =
                        if let Some(cursor_to_center) = cursor.position_from(bounds.center()) {
                            let factor = scaling - old_scaling;

                            let mut new_translation = self.grid.translation
                                - Vector::new(
                                    cursor_to_center.x * factor.x / (scaling.x * scaling.x),
                                    cursor_to_center.y * factor.y / (scaling.y * scaling.y),
                                );

                            self.grid.limit_to_bounds(&mut new_translation, bounds, scaling);

                            Some(new_translation)
                        } else {
                            None
                        };

                    return Some((
                        event::Status::Captured,
                        Some(TrackMessage::Scaled { scaling, translation }),
                    ));
                }
            }
        }
        return None;
    }

    fn cursor_moved(
        &self,
        event: Event,
        track_state: &mut TrackState,
        projected_cursor: Point,
        music_scale_cursor: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        //
        //

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            | Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
            | Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => return None,
            _ => {}
        };

        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
            //
            //
            if let NoteInteraction::Writing { .. } = track_state.note_interaction {
                return None;
            }

            match &mut track_state.note_interaction {
                inter @ NoteInteraction::Resizing { .. } => {
                    return Some(inter.handle_resizing(music_scale_cursor, self));
                }
                inter @ NoteInteraction::Selecting { .. } => {
                    return Some(inter.handle_selecting(projected_cursor, music_scale_cursor));
                }
                inter @ NoteInteraction::Dragging { .. } => {
                    return Some(inter.handle_dragging(projected_cursor, self));
                }

                _ => {}
            };

            let mut over_note =
                self.selected.notes.get_note_under_cursor(&self.grid, music_scale_cursor);

            if let None = over_note {
                over_note = self.midi_notes.get_note_under_cursor(&self.grid, music_scale_cursor);
            }

            // check if the mouse is over a note or the edge of a note
            match over_note {
                //
                Some(OverNote { note_index: _, note_edge: NoteEdge::Start })
                | Some(OverNote { note_index: _, note_edge: NoteEdge::End }) => {
                    track_state.note_interaction = NoteInteraction::ResizingHover;
                }

                _ => {
                    // println!("watson");
                    track_state.note_interaction = NoteInteraction::None;
                }
            };
        }
        None
    }

    fn init_drag_or_resize(
        &self,
        event: Event,
        track_state: &mut TrackState,
        projected_cursor: Point,
        music_scale_cursor: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if track_state.note_interaction.is_write_mode() {
            return None;
        }

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            // Check if a Selected note has been clicked
            if let Some(OverNote { note_index: _, note_edge }) =
                self.selected.notes.get_note_under_cursor(&self.grid, music_scale_cursor)
            {
                let new_selection = self.selected.notes.clone();

                track_state.drag_or_resize(note_edge, projected_cursor, new_selection);

                return Some((event::Status::Captured, None));
            }
        }

        return None;
    }

    fn init_pen_or_select(
        &self,
        event: Event,
        track_state: &mut TrackState,
        projected_cursor: Point,
        music_scale_cursor: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        // TODO change the resize note rectangle
        //
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            // println!("reached here");
            // write or delete a note if in pen mode
            if track_state.note_interaction.is_write_mode() {
                // println!("inside write mode");
                return Some(
                    track_state.note_interaction.init_note_writing(music_scale_cursor, &self),
                );
            }

            // Check if a non-Selected note has been clicked
            if let Some(OverNote { note_index, note_edge }) =
                self.midi_notes.get_note_under_cursor(&self.grid, music_scale_cursor)
            {
                let mut new_selected = self.selected.notes.clone();
                let note = self.midi_notes.get(note_index);

                // if the control key is pressed
                //
                let message = if self.modifiers.control() {
                    //
                    // add the clicked note to the Selected notes
                    let added_note = new_selected.add(&note);

                    (
                        event::Status::Captured,
                        Some(TrackMessage::UpdateSelection {
                            change_selection: ChangeSelection::AddOneToSelected {
                                note_index: added_note.note_index_after,
                            },
                        }),
                    )
                } else {
                    // if the control key is not pressed, clear the Selected notes and
                    // select the clicked note
                    //
                    new_selected.clear();
                    new_selected.add(&note);

                    (
                        event::Status::Captured,
                        Some(TrackMessage::UpdateSelection {
                            change_selection: ChangeSelection::SelectOne { note_index },
                        }),
                    )
                };

                track_state.drag_or_resize(note_edge, projected_cursor, new_selected);

                return Some(message);
            }

            // return Some((event::Status::Captured, None));
        }
        // _ => return None,
        return None;
    }

    fn change_notes(
        &self,
        event: Event,
        track_state: &mut TrackState,
        music_scale_cursor: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        //

        if let Event::Keyboard(keyboard::Event::KeyPressed {
            key_code: keyboard::KeyCode::Delete,
            ..
        }) = event
        {
            if self.is_active {
                return Some((event::Status::Captured, Some(TrackMessage::DeleteSelectedNotes)));
            } else {
                return None;
            }
        }

        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) = event {
            //
            // println!("blah");
            match track_state.note_interaction {
                NoteInteraction::Writing { .. } => {
                    track_state.note_interaction =
                        NoteInteraction::Writing { writing_mode: WritingMode::None };
                }
                //
                NoteInteraction::Selecting { initial_music_cursor, .. } => {
                    // println!("blah2");
                    let delta_cursor_pos = music_scale_cursor - initial_music_cursor;
                    let size = Size::new(delta_cursor_pos.x, delta_cursor_pos.y);

                    let selecting_square = Rectangle::new(initial_music_cursor, size);
                    track_state.note_interaction = NoteInteraction::None;

                    let note_indices = self.midi_notes.get_notes_in_rect(selecting_square);

                    return Some((
                        event::Status::Captured,
                        Some(TrackMessage::UpdateSelection {
                            change_selection: ChangeSelection::SelectMany { note_indices },
                        }),
                    ));
                }
                NoteInteraction::Dragging { .. } => {
                    track_state.note_interaction = NoteInteraction::None;

                    if self.drag == Drag::ZERO {
                        return Some((event::Status::Ignored, None));
                    }

                    return Some((
                        event::Status::Captured,
                        Some(TrackMessage::FinishDragging {
                            drag: self.drag,
                            scale: self.grid.scale.clone(),
                        }),
                    ));
                }
                NoteInteraction::Resizing { resize_end, .. } => {
                    track_state.note_interaction = NoteInteraction::None;

                    if self.drag.delta_time == 0.0 {
                        return Some((event::Status::Ignored, None));
                    }

                    let delta_time = self.drag.delta_time;

                    return Some((
                        event::Status::Captured,
                        Some(TrackMessage::FinishResizingNote { delta_time, resize_end }),
                    ));
                }

                _ => {
                    track_state.note_interaction = NoteInteraction::None;
                }
            }
        }
        return None;
    }

    fn write_or_delete_notes(
        &self,
        event: Event,
        track_state: &mut TrackState,
        music_scale_cursor: Point,
        cursor_in_bounds: bool,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        if !cursor_in_bounds {
            // cannot write outside the track window
            match track_state.note_interaction {
                NoteInteraction::Writing { .. } => {
                    return None;
                }
                _ => {}
            }
        }

        if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
            if let NoteInteraction::Writing { .. } = &track_state.note_interaction {
                let note_writing =
                    track_state.note_interaction.handle_note_writing(music_scale_cursor, &self);
                return note_writing;
            }
        }
        return None;
    }

    fn init_selecting(
        &self,
        event: Event,
        track_state: &mut TrackState,
        projected_cursor: Point,
        music_scale_cursor: Point,
    ) -> Option<(event::Status, Option<TrackMessage>)> {
        // TODO change the resize note rectangle
        //
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            // if no note has been clicked, start selecting
            track_state.note_interaction = NoteInteraction::Selecting {
                initial_music_cursor: music_scale_cursor,
                initial_cursor_proj: projected_cursor,
            };

            // if the control key is not pressed, clear the Selected notes
            if !self.modifiers.control() {
                // println!("clearing selection");
                return Some((
                    event::Status::Captured,
                    Some(TrackMessage::UpdateSelection {
                        change_selection: ChangeSelection::DrainSelect,
                    }),
                ));
            }
        }
        None
    }
}

#[derive(Clone, Debug, Copy)]
pub enum AddMode {
    Drain,
    Custom,
    Normal,
}

#[derive(Clone, Debug)]
pub enum TrackMessage {
    Translated { translation: Vector },
    Scaled { scaling: Vector, translation: Option<Vector> },
    AddNote { note: MidiNote, add_mode: AddMode }, // bool: first_note_of_write_mode
    AddManyNotes { notes: MidiNotes },
    CursorMoved,
    DeleteSelectedNotes,
    DeleteOne { note_index_before: NoteIndex, is_selected: bool },
    UpdateSelection { change_selection: ChangeSelection },
    Dragged { cursor_delta: Vector, original_notes: MidiNotes },
    FinishDragging { drag: Drag, scale: Scale },

    ResizedNotes { delta_time: f32, original_notes: MidiNotes, resize_end: NoteEdge },
    FinishResizingNote { delta_time: f32, resize_end: NoteEdge },

    Selecting { selecting_square: Rectangle, direct_selecting_square: Rectangle },
    // FinishSelecting {
    //     selecting_square: Rectangle,
    //     // keep_already_selected: bool,
    // },
    ModifiersChanged(Modifiers),
    Toggle,
    ResetLock,
    Undo,
    Redo,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum Pending {
    AddNote,
    DeleteNote,
    SelectNote,
    None,
}

pub struct TrackState {
    pub grid_interaction: GridInteraction,
    pub note_interaction: NoteInteraction,
    pub pending: Pending,
}

impl Default for TrackState {
    fn default() -> Self {
        Self {
            grid_interaction: GridInteraction::default(),
            note_interaction: NoteInteraction::default(),
            pending: Pending::None,
        }
    }
}

impl TrackState {
    pub fn drag_or_resize(
        &mut self,
        note_edge: NoteEdge,
        cursor: Point,
        original_notes: MidiNotes,
    ) {
        match note_edge {
            //
            // start dragging
            NoteEdge::None => {
                self.note_interaction =
                    NoteInteraction::Dragging { initial_cursor_pos: cursor, original_notes };
            }

            //
            // start resizing if the click happened on the edge of a note
            note_edge => {
                self.note_interaction = NoteInteraction::Resizing {
                    initial_cursor_pos: cursor,
                    original_notes,
                    resize_end: note_edge,
                };
            }
        }
    }
}

impl canvas::Program<TrackMessage, PianoTheme> for Track {
    type State = TrackState;

    fn update(
        &self,
        track_state: &mut TrackState,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<TrackMessage>) {
        let cursor_position = if let Some(pos) = cursor.position_from(bounds.position()) {
            pos
        } else {
            return (event::Status::Ignored, None);
        };

        let cursor_in_bounds: bool = cursor.is_over(&bounds);
        // a click or a scroll outside the track window has not effect
        if !cursor_in_bounds {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(_))
                | Event::Mouse(mouse::Event::WheelScrolled { .. }) => {
                    return (event::Status::Ignored, None);
                }
                _ => {}
            }
        }

        // start panning grid with middle mouse button
        if let Some(msg) = self.start_pan(event, track_state, cursor_position) {
            return msg;
        }
        if let Some(msg) = self.pan_grid(event, track_state, cursor_position, bounds) {
            return msg;
        }
        if let Some(msg) = self.end_pan(event, track_state) {
            return msg;
        }

        // zooming with mouse wheel
        if let Some(msg) = self.zoom(event, cursor, bounds, cursor_in_bounds) {
            return msg;
        }

        // record modifiers, debug, enter write mode
        if let Some(msg) = self.keyboard_key(event, track_state) {
            return msg;
        }

        let projected_cursor = self.grid.to_track_axes(cursor_position, &bounds.size());
        let music_scale_cursor = self.grid.adjust_to_music_scale(projected_cursor);

        if let Some(msg) =
            self.cursor_moved(event, track_state, projected_cursor, music_scale_cursor)
        {
            // println!("cursor moved: {:?}", msg);
            return msg;
        }

        // println!("track interaction-1: {:?}", track_state.note_interaction);

        // potential effects on notes: delete, add, change selected
        if let Some(msg) =
            self.init_drag_or_resize(event, track_state, projected_cursor, music_scale_cursor)
        {
            return msg;
        }

        // println!("track interaction0: {:?}", track_state.note_interaction);

        //
        // If the track is ready for action, change the track state accordingly.
        // And reset the lock
        //
        // The problem is that the unlock message is only sent when an event occurs,
        // which cancels the effect of that event (like FinishSelecting)
        if track_state.pending != Pending::None && self.do_unlock {
            track_state.pending = Pending::None;
            // println!("not pending");
            return (event::Status::Captured, Some(TrackMessage::ResetLock));
        }

        // if a message has not been acted on yet, ignore this event and wait for the
        // message to be acted on
        if track_state.pending != Pending::None {
            // println!("skipping: {:?}", track_state.pending);
            return (event::Status::Captured, None);
        }

        // println!("track interaction1: {:?}", track_state.note_interaction);

        // potential effects on notes: delete, add, change selected
        if let Some(msg) =
            self.init_pen_or_select(event, track_state, projected_cursor, music_scale_cursor)
        {
            // println!("pending 1: {:?}", "a");

            return msg;
        }

        // println!("track interaction2: {:?}", track_state.note_interaction);

        // potential effects on notes: delete selected using Delete key, resize, or drag using mouse
        if let Some(msg) = self.change_notes(event, track_state, music_scale_cursor) {
            // println!("pending 2: {:?}", "a");
            track_state.pending = Pending::AddNote;
            return msg;
        }

        // println!("track interaction3: {:?}", track_state.note_interaction);

        // potential effects on notes: delete 1 note, add 1 note by moving cursor on empty space
        if let Some(msg) =
            self.write_or_delete_notes(event, track_state, music_scale_cursor, cursor_in_bounds)
        {
            // println!("pending 3: {:?}", "a");
            track_state.pending = Pending::AddNote;
            // println!("WHAAT: {:?}", msg);
            return msg;
        }

        // If all else fails, try to start a selecting box
        if let Some(msg) =
            self.init_selecting(event, track_state, projected_cursor, music_scale_cursor)
        {
            // println!("pending 4: {:?}", "a");
            track_state.pending = Pending::AddNote;
            return msg;
        }

        (event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _track_state: &TrackState,
        _theme: &PianoTheme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        // let grid = self.tracks[0].grid;
        let background = self.grid.draw_background(bounds, &self.grid_cache);
        let text_overlay = self.grid.draw_text_and_hover_overlay(bounds, cursor);

        let trans = 0.25;
        let yellow = Color::from_rgba(1.0, 1.0, 0.0, trans);
        let dark_yellow = Color::from_rgba(0.5, 0.5, 0.0, trans);

        let notes_overlay = self.midi_notes.draw_notes(
            &self.grid,
            &bounds,
            &cursor,
            &self.notes_cache,
            yellow,
            trans,
        );

        let selected_notes_elements = self.selected.notes.draw_notes(
            &self.grid,
            &bounds,
            &cursor,
            &self.selected_notes_cache,
            dark_yellow,
            trans,
        );

        let selecting_box =
            self.selected.draw_selecting_square(bounds, &self.grid, &self.selection_square_cache);

        vec![background, notes_overlay, selected_notes_elements, selecting_box, text_overlay]
    }

    fn mouse_interaction(
        &self,
        track_state: &TrackState,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> mouse::Interaction {
        if !cursor.is_over(&bounds) {
            return mouse::Interaction::default();
        }
        match track_state.note_interaction {
            NoteInteraction::Resizing { .. } | NoteInteraction::ResizingHover => {
                mouse::Interaction::ResizingHorizontally
            }
            NoteInteraction::Writing { .. } => mouse::Interaction::Crosshair,

            _ => mouse::Interaction::default(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TimingInfo {
    #[allow(dead_code)]
    pub bpm: f32,
    _time_signature: (u8, u8),
    _beat_per_measure: u8,
    _beat_value: u8, // 4 for quarter note, 8 for eighth note, etc.
    pub track_length: f32,
}

impl Default for TimingInfo {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            _time_signature: (4, 4),
            _beat_per_measure: 4,
            _beat_value: 4,
            track_length: 4.0,
        }
    }
}
