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
    Selected,
};
use crate::note::scale::ScaleType;
use crate::piano_theme::PianoTheme;

use crate::config::{MAX_SCALING, MIN_SCALING};
use crate::util::{Action, History, SelectionAction, TrackAction, TrackId};

pub type TrackElement<'a> = iced::Element<'a, TrackMessage, iced::Renderer<PianoTheme>>;

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
    pub active: bool,
    pub modifiers: keyboard::Modifiers,
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

        midi_notes.add(note0);
        midi_notes.add(note1);
        midi_notes.add(note2);

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
            active: true,
            modifiers: keyboard::Modifiers::default(),
        }
    }

    pub fn view(&self) -> TrackElement {
        Canvas::new(self).width(Length::Fill).height(Length::Fill).into()
    }

    pub fn update(&mut self, message: TrackMessage, history: &mut History) {
        match message {
            TrackMessage::Toggle => {
                println!("Toggled");
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
            TrackMessage::Translated(translation) => {
                self.grid.translation = translation;

                self.notes_cache.clear();
                self.grid_cache.clear();
                self.selected_notes_cache.clear();
            }

            TrackMessage::Scaled(scaling, translation) => {
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

            TrackMessage::UpdateSelection { change_selection } => {
                match change_selection {
                    ChangeSelection::DrainSelect => {
                        let new_indices = self.selected.notes.drain(&mut self.midi_notes);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::DeselectAllNotes { new_indices },
                        );
                    }
                    ChangeSelection::SelectAll => {
                        let new_indices = self.midi_notes.drain(&mut self.selected.notes);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::SelectAllNotes { new_indices },
                        );
                    }
                    ChangeSelection::AddOneToSelected { note_index } => {
                        let note = self.midi_notes.remove(note_index);
                        let new_index = self.selected.notes.add(note);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::SelectNote { note_index, new_index },
                        );
                    }
                    ChangeSelection::UnselectOne { note_index } => {
                        let note = self.selected.notes.remove(note_index);
                        let new_index = self.midi_notes.add(note);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::DeselectNote { note_index, new_index },
                        );
                    }
                    ChangeSelection::UnselectAllButOne { note_index } => {
                        let selected_note = self.selected.notes.remove(note_index);
                        let new_indices = self.selected.notes.drain(&mut self.midi_notes);
                        self.selected.notes.add(selected_note);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::DeselectAllButOne { note_index, new_indices },
                        );
                    }
                    ChangeSelection::SelectOne { note_index } => {
                        let selected_note = self.midi_notes.remove(note_index);
                        let new_indices = self.selected.notes.drain(&mut self.midi_notes);
                        self.selected.notes.add(selected_note);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::SelectOne { note_index, new_indices },
                        );
                    }

                    ChangeSelection::SelectMany { note_indices } => {
                        let removed_notes = self.midi_notes.remove_notes(note_indices.clone());
                        let new_indices = self.selected.notes.add_midi_notes(removed_notes);

                        history.add_selection(
                            self.track_id,
                            SelectionAction::SelectManyNotes { note_indices, new_indices },
                        );
                    }
                };
                self.selected.selecting_square = None;
                self.selected.direct_selecting_square = None;
                self.selected_notes_cache.clear();
                self.notes_cache.clear();
                self.selection_square_cache.clear();
            }

            // TrackMessage::Dragged { delta_pitch, delta_time, mut original_notes } => {
            TrackMessage::Dragged { cursor_delta, original_notes } => {
                // println!("");
                // let original_notes = original_notes.clone();
                // let mut modified_notes: MidiNotes = original_notes.clone();
                // modified_notes.modify_all_notes(|mut note| {
                //     let chromatic_starting_pitch = note.pitch.get();

                //     let scale_starting_pitch = self
                //         .grid
                //         .scale
                //         .from_chromatic_index_to_scale_index(chromatic_starting_pitch)
                //         as i8;

                //     let mut new_pitch_index =
                //         (scale_starting_pitch + cursor_delta.y as i8) as usize;

                //     // if the new_pitch_index is out of bounds, cancel the move
                //     if new_pitch_index >= self.grid.scale.midi_size() {
                //         new_pitch_index = scale_starting_pitch as usize;
                //     }

                //     let new_pitch = self.grid.scale.midi_range[new_pitch_index] as i16;

                //     let delta_time = cursor_delta.x;
                //     note.pitch = Pitch(new_pitch);

                //     note.start = note.start + delta_time;
                //     note.end = note.end + delta_time;
                // });

                // println!("original_notes: {:?}", original_notes);

                let mut modified_notes: MidiNotes = original_notes.clone();
                modified_notes.drag_all_notes(cursor_delta, &self.grid);

                self.selected.notes.clear();
                self.selected.notes.add_midi_notes(modified_notes.clone());
                self.selected_notes_cache.clear();
                self.notes_cache.clear();

                history.add_track_action(self.track_id, TrackAction::DraggedNotes { cursor_delta });
            }

            TrackMessage::ResizedNotes { mut original_notes, delta_time, resize_end } => {
                original_notes.resize_all_notes(resize_end, delta_time);

                self.selected.notes = original_notes;
                self.selected_notes_cache.clear();

                history.add_track_action(
                    self.track_id,
                    TrackAction::ResizedNotes { delta_time, resize_end },
                );
            }

            TrackMessage::Selecting { selecting_square, direct_selecting_square } => {
                self.selected.selecting_square = Some(selecting_square);
                self.selected.direct_selecting_square = Some(direct_selecting_square);
                self.selection_square_cache.clear();
            }

            TrackMessage::DeleteMidiNotes { notes_to_delete } => {
                let notes_before_deletion = self.midi_notes.remove_notes(notes_to_delete.clone());
                self.notes_cache.clear();

                history.add_track_action(
                    self.track_id,
                    TrackAction::RemoveManyNotes {
                        note_indices_before: notes_to_delete,
                        notes_before_deletion,
                    },
                );
            }

            TrackMessage::AddNote(note_to_add) => {
                let note_index_after = self.midi_notes.add(note_to_add.clone());
                self.notes_cache.clear();

                history.add_track_action(
                    self.track_id,
                    TrackAction::AddNote { note_to_add, note_index_after },
                );
            }

            TrackMessage::AddManyNotes(notes_to_add) => {
                let note_indices_after = self.midi_notes.add_midi_notes(notes_to_add.clone());
                self.notes_cache.clear();

                history.add_track_action(
                    self.track_id,
                    TrackAction::AddManyNotes { notes_to_add, note_indices_after },
                );
            }

            TrackMessage::Undo(track_action) => {
                let mut selection_undo = true;

                match track_action {
                    TrackAction::AddNote { note_index_after, .. } => {
                        self.midi_notes.remove(note_index_after);
                    }
                    TrackAction::AddManyNotes { note_indices_after, .. } => {
                        self.midi_notes.remove_notes(note_indices_after);
                    }
                    TrackAction::RemoveNote { note_before, .. } => {
                        self.midi_notes.add(note_before);
                    }
                    TrackAction::RemoveManyNotes { notes_before_deletion, .. } => {
                        self.midi_notes.add_midi_notes(notes_before_deletion);
                    }
                    TrackAction::DraggedNotes { cursor_delta, .. } => {
                        // self.midi_notes.remove(note_index_after);
                    }
                    TrackAction::ResizedNotes { delta_time, resize_end } => {
                        // self.selected.notes.remove_notes(note_indices_after);
                    } // _ => {}
                    TrackAction::SelectionAction(selection_action) => {
                        match selection_action {
                            SelectionAction::SelectNote { note_index, new_index } => {}
                            SelectionAction::DeselectNote { note_index, new_index } => {}
                            SelectionAction::SelectManyNotes { note_indices, new_indices } => {}
                            SelectionAction::DeselectManyNotes { note_indices, new_indices } => {}
                            SelectionAction::SelectAllNotes { new_indices } => {}
                            SelectionAction::DeselectAllNotes { new_indices } => {}
                            SelectionAction::DeselectAllButOne { note_index, new_indices } => {}
                            SelectionAction::SelectOne { note_index, new_indices } => {}
                        }
                        selection_undo = false;
                    }
                }
            }
            TrackMessage::Redo(action) => {
                // for action in action_group.actions {
                //     if let Action::TrackAction(track_action) = action {
                //         match track_action {
                //             TrackAction::AddNote { note, .. } => {
                //                 self.midi_notes.add(note);
                //             }
                //             TrackAction::AddManyNotes { notes, .. } => {
                //                 self.midi_notes.add_midi_notes(notes);
                //             }
                //             TrackAction::RemoveNote { note_index_before, .. } => {
                //                 self.midi_notes.remove(note_index_before);
                //             }
                //             TrackAction::RemoveManyNotes { note_indices_before, .. } => {
                //                 self.midi_notes.remove_notes(note_indices_before);
                //             }
                //             TrackAction::ChangeNote { note_after, note_index_before, .. } => {
                //                 self.midi_notes.remove(note_index_before);
                //                 self.midi_notes.add(note_after);
                //             }
                //             TrackAction::ChangeManyNotes {
                //                 notes_after,
                //                 note_indices_before,
                //                 ..
                //             } => {
                //                 self.midi_notes.remove_notes(note_indices_before);
                //                 self.midi_notes.add_midi_notes(notes_after);
                //             }
                //         }
                //     }
                // }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum TrackMessage {
    Translated(Vector),
    Scaled(Vector, Option<Vector>),
    AddNote(MidiNote),
    AddManyNotes(MidiNotes),
    CursorMoved,
    DeleteMidiNotes { notes_to_delete: Vec<NoteIndex> },
    UpdateSelection { change_selection: ChangeSelection },
    Dragged { cursor_delta: Vector, original_notes: MidiNotes },
    // FinishDragging,
    ResizedNotes { delta_time: f32, original_notes: MidiNotes, resize_end: NoteEdge },
    // FinishResizingNote,
    Selecting { selecting_square: Rectangle, direct_selecting_square: Rectangle },
    // FinishSelecting {
    //     selecting_square: Rectangle,
    //     // keep_already_selected: bool,
    // },
    ModifiersChanged(Modifiers),
    Toggle,
    Undo(TrackAction),
    Redo(TrackAction),
}

#[derive(Default)]
pub struct TrackState {
    pub grid_interaction: GridInteraction,
    pub note_interaction: NoteInteraction,
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
        let cursor_in_bounds: bool = cursor.is_over(&bounds);

        let cursor_position = if let Some(pos) = cursor.position_from(bounds.position()) {
            pos
        } else {
            return (event::Status::Ignored, None);
        };

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

        // let region = self.grid.visible_region(bounds.size());
        // TODO: uncomment
        let projected_cursor = self.grid.to_track_axes(cursor_position, &bounds.size());
        // BUG:  when cursor is out of bounds
        let music_scale_cursor = self.grid.adjust_to_music_scale(projected_cursor);

        match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                if modifiers.shift() {
                    track_state.grid_interaction = GridInteraction::Panning {
                        translation: self.grid.translation,
                        start: cursor_position,
                        // initial_cursor_pos: cursor_position,
                        // initial_grid_offset: self.grid.offset,
                    };
                } else {
                    track_state.grid_interaction = GridInteraction::None;
                }

                (event::Status::Captured, Some(TrackMessage::ModifiersChanged(modifiers)))
            }

            Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. })
                if key_code == keyboard::KeyCode::B =>
            {
                track_state.note_interaction.toggle_write_mode();

                // track_state.note_interaction = NoteInteraction::WriteNoteMode;
                (event::Status::Captured, None)
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if let GridInteraction::Panning { .. } = track_state.grid_interaction {
                    track_state.grid_interaction = GridInteraction::None;
                    return (event::Status::Captured, None);
                }

                (event::Status::Ignored, None)
            }

            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                //
                match track_state.note_interaction {
                    //
                    NoteInteraction::Selecting { initial_music_cursor, .. } => {
                        let delta_cursor_pos = music_scale_cursor - initial_music_cursor;
                        let size = Size::new(delta_cursor_pos.x, delta_cursor_pos.y);

                        let selecting_square = Rectangle::new(initial_music_cursor, size);
                        track_state.note_interaction = NoteInteraction::None;

                        let note_indices = self.midi_notes.get_notes_in_rect(selecting_square);

                        return (
                            event::Status::Captured,
                            Some(TrackMessage::UpdateSelection {
                                change_selection: ChangeSelection::SelectMany { note_indices },
                            }),
                        );
                    }
                    NoteInteraction::Dragging { .. } => {
                        track_state.note_interaction = NoteInteraction::None;
                        // return (event::Status::Captured, Some(TrackMessage::FinishDragging));
                    }
                    NoteInteraction::Resizing { .. } => {
                        track_state.note_interaction = NoteInteraction::None;
                        // return (event::Status::Captured, Some(TrackMessage::FinishResizingNote));
                    }
                    NoteInteraction::WriteNoteMode { .. } => {
                        track_state.note_interaction = NoteInteraction::WriteNoteMode(false);
                    }

                    _ => {
                        track_state.note_interaction = NoteInteraction::None;
                    }
                }
                //

                (event::Status::Ignored, None)
            }

            // left button
            //
            // TODO
            //
            //
            //
            // change the resize note rectangle
            //
            //
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                //

                // draw a note if in pen mode and not over a note
                if track_state.note_interaction.is_write_mode() {
                    track_state.note_interaction = NoteInteraction::WriteNoteMode(true);
                    // if let Some(OverNote { .. }) =
                    //     self.midi_notes.get_note_under_cursor(&self.grid, music_scale_cursor)
                    // {
                    //     return (event::Status::Ignored, None);
                    // }

                    // let pitch = Pitch(music_scale_cursor.y.floor() as i16);
                    // let start = music_scale_cursor.x.floor();
                    // let end = start + 1.0;

                    // let note = MidiNote::new(start, end, pitch);

                    // return (event::Status::Captured, Some(TrackMessage::AddNote(note)));
                    return (event::Status::Captured, None);
                }

                // Check if a Selected note has been clicked
                if let Some(OverNote { note_index: _, note_edge }) =
                    self.selected.notes.get_note_under_cursor(&self.grid, music_scale_cursor)
                {
                    let new_selection = self.selected.notes.clone();
                    let message = (event::Status::Captured, None);

                    track_state.drag_or_resize(note_edge, projected_cursor, new_selection);

                    return message;
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
                        let new_note_index = new_selected.add(note);

                        (
                            event::Status::Captured,
                            Some(TrackMessage::UpdateSelection {
                                change_selection: ChangeSelection::AddOneToSelected {
                                    note_index: new_note_index,
                                },
                            }),
                        )
                    } else {
                        // if the control key is not pressed, clear the Selected notes and
                        // select the clicked note
                        new_selected.clear();
                        new_selected.add(note);

                        (
                            event::Status::Captured,
                            Some(TrackMessage::UpdateSelection {
                                change_selection: ChangeSelection::SelectOne { note_index },
                            }),
                        )
                    };

                    track_state.drag_or_resize(note_edge, projected_cursor, new_selected);

                    return message;
                }

                // if no note has been clicked, start selecting
                track_state.note_interaction = NoteInteraction::Selecting {
                    initial_music_cursor: music_scale_cursor,
                    initial_cursor_proj: projected_cursor,
                };

                // if the control key is not pressed, clear the Selected notes
                if !self.modifiers.control() {
                    return (
                        event::Status::Captured,
                        Some(TrackMessage::UpdateSelection {
                            change_selection: ChangeSelection::DrainSelect,
                        }),
                    );
                }

                (event::Status::Captured, None)
            }

            //
            //
            // Right button
            //
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                //
                track_state.grid_interaction = GridInteraction::Panning {
                    translation: self.grid.translation,
                    start: cursor_position,
                };

                (event::Status::Captured, None)
            }

            //
            // moving cursor
            //
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // Panning
                //
                //
                if let GridInteraction::Panning { translation, start } =
                    track_state.grid_interaction
                {
                    let mut new_translation = Vector::new(
                        translation.x + (cursor_position.x - start.x) / self.grid.scaling.x,
                        translation.y + (cursor_position.y - start.y) / self.grid.scaling.y,
                    );

                    self.grid.limit_to_bounds(&mut new_translation, bounds, self.grid.scaling);

                    return (
                        event::Status::Captured,
                        Some(TrackMessage::Translated(new_translation)),
                    );
                };

                match &track_state.note_interaction {
                    inter @ NoteInteraction::Resizing { .. } => {
                        return inter.handle_resizing(music_scale_cursor);
                    }
                    inter @ NoteInteraction::Selecting { .. } => {
                        return inter.handle_selecting(projected_cursor, music_scale_cursor);
                    }
                    inter @ NoteInteraction::Dragging { .. } => {
                        return inter.handle_dragging(projected_cursor, !self.modifiers.alt());
                    }
                    inter @ NoteInteraction::WriteNoteMode(_) => {
                        return inter.handle_note_writing(music_scale_cursor, &self);
                    }
                    _ => {}
                };
                // Note resizing
                // if let NoteInteraction::Resizing {
                //     initial_cursor_pos,
                //     original_notes,
                //     resize_end,
                // } = &track_state.note_interaction
                // {
                //     let cursor_delta = music_scale_cursor - *initial_cursor_pos;

                //     message = Some(TrackMessage::ResizedNotes {
                //         delta_time: cursor_delta.x,
                //         original_notes: original_notes.clone(),
                //         resize_end: resize_end.clone(),
                //     });
                //     event_status = event::Status::Captured;

                //     return (event_status, message);
                // }

                // // Selecting
                // if let NoteInteraction::Selecting { initial_music_cursor, initial_cursor_proj } =
                //     &track_state.note_interaction
                // {
                //     let cursor_delta = music_scale_cursor - *initial_music_cursor;

                //     let selecting_square = Rectangle::new(
                //         *initial_music_cursor,
                //         Size::new(cursor_delta.x, cursor_delta.y),
                //     );

                //     let direct_cursor_delta = projected_cursor - *initial_cursor_proj;
                //     let direct_selecting_square = Rectangle::new(
                //         *initial_cursor_proj,
                //         Size::new(direct_cursor_delta.x, direct_cursor_delta.y),
                //     );

                //     message =
                //         Some(TrackMessage::Selecting { selecting_square, direct_selecting_square });
                //     event_status = event::Status::Captured;

                //     return (event_status, message);
                // }

                // // Note Dragging
                // //
                // //
                // if let NoteInteraction::Dragging { initial_cursor_pos, original_notes } =
                //     &track_state.note_interaction
                // {
                //     // snap to pitch
                //     // let mut music_floor_cursor =
                //     //     Vector::new(music_scale_cursor.x, music_scale_cursor.y.floor());
                //     // let mut music_floor_initial_cursor =
                //     //     Vector::new(initial_cursor_pos.x, initial_cursor_pos.y.floor());

                //     // let mut music_cursor_delta: Vector =
                //     //     (music_scale_cursor - *initial_cursor_pos).into();

                //     // always snap to pitch
                //     let mut floor_cursor =
                //         Vector::new(projected_cursor.x, projected_cursor.y.floor());
                //     let mut floor_initial_cursor =
                //         Vector::new(initial_cursor_pos.x, initial_cursor_pos.y.floor());

                //     // println!("initial_cursor_pos: {:?}", initial_cursor_pos);
                //     // println!("projected_cursor: {:?}", projected_cursor);

                //     let mut cursor_delta: Vector = (projected_cursor - *initial_cursor_pos).into();

                //     // snap to beat
                //     if !self.modifiers.alt() {
                //         // music_floor_cursor.x = music_floor_cursor.x.floor();
                //         // music_floor_initial_cursor.x = music_floor_initial_cursor.x.floor();
                //         // music_cursor_delta =
                //         //     (music_floor_cursor - music_floor_initial_cursor).into();

                //         floor_cursor.x = floor_cursor.x.floor();
                //         floor_initial_cursor.x = floor_initial_cursor.x.floor();
                //         cursor_delta = (floor_cursor - floor_initial_cursor).into();
                //     }

                //     message = Some(TrackMessage::Dragged {
                //         cursor_delta: cursor_delta,
                //         original_notes: original_notes.clone(),
                //     });

                //     event_status = event::Status::Captured;

                //     return (event_status, message);
                // };

                if let NoteInteraction::WriteNoteMode(do_write) = track_state.note_interaction {
                    return (event::Status::Ignored, None);

                    // no mouse interaction yet
                } else {
                    let mut over_note =
                        self.selected.notes.get_note_under_cursor(&self.grid, music_scale_cursor);

                    if let None = over_note {
                        over_note =
                            self.midi_notes.get_note_under_cursor(&self.grid, music_scale_cursor);
                    }

                    // check if the mouse is over a note or the edge of a note
                    match over_note {
                        //
                        Some(OverNote { note_index: _, note_edge: NoteEdge::Start })
                        | Some(OverNote { note_index: _, note_edge: NoteEdge::End }) => {
                            track_state.note_interaction = NoteInteraction::ResizingHover;
                        }

                        _ => {
                            track_state.note_interaction = NoteInteraction::None;
                        }
                    };
                    return (event::Status::Ignored, None);
                }

                // (event_status, message)
            }

            Event::Mouse(mouse::Event::WheelScrolled { delta }) => match delta {
                mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                    if !cursor_in_bounds {
                        return (event::Status::Ignored, None);
                    }

                    if y < 0.0 && self.grid.scaling.x <= MIN_SCALING.x
                        || y > 0.0 && self.grid.scaling.x >= MAX_SCALING.x
                        || y < 0.0 && self.grid.scaling.y <= MIN_SCALING.y
                        || y > 0.0 && self.grid.scaling.y >= MAX_SCALING.y
                    {
                        return (event::Status::Captured, None);
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

                    (event::Status::Captured, Some(TrackMessage::Scaled(scaling, translation)))
                }
            },
            //     _ => (event::Status::Ignored, None),
            // },
            _ => (event::Status::Ignored, None),
        }
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

        let yellow = Color::from_rgb(1.0, 1.0, 0.0);
        let dark_yellow = Color::from_rgb(0.5, 0.5, 0.0);

        let notes_overlay =
            self.midi_notes.draw_notes(&self.grid, &bounds, &cursor, &self.notes_cache, yellow);

        let selected_notes_elements = self.selected.notes.draw_notes(
            &self.grid,
            &bounds,
            &cursor,
            &self.selected_notes_cache,
            dark_yellow,
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
            NoteInteraction::WriteNoteMode(_) => mouse::Interaction::Crosshair,

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
