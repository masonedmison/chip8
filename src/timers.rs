use crate::waves::Audio;

pub trait TimerActions {
    fn set(&mut self, value: u8);
    fn decrement(&mut self);
}

impl TimerActions for SoundTimer {
    fn decrement(&mut self) {
        self.value -= 1;
        self.action();
    }

    fn set(&mut self, value: u8) {
        self.value = value;
    }
}

impl TimerActions for DelayTimer {
    fn decrement(&mut self) {
        if self.value > 1 {
            self.value -= 1;
        }
    }

    fn set(&mut self, value: u8) {
        self.value = value;
    }
}

pub struct SoundTimer {
    pub(crate) value: u8,
    audio: Audio,
    sounding: bool,
}
impl SoundTimer {
    fn action(&self) {
        if self.value > 0 && !self.sounding {
            self.audio.sound();
        } else if self.value <= 0 && self.sounding {
            self.audio.pause();
        }
    }
}
pub struct DelayTimer {
    pub(crate) value: u8,
}

pub struct Timers {
    pub sound: SoundTimer,
    pub delay: DelayTimer,
}
impl Timers {
    pub fn new(audio: Audio) -> Timers {
        Timers {
            sound: SoundTimer {
                value: 0,
                audio: audio,
                sounding: false,
            },
            delay: DelayTimer { value: 0 },
        }
    }
}
