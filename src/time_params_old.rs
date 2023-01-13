use midly::{MetaMessage, Smf, Timing::Metrical, TrackEventKind};

pub struct TimeParams {
    pub ticks_per_beat: u16,
    pub avg_tempo_in_us_per_beat: u32,
    pub total_beats: f32,
    pub total_time_in_s: f32,
}

impl TimeParams {
    // TODO: make this function more robust.
    //
    // It currently assumes that the timing is Metrical(timing), but it could be
    // given with Timecode(Fps, u8) with the length of a tick is 1/fps/subframe.
    //
    // It currently only reads MetaMessage::Tempo from the first track,
    // but the tempo could be given in other tracks as well
    pub fn get(smf: &Smf) -> TimeParams {
        let mut ticks_per_beat = 0;
        if let Metrical(timing) = &smf.header.timing {
            ticks_per_beat = timing.as_int();
        }

        let mut avg_tempo_in_us_per_beat = 0;
        let mut num_tempo = 0;

        for event in &smf.tracks[0] {
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                num_tempo += 1;
                avg_tempo_in_us_per_beat += tempo.as_int();
            }
        }
        avg_tempo_in_us_per_beat /= num_tempo;
        let avg_tempo_in_s_per_beat = avg_tempo_in_us_per_beat as f32 / 1000_000.0;

        let mut acc_ticks: u32 = 0;
        for event in &smf.tracks[0] {
            acc_ticks += event.delta.as_int() as u32;
        }

        let total_beats = acc_ticks as f32 / ticks_per_beat as f32;
        let total_time_in_s = avg_tempo_in_s_per_beat * total_beats;

        // use core::ops::Div;
        // println!("avg tempo: {:?} s per beat", avg_tempo_in_s_per_beat);
        // println!("acc_time: {:?} ticks", acc_ticks as f32);
        // println!("total number of beats: {:?}", total_beats);
        // println!(
        //     "total time: {}:{}",
        //     total_time_in_s.div(60.0).floor(),
        //     (total_time_in_s % 60.0).ceil()
        // );

        return TimeParams {
            ticks_per_beat,
            avg_tempo_in_us_per_beat,
            total_beats,
            total_time_in_s,
        };
    }

    pub fn get_bpm(&self) -> f32 {
        return 60.0 / (self.avg_tempo_in_us_per_beat as f32 / 1000_000.0);
    }
}
