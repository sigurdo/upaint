use keystrokes_parsing::KeystrokeSequence;

#[derive(Clone, Debug, PartialEq)]
pub struct Macro {
    pub keystrokes: KeystrokeSequence,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroRecording {
    pub keystrokes: KeystrokeSequence,
    pub slot: char,
}

impl MacroRecording {
    pub fn new(slot: char) -> Self {
        Self {
            keystrokes: KeystrokeSequence::new(),
            slot,
        }
    }
}
