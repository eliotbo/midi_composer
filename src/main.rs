//! Midi track
//!
use iced::executor;
use iced::keyboard;
use iced::widget::{
    button,
    // column,
    container,
    text,
    // checkbox,
    // horizontal_space, pick_list, row,
    // slider, text,
    Column,
};
use iced::window;

use iced::alignment;
use iced::{Application, Command, Length, Settings, Subscription};
use iced_native::{event, subscription, Event};

pub use iced_native;

pub mod piano_theme;
pub use piano_theme::PianoTheme;

pub mod track;
use track::{TimingInfo, Track, TrackMessage};

pub mod grid;
pub use grid::Grid;

pub mod note;
pub mod util;

// mod midi_notes;
// use midi_notes::MidiNote;

mod config;
// mod scale;

use crate::config::INIT_GRID_SIZE;
use crate::util::{Action, History, TrackId};

use std::collections::HashMap;
// TODO: make my own Vector type that is compatible with element-wise operations
//
//

pub fn main() -> iced::Result {
    // env_logger::builder().format_timestamp(None).init();

    MidiEditor::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct MidiEditor {
    history: History,
    tracks: HashMap<TrackId, Track>,
    track_order: Vec<TrackId>,
    _timein_info: TimingInfo,
    _selection: Selected,
}

impl Default for MidiEditor {
    fn default() -> Self {
        let tracks = vec![0, 1].iter().map(|id| (*id, Track::new(*id))).collect();
        Self {
            history: History::default(),
            tracks, // vec![Track::new(0), Track::new(1)],
            track_order: vec![0, 1],
            _timein_info: TimingInfo::default(),
            _selection: Selected { _track_number: 0, _note_number: 0 },
        }
    }
}

// #[derive(Debug, Clone)]
// struct Action;

#[derive(Debug, Copy, Clone)]
struct Selected {
    _track_number: TrackId,
    _note_number: u32,
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Track(TrackId, TrackMessage),
    EventOccurred(iced_native::Event),
}

impl MidiEditor {
    fn handle_undo(&mut self) -> Command<EditorMessage> {
        if let Some(action) = self.history.undo() {
            match action {
                Action::TrackAction(track_id, track_action) => {
                    Command::perform(async move { track_action }, move |act| {
                        EditorMessage::Track(track_id, TrackMessage::Undo(act))
                    })
                }
                Action::None => Command::none(),
            }
        } else {
            Command::none()
        }
    }

    fn handle_redo(&mut self) -> Command<EditorMessage> {
        if let Some(action) = self.history.redo() {
            match action {
                Action::TrackAction(track_id, track_action) => {
                    Command::perform(async move { track_action }, move |act| {
                        EditorMessage::Track(track_id, TrackMessage::Redo(act))
                    })
                }
                Action::None => Command::none(),
            }
        } else {
            Command::none()
        }
    }
}

type EditorElement<'a> = iced::Element<'a, EditorMessage, iced::Renderer<PianoTheme>>;

impl Application for MidiEditor {
    type Message = EditorMessage;
    type Theme = PianoTheme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<EditorMessage>) {
        (Self { ..Self::default() }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Background of midi editor - Iced")
    }

    fn update(&mut self, message: EditorMessage) -> Command<EditorMessage> {
        match message {
            EditorMessage::Track(track_id, message) => {
                if let Some(track) = self.tracks.get_mut(&track_id) {
                    track.update(message, &mut self.history);
                } else {
                    println!("Called non-existent track id: {}", track_id);
                }
                Command::none()
            }
            EditorMessage::EventOccurred(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed { modifiers, key_code })
                    if modifiers.command() && key_code == keyboard::KeyCode::Z =>
                {
                    self.handle_undo()
                }
                // redo
                Event::Keyboard(keyboard::Event::KeyPressed { modifiers, key_code })
                    if modifiers.command() && key_code == keyboard::KeyCode::Y =>
                {
                    self.handle_redo()
                }
                // Event::Keyboard(keyboard::Event::KeyPressed { modifiers, key_code })
                //     if key_code == keyboard::KeyCode::B =>
                // {
                //     for mut track in self.tracks {

                //     }
                // }
                _ => Command::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<EditorMessage> {
        iced_native::subscription::events().map(EditorMessage::EventOccurred)
    }

    fn view(&self) -> EditorElement {
        let button = |label| {
            button(text(label).horizontal_alignment(alignment::Horizontal::Center))
                .padding(10)
                .width(Length::Units(80))
        };

        let toggle_button =
            button("Toggle").on_press(EditorMessage::Track(1, TrackMessage::Toggle));

        let mut elements: Vec<EditorElement> = self
            .track_order
            .iter()
            .map(|track_id| {
                let track = &self.tracks[track_id];
                let canvas: track::TrackElement = (&track).view();
                let editor_canvas: EditorElement =
                    canvas.map(move |message| EditorMessage::Track(*track_id, message));
                editor_canvas
            })
            .collect();

        elements.push(toggle_button.into());

        let content = Column::with_children(elements).spacing(15);

        let tainer: EditorElement = container(content)
            .width(Length::Units(INIT_GRID_SIZE.width as u16))
            .height(Length::Units(INIT_GRID_SIZE.height as u16 * 2))
            .padding(iced::Padding::from(4))
            .center_x()
            .align_x(alignment::Horizontal::Right)
            .align_y(alignment::Vertical::Center)
            .style(piano_theme::Container::Box)
            .into();

        container(tainer)
            .width(Length::Fill)
            .height(Length::Fill)
            // .padding(iced::Padding::from(20))
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> PianoTheme {
        PianoTheme::NORMAL
    }
}
