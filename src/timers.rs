use crate::waves::Audio;

pub trait TimerActions {
    fn set(&mut self, value: u8);
    fn get(&self) -> u8;
    fn decrement(&mut self);
}

impl TimerActions for SoundTimer {
    fn decrement(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
        self.action();
    }

    fn set(&mut self, value: u8) {
        self.value = value;
    }

    fn get(&self) -> u8 {
        self.value
    }
}

impl TimerActions for DelayTimer {
    fn decrement(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }

    fn set(&mut self, value: u8) {
        self.value = value;
    }

    fn get(&self) -> u8 {
        self.value
    }
}

pub struct SoundTimer {
    pub(crate) value: u8,
    audio: Audio,
    sounding: bool,
}
impl SoundTimer {
    pub fn new(audio: Audio) -> SoundTimer {
        SoundTimer {
            value: 0,
            audio: audio,
            sounding: false,
        }
    }
    fn action(&mut self) {
        if self.value > 0 && !self.sounding {
            self.sounding = true;
            self.audio.sound();
        } else if self.value <= 0 && self.sounding {
            self.sounding = false;
            self.audio.pause();
        }
    }
}
pub struct DelayTimer {
    pub(crate) value: u8,
}
