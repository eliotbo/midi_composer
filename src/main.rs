//! Midi track
//!
use iced::executor;
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
use iced::{Application, Command, Length, Settings};
pub use iced_native;

pub mod piano_theme;
pub use piano_theme::PianoTheme;

pub mod track;
use track::{TimingInfo, Track, TrackMessage};

pub mod grid;
pub use grid::Grid;

mod midi_notes;
// use midi_notes::MidiNote;

mod config;
mod scale;

use crate::config::INIT_GRID_SIZE;

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
    _history: Vec<Action>,
    tracks: Vec<Track>,
    _timein_info: TimingInfo,
    _selection: Selected,
}

impl Default for MidiEditor {
    fn default() -> Self {
        Self {
            _history: Vec::new(),
            tracks: vec![Track::default(), Track::default()],
            _timein_info: TimingInfo::default(),
            _selection: Selected { _track_number: 0, _note_number: 0 },
        }
    }
}

#[derive(Debug, Clone)]
struct Action;

#[derive(Debug, Copy, Clone)]
struct Selected {
    _track_number: u16,
    _note_number: u32,
}

#[derive(Debug, Clone)]
enum EditorMessage {
    Track(usize, TrackMessage),
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
            EditorMessage::Track(track_number, message) => {
                self.tracks[track_number].update(message);
            }
        }

        Command::none()
    }

    fn view(&self) -> EditorElement {
        //         let track: &Track = &self.tracks[0];
        // let canvas: track::TrackElement = track.view();
        // let editor_canvas: EditorElement = canvas.map(move |message| EditorMessage::Track(message));

        // let mut content = column![];

        // for (track_num, track) in self.tracks.iter().enumerate() {
        //     // let track: &Track = &self.tracks[0];
        //     let canvas: track::TrackElement = (&track).view();
        //     let editor_canvas: EditorElement =
        //         canvas.map(move |message| EditorMessage::Track(message, track_num));
        //     content.push(editor_canvas);
        // }

        let button = |label| {
            button(text(label).horizontal_alignment(alignment::Horizontal::Center))
                .padding(10)
                .width(Length::Units(80))
        };

        let toggle_button =
            button("Toggle").on_press(EditorMessage::Track(1, TrackMessage::Toggle));

        let mut elements: Vec<EditorElement> = self
            .tracks
            .iter()
            .enumerate()
            .map(|(track_number, track)| {
                let canvas: track::TrackElement = (&track).view();
                let editor_canvas: EditorElement =
                    canvas.map(move |message| EditorMessage::Track(track_number, message));
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
