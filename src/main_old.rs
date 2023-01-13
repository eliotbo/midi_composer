mod time_params;
mod util;

use midly;
// use midly::MetaMessage;

use midly::{
    num::{u4, u7},
    MidiMessage, TrackEventKind,
};
use time_params::TimeParams;
use util::{InstrumentTrack, Note};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct NotePress {
    pub start_tick: u32,
    pub vel: u7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoteOnHash {
    channel: u4,
    key: u7,
}

pub type NotePresses = HashMap<NoteOnHash, NotePress>;

// impl NotePresses {
//     pub fn get(&self, channel: Channel, key: Key) -> Option<&Vec<NotePress>> {
//         self.get(&(channel, key))
//     }
// }

fn main() {
    let data = std::fs::read("you_live_and_you_learn.mid").unwrap();
    // let data = std::fs::read("You Get What's Coming, Flute, 8.5_10, - Orchestrated.mid").unwrap();
    let smf = midly::Smf::parse(&data).unwrap();

    let time_params = TimeParams::get(&smf);
    println!("song length: {:.2} seconds", time_params.total_time_in_s);
    println!("bpm: {:.2}", time_params.get_bpm());

    let mut notes: Vec<Note> = Vec::new();
    let mut note_ons: NotePresses = HashMap::new();
    let mut ticks_since_start = 0;
    for event in &smf.tracks[1] {
        //
        ticks_since_start += event.delta.as_int() as u32;
        if let TrackEventKind::Midi { channel, message } = event.kind {
            //
            if let MidiMessage::NoteOn { key, vel } = message {
                //
                note_ons.insert(
                    NoteOnHash { channel, key },
                    NotePress {
                        start_tick: ticks_since_start,
                        vel,
                    },
                );
            }

            if let MidiMessage::NoteOff { key, vel: _ } = message {
                //
                if let Some(note_press) = note_ons.remove(&NoteOnHash { channel, key }) {
                    //
                    let note = Note {
                        pitch: key.as_int(),
                        start_time: note_press.start_tick,
                        end_time: ticks_since_start,
                        velocity: note_press.vel.as_int(),
                    };
                    notes.push(note);
                    // println!("{:?}", note);
                }
            }

            // println!("{:?} -> {:?}", event.delta.as_int(), message);
            // if let midly::MidiMessage::NoteOn { pitch, velocity } = midi.message {
            //     println!("pitch: {}, velocity: {}", pitch, velocity);
            // }
        }
    }

    // for note in &notes {
    //     println!("{:?}", note);
    // }

    // println!("notes: {:?}", notes);
}
