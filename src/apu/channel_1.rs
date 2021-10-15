use sdl2::audio;

use crate::CLOCK_FREQUENCY;

use super::{
    WaveDuty, EIGHTH_WAVE_DUTY_WAVEFORM, FOURTH_WAVE_DUTY_WAVEFORM, HALF_WAVE_DUTY_WAVEFORM,
    THREE_QUARTERS_WAVE_DUTY_WAVEFORM,
};

const SEQUENCER_CLOCK_FREQUENCY: u64 = 512;

const SEQUENCER_CLOCK_PERIOD: u64 = CLOCK_FREQUENCY / SEQUENCER_CLOCK_FREQUENCY;

const LENGTH_COUNTER_CLOCKS: [bool; 8] = [false, false, true, false, true, false, true, false];
const VOLUME_ENVELOPE_CLOCKS: [bool; 8] = [false, false, false, false, false, false, false, true];
const SWEEP_CLOCKS: [bool; 8] = [false, false, true, false, false, false, true, false];

#[derive(Clone, Debug, Default)]
pub struct Channel1 {
    envelope_ticks_left: u8,
    sweep_ticks_left: u8,
    length_counter: u8,
    clock: u64,
    current_envelope_volume: u8,

    sweep: u8,
    sound_length_wave_duty: u8,
    volume_envelope: u8,
    frequency_low: u8,
    frequency_high: u8,
    frequency_shadow: u16,
    wave_duty_timer_ticks_left: u16,
    wave_duty_index: usize,
    frame_sequencer_idx: usize,
    enabled: bool,
}

impl Channel1 {
    pub fn step(&mut self) {
        if self.clock % SEQUENCER_CLOCK_PERIOD == 0 {
            if self.get_envelope_length() != 0 && VOLUME_ENVELOPE_CLOCKS[self.frame_sequencer_idx] {
                self.envelope_ticks_left = self.envelope_ticks_left.saturating_sub(1);
                if self.envelope_ticks_left == 0 {
                    if self.get_envelope_increase() {
                        self.current_envelope_volume = (self.current_envelope_volume + 1).min(0xF);
                    } else {
                        self.current_envelope_volume =
                            self.current_envelope_volume.saturating_sub(1);
                    }

                    self.envelope_ticks_left = if self.get_envelope_length() == 0 {
                        8
                    } else {
                        self.get_envelope_length()
                    };
                }
            }

            if self.get_sweep_length() != 0 && SWEEP_CLOCKS[self.frame_sequencer_idx] {
                self.sweep_ticks_left = self.sweep_ticks_left.saturating_sub(1);
                if self.sweep_ticks_left == 0 {
                    let new_value = if self.get_sweep_increase() {
                        self.frequency_shadow + (self.frequency_shadow >> self.get_sweep_shift())
                    } else {
                        self.frequency_shadow - (self.frequency_shadow >> self.get_sweep_shift())
                    };

                    if new_value <= 2047 {
                        self.write_channel_frequency(new_value);
                        self.frequency_shadow = new_value;
                    } else {
                        self.set_enabled(false);
                    };

                    self.sweep_ticks_left = if self.get_sweep_length() == 0 {
                        8
                    } else {
                        self.get_sweep_length()
                    };
                }
            }

            if self.stop_when_length_expires() && LENGTH_COUNTER_CLOCKS[self.frame_sequencer_idx] {
                self.length_counter = self.length_counter.saturating_sub(1);
                if self.length_counter == 0 {
                    self.set_enabled(false);
                }
            }

            self.frame_sequencer_idx = (self.frame_sequencer_idx + 1) % 8;
        }

        self.wave_duty_timer_ticks_left = self.wave_duty_timer_ticks_left.saturating_sub(1);
        if self.wave_duty_timer_ticks_left == 0 {
            self.wave_duty_timer_ticks_left = (2048 - self.get_channel_frequency()) * 4;
            self.wave_duty_index = (self.wave_duty_index + 1) % 8;
        }

        if self.get_initial_envelope_volume() == 0 {
            // self.set_enabled(false);
        }

        self.clock += 1;
    }

    pub fn sample(&self) -> u8 {
        let audio_high = match self.get_wave_pattern_duty() {
            WaveDuty::Eighth => EIGHTH_WAVE_DUTY_WAVEFORM[self.wave_duty_index],
            WaveDuty::Fourth => FOURTH_WAVE_DUTY_WAVEFORM[self.wave_duty_index],
            WaveDuty::Half => HALF_WAVE_DUTY_WAVEFORM[self.wave_duty_index],
            WaveDuty::ThreeQuarters => THREE_QUARTERS_WAVE_DUTY_WAVEFORM[self.wave_duty_index],
        };

        if audio_high && self.get_enabled() && self.get_initial_envelope_volume() != 0 {
            self.current_envelope_volume
        } else {
            0
        }
    }
}

impl Channel1 {
    pub fn read_sweep(&self) -> u8 {
        self.sweep
    }

    pub fn write_sweep(&mut self, value: u8) {
        self.sweep = value;
    }

    pub fn read_sound_length_wave_duty(&self) -> u8 {
        self.sound_length_wave_duty
    }

    pub fn write_sound_length_wave_duty(&mut self, value: u8) {
        const SOUND_LENGTH_MASK: u8 = 0b0011_1111;

        self.sound_length_wave_duty = value;
        self.length_counter = 64 - (value & SOUND_LENGTH_MASK);
    }

    pub fn read_volume_envelope(&self) -> u8 {
        self.volume_envelope
    }

    pub fn write_volume_envelope(&mut self, value: u8) {
        self.volume_envelope = value;

        self.current_envelope_volume = self.get_initial_envelope_volume();
        self.envelope_ticks_left = self.get_envelope_length();
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
}

impl Channel1 {
    fn get_sweep_length(&self) -> u8 {
        const SWEEP_REGISTER_TIME_SHIFT: usize = 4;
        const SWEEP_REGISTER_TIME_MASK: u8 = 0b111 << SWEEP_REGISTER_TIME_SHIFT;

        (self.sweep & SWEEP_REGISTER_TIME_MASK) >> SWEEP_REGISTER_TIME_SHIFT
    }

    fn get_sweep_increase(&self) -> bool {
        const SWEEP_REGISTER_INCREASE_MASK: u8 = 1 << 3;
        (self.sweep & SWEEP_REGISTER_INCREASE_MASK) == 0
    }

    fn get_sweep_shift(&self) -> u8 {
        const SWEEP_REGISTER_SWEEP_SHIFT: u8 = 0b111;
        self.sweep & SWEEP_REGISTER_SWEEP_SHIFT
    }

    fn get_wave_pattern_duty(&self) -> WaveDuty {
        const SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT: usize = 6;

        const SOUND_LENGTH_WAVE_PATTERN_DUTY_MASK: u8 =
            0b11 << SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT;
        const SOUND_LENGTH_WAVE_PATTERN_DUTY_EIGHTH_MASK: u8 =
            0b00 << SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT;
        const SOUND_LENGTH_WAVE_PATTERN_DUTY_FOURTH_MASK: u8 =
            0b01 << SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT;
        const SOUND_LENGTH_WAVE_PATTERN_DUTY_HALF_MASK: u8 =
            0b10 << SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT;
        const SOUND_LENGTH_WAVE_PATTERN_DUTY_THREE_QUARTERS_MASK: u8 =
            0b11 << SOUND_LENGTH_WAVE_PATTERN_DUTY_SHIFT;

        match self.sound_length_wave_duty & SOUND_LENGTH_WAVE_PATTERN_DUTY_MASK {
            SOUND_LENGTH_WAVE_PATTERN_DUTY_EIGHTH_MASK => WaveDuty::Eighth,
            SOUND_LENGTH_WAVE_PATTERN_DUTY_FOURTH_MASK => WaveDuty::Fourth,
            SOUND_LENGTH_WAVE_PATTERN_DUTY_HALF_MASK => WaveDuty::Half,
            SOUND_LENGTH_WAVE_PATTERN_DUTY_THREE_QUARTERS_MASK => WaveDuty::ThreeQuarters,
            _ => unreachable!(),
        }
    }

    fn get_initial_envelope_volume(&self) -> u8 {
        const INITIAL_VOLUME_ENVELOPE_SHIFT: usize = 4;
        const INITIAL_VOLUME_ENVELOPE_MASK: u8 = 0b1111 << INITIAL_VOLUME_ENVELOPE_SHIFT;

        (self.volume_envelope & INITIAL_VOLUME_ENVELOPE_MASK) >> INITIAL_VOLUME_ENVELOPE_SHIFT
    }

    fn get_envelope_increase(&self) -> bool {
        const ENVELOPE_DIRECTION_MASK: u8 = 1 << 3;

        (self.volume_envelope & ENVELOPE_DIRECTION_MASK) == ENVELOPE_DIRECTION_MASK
    }

    fn get_envelope_length(&self) -> u8 {
        const ENVELOPE_LENGTH_MASK: u8 = 0b111;

        self.volume_envelope & ENVELOPE_LENGTH_MASK
    }

    const CHANNEL_FREQUENCY_HIGH_MASK: u8 = 0b111;

    fn get_channel_frequency(&self) -> u16 {
        let channel_frequency_low = self.frequency_low;
        let channel_frequency_high = self.frequency_high & Self::CHANNEL_FREQUENCY_HIGH_MASK;
        let channel_frequency = u16::from_be_bytes([channel_frequency_high, channel_frequency_low]);
        channel_frequency
    }

    fn write_channel_frequency(&mut self, value: u16) {
        let [channel_frequency_high, channel_frequency_low] = value.to_be_bytes();

        self.frequency_low = channel_frequency_low;
        self.frequency_high = (channel_frequency_high & Self::CHANNEL_FREQUENCY_HIGH_MASK)
            | (self.frequency_high & !Self::CHANNEL_FREQUENCY_HIGH_MASK);
    }

    fn stop_when_length_expires(&self) -> bool {
        const FREQUENCY_HIGH_STOP_WHEN_LENGTH_EXPIRES_MASK: u8 = 0b0100_0000;
        (self.frequency_high & FREQUENCY_HIGH_STOP_WHEN_LENGTH_EXPIRES_MASK) != 0
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        if value {
            if self.length_counter == 0 {
                self.length_counter = 64;
            }
            self.wave_duty_timer_ticks_left = (2048 - self.get_channel_frequency()) * 4;
            self.envelope_ticks_left = self.get_envelope_length();
            self.current_envelope_volume = self.get_initial_envelope_volume();

            self.frequency_shadow = self.get_channel_frequency();
            self.sweep_ticks_left = self.get_sweep_length();

            self.enabled = true;
        } else {
            self.enabled = false;
        }
    }
}
