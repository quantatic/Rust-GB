use sdl2::audio;

use crate::CLOCK_FREQUENCY;

use super::{
    WaveDuty, EIGHTH_WAVE_DUTY_WAVEFORM, FOURTH_WAVE_DUTY_WAVEFORM, HALF_WAVE_DUTY_WAVEFORM,
    THREE_QUARTERS_WAVE_DUTY_WAVEFORM,
};

const SEQUENCER_CLOCK_FREQUENCY: u64 = 512;

const SEQUENCER_CLOCK_PERIOD: u64 = CLOCK_FREQUENCY / SEQUENCER_CLOCK_FREQUENCY;

const LENGTH_COUNTER_CLOCKS: [bool; 8] = [false, false, true, false, true, false, true, false];

enum OutputLevel {
    Mute,
    Full,
    Half,
    Quarter,
}

#[derive(Clone, Debug, Default)]
pub struct Channel3 {
    sound_on_off: u8,
    sound_length: u8,
    length_counter: u16,
    output_level: u8,
    frequency_low: u8,
    frequency_high: u8,
    clock: u64,

    wave_timer_ticks_left: u16,
    wave_index: usize,
    frame_sequencer_idx: usize,
    wave_table: [u8; 16],

    enabled: bool,
}

impl Channel3 {
    pub fn step(&mut self) {
        if self.clock % SEQUENCER_CLOCK_PERIOD == 0 {
            if self.stop_when_length_expires() && LENGTH_COUNTER_CLOCKS[self.frame_sequencer_idx] {
                self.length_counter = self.length_counter.saturating_sub(1);
                if self.length_counter == 0 {
                    self.set_enabled(false);
                }
            }

            self.frame_sequencer_idx = (self.frame_sequencer_idx + 1) % 8;
        }

        self.wave_timer_ticks_left = self.wave_timer_ticks_left.saturating_sub(1);
        if self.wave_timer_ticks_left == 0 {
            self.wave_timer_ticks_left = (2048 - self.get_channel_frequency()) * 2;
            self.wave_index = (self.wave_index + 1) % 32;
        }

        if !self.get_sound_playback() {
            self.set_enabled(false);
        }

        self.clock += 1;
    }

    pub fn sample(&self) -> u8 {
        let wave_table_entry = match self.wave_index {
            idx @ 0..=15 => self.wave_table[idx] >> 4,
            idx @ 16..=31 => self.wave_table[idx - 16] & 0b1111,
            _ => unreachable!(),
        };

        let sample = match self.get_output_level() {
            OutputLevel::Mute => 0,
            OutputLevel::Quarter => wave_table_entry >> 2,
            OutputLevel::Half => wave_table_entry >> 1,
            OutputLevel::Full => wave_table_entry,
        };

        if self.get_sound_playback() && self.get_enabled() {
            sample
        } else {
            0
        }
    }
}

impl Channel3 {
    pub fn read_sound_on_off(&self) -> u8 {
        self.sound_on_off
    }

    pub fn write_sound_on_off(&mut self, value: u8) {
        self.sound_on_off = value;
    }

    pub fn read_sound_length(&self) -> u8 {
        self.sound_length
    }

    pub fn write_sound_length(&mut self, value: u8) {
        self.sound_length = value;
        self.length_counter = 256 - u16::from(value);
    }

    pub fn read_output_level(&self) -> u8 {
        self.output_level
    }

    pub fn write_output_level(&mut self, value: u8) {
        self.output_level = value
    }

    pub fn read_frequency_low(&self) -> u8 {
        self.frequency_low
    }

    pub fn write_frequency_low(&mut self, value: u8) {
        self.frequency_low = value;
    }

    pub fn read_frequency_high(&self) -> u8 {
        self.frequency_high
    }

    pub fn write_frequency_high(&mut self, value: u8) {
        const FREQUENCY_HIGH_ENABLED_MASK: u8 = 1 << 7;

        if (value & FREQUENCY_HIGH_ENABLED_MASK) == FREQUENCY_HIGH_ENABLED_MASK {
            self.set_enabled(true);
        }
        self.frequency_high = value;
    }

    pub fn read_wave_pattern_ram(&self, offset: u16) -> u8 {
        self.wave_table[usize::from(offset)]
    }

    pub fn write_wave_pattern_ram(&mut self, value: u8, offset: u16) {
        self.wave_table[usize::from(offset)] = value;
    }
}

impl Channel3 {
    fn get_sound_playback(&self) -> bool {
        const SOUND_ON_OFF_PLAYBACK_MASK: u8 = 1 << 7;

        (self.sound_on_off & SOUND_ON_OFF_PLAYBACK_MASK) == SOUND_ON_OFF_PLAYBACK_MASK
    }

    fn get_output_level(&self) -> OutputLevel {
        const OUTPUT_LEVEL_SHIFT: usize = 5;
        const OUTPUT_LEVEL_MASK: u8 = 0b11 << OUTPUT_LEVEL_SHIFT;
        const OUTPUT_LEVEL_MUTE_MASK: u8 = 0b00 << OUTPUT_LEVEL_SHIFT;
        const OUTPUT_LEVEL_FULL_MASK: u8 = 0b01 << OUTPUT_LEVEL_SHIFT;
        const OUTPUT_LEVEL_HALF_MASK: u8 = 0b10 << OUTPUT_LEVEL_SHIFT;
        const OUTPUT_LEVEL_QUARTER_MASK: u8 = 0b11 << OUTPUT_LEVEL_SHIFT;

        match self.output_level & OUTPUT_LEVEL_MASK {
            OUTPUT_LEVEL_MUTE_MASK => OutputLevel::Mute,
            OUTPUT_LEVEL_QUARTER_MASK => OutputLevel::Quarter,
            OUTPUT_LEVEL_HALF_MASK => OutputLevel::Half,
            OUTPUT_LEVEL_FULL_MASK => OutputLevel::Full,
            _ => unreachable!(),
        }
    }

    const CHANNEL_FREQUENCY_HIGH_MASK: u8 = 0b111;

    fn get_channel_frequency(&self) -> u16 {
        let channel_frequency_low = self.frequency_low;
        let channel_frequency_high = self.frequency_high & Self::CHANNEL_FREQUENCY_HIGH_MASK;
        let channel_frequency = u16::from_be_bytes([channel_frequency_high, channel_frequency_low]);
        channel_frequency
    }

    fn stop_when_length_expires(&self) -> bool {
        const FREQUENCY_HIGH_STOP_WHEN_LENGTH_EXPIRES_MASK: u8 = 0b0100_0000;
        (self.frequency_high & FREQUENCY_HIGH_STOP_WHEN_LENGTH_EXPIRES_MASK)
            == FREQUENCY_HIGH_STOP_WHEN_LENGTH_EXPIRES_MASK
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        if value {
            if self.length_counter == 0 {
                self.length_counter = 256;
            }

            self.wave_timer_ticks_left = (2048 - self.get_channel_frequency()) * 2;
            self.wave_index = 0;

            self.enabled = true;
        } else {
            self.enabled = false;
        }
    }
}
