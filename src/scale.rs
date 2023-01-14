pub static NOTE_LABELS: [&'static str; 12] =
    ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

#[allow(dead_code)]
pub fn test() {
    let scale = Scale::new(ScaleType::Major, 2);

    println!(
        "base_notes: {:?}",
        scale.midi_range.iter().map(|&x| scale.int_to_label(x)).collect::<Vec<String>>()
    );

    let midi_range_labels =
        scale.midi_range.iter().map(|&x| scale.int_to_label(x)).collect::<Vec<String>>();

    println!("filtered midi notes: {:?}", midi_range_labels);
    println!("cale.midi_range: {:?}", scale.midi_range);
    println!("range: {:?}", scale.get_range(55, 12));
}

#[derive(Clone)]
pub struct Scale {
    pub scale_type: ScaleType,
    pub root: u8,
    pub midi_range: Vec<u8>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ScaleType {
    Major,
    Minor,
    Pentatonic,
    Blues,
    Chromatic,
    Custom(Vec<u8>),
}

impl Default for Scale {
    fn default() -> Self {
        Scale::new(ScaleType::Minor, 2)
    }
}

impl Scale {
    pub fn new(scale_type: ScaleType, root: u8) -> Self {
        let midi_range = Self::make_filtered_midi_range(&scale_type, root);
        Scale { scale_type, root, midi_range }
    }

    pub fn get_base_notes(scale_type: &ScaleType, root: u8) -> Vec<u8> {
        match scale_type {
            ScaleType::Major => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleType::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleType::Pentatonic => vec![0, 2, 4, 7, 9],
            ScaleType::Blues => vec![0, 3, 5, 6, 7, 10],
            ScaleType::Chromatic => vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            ScaleType::Custom(scale) => scale.clone(),
        }
        .iter()
        .map(|&x| (x + root) % 12)
        .collect()
    }

    // from C-2 tp C8
    fn make_filtered_midi_range(scale_type: &ScaleType, root: u8) -> Vec<u8> {
        let base_notes = Self::get_base_notes(scale_type, root);
        let mut filtered_midi_range = Vec::new();
        for i in 0..128 {
            if base_notes.contains(&(i % 12)) {
                filtered_midi_range.push(i);
            }
        }
        filtered_midi_range
    }

    pub fn int_to_label(&self, note: u8) -> String {
        format!("{}{}", NOTE_LABELS[(note % 12) as usize], note as i16 / 12 - 2)
    }

    pub fn get_range(&self, start: u8, size: u8) -> Vec<u8> {
        let mut range = Vec::new();
        for i in start..start + size {
            if self.midi_range.contains(&i) {
                range.push(i);
            }
        }
        range
    }

    pub fn midi_size(&self) -> usize {
        self.midi_range.len()
    }

    pub fn size(&self) -> u8 {
        Self::get_base_notes(&self.scale_type, self.root).len() as u8
    }
}
