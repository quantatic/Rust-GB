use crate::CLOCK_FREQUENCY;

use super::{
    WaveDuty, EIGHTH_WAVE_DUTY_WAVEFORM, FOURTH_WAVE_DUTY_WAVEFORM, HALF_WAVE_DUTY_WAVEFORM,
    THREE_QUARTERS_WAVE_DUTY_WAVEFORM,
};

const SEQUENCER_CLOCK_FREQUENCY: u32 = 512;

const SEQUENCER_CLOCK_PERIOD: u32 = CLOCK_FREQUENCY / SEQUENCER_CLOCK_FREQUENCY;

const LENGTH_COUNTER_CLOCKS: [bool; 8] = [false, false, true, false, true, false, true, false];
const VOLUME_ENVELOPE_CLOCKS: [bool; 8] = [false, false, false, false, false, false, false, true];

#[derive(Clone, Debug, Default)]
pub struct Channel4 {
    sound_length: u8,
    length_counter: u8,
    envelope_ticks_left: u8,
    volume_envelope: u8,
    current_envelope_volume: u8,
    polynomial_counter: u8,
    linear_feedback_shift_register: u16,
    counter_consecutive: u8,

    noise_ticks_left: u16,
    frame_sequencer_idx: usize,

    clock: u32,

    enabled: bool,
}

impl Channel4 {
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

            if self.stop_when_length_expires() && LENGTH_COUNTER_CLOCKS[self.frame_sequencer_idx] {
                self.length_counter = self.length_counter.saturating_sub(1);
                if self.length_counter == 0 {
                    self.set_enabled(false);
                }
            }

            self.frame_sequencer_idx = (self.frame_sequencer_idx + 1) % 8;
        }

        self.noise_ticks_left = self.noise_ticks_left.saturating_sub(1);
        if self.noise_ticks_left == 0 {
            let xor_result = (self.linear_feedback_shift_register & 0b01)
                ^ ((self.linear_feedback_shift_register & 0b10) >> 1);
            self.linear_feedback_shift_register =
                (self.linear_feedback_shift_register >> 1) | (xor_result << 14);

            if self.get_counter_byte_width() {
                self.linear_feedback_shift_register &= !(1 << 6);
                self.linear_feedback_shift_register |= xor_result << 6;
            }

            self.noise_ticks_left =
                u16::from(self.get_divisor()) << u16::from(self.get_shift_clock_frequency());
        }

        self.clock += 1;
    }

    pub fn sample(&self) -> u8 {
        let audio_high = (self.linear_feedback_shift_register & 0b1) == 0;

        if audio_high && self.get_enabled() && self.get_initial_envelope_volume() != 0 {
            self.current_envelope_volume
        } else {
            0
        }
    }
}

impl Channel4 {
    pub fn read_sound_length_register(&self) -> u8 {
        self.sound_length
    }

    pub fn write_sound_length_register(&mut self, value: u8) {
        const SOUND_LENGTH_MASK: u8 = 0b0011_1111;

        self.sound_length = value;
        self.length_counter = 64 - (value & SOUND_LENGTH_MASK);
    }

    pub fn read_volume_envelope(&self) -> u8 {
        self.volume_envelope
    }

    pub fn write_volume_envelope(&mut self, value: u8) {
        self.volume_envelope = value;

        self.envelope_ticks_left = self.get_envelope_length();
    }

    pub fn read_polynomial_counter(&self) -> u8 {
        self.polynomial_counter
    }

    pub fn write_polynomial_counter(&mut self, value: u8) {
        self.polynomial_counter = value;
    }

    pub fn read_counter_consecutive(&self) -> u8 {
        self.counter_consecutive
    }

    pub fn write_counter_consecutive(&mut self, value: u8) {
        const COUNTER_CONSECUTIVE_ENABLED_MASK: u8 = 1 << 7;

        if (value & COUNTER_CONSECUTIVE_ENABLED_MASK) == COUNTER_CONSECUTIVE_ENABLED_MASK {
            self.set_enabled(true);
        }
        self.counter_consecutive = value
    }
}

impl Channel4 {
    fn get_initial_envelope_volume(&self) -> u8 {
        const INITIAL_VOLUME_ENVELOPE_SHIFT: usize = 4;
        const INITIAL_VOLUME_ENVELOPE_MASK: u8 = 0b1111 << INITIAL_VOLUME_ENVELOPE_SHIFT;

        (self.volume_envelope & INITIAL_VOLUME_ENVELOPE_MASK) >> INITIAL_VOLUME_ENVELOPE_SHIFT
    }

    fn get_envelope_increase(&self) -> bool {
        const ENVELOPE_DIRECTION_MASK: u8 = 0b1000;

        (self.volume_envelope & ENVELOPE_DIRECTION_MASK) == ENVELOPE_DIRECTION_MASK
    }

    fn get_envelope_length(&self) -> u8 {
        const ENVELOPE_LENGTH_MASK: u8 = 0b111;

        self.volume_envelope & ENVELOPE_LENGTH_MASK
    }

    fn get_shift_clock_frequency(&self) -> u8 {
        const POLYNOMIAL_COUNTER_SHIFT_CLOCK_FREQUENCY_SHIFT: usize = 4;
        const POLYNOMIAL_COUNTER_SHIFT_CLOCK_FREQUENCY_MASK: u8 =
            0b1111 << POLYNOMIAL_COUNTER_SHIFT_CLOCK_FREQUENCY_SHIFT;

        (self.polynomial_counter & POLYNOMIAL_COUNTER_SHIFT_CLOCK_FREQUENCY_MASK)
            >> POLYNOMIAL_COUNTER_SHIFT_CLOCK_FREQUENCY_SHIFT
    }

    fn get_counter_byte_width(&self) -> bool {
        const POLYNOMIAL_COUNTER_STEP_WIDTH_MASK: u8 = 0b1000;

        (self.polynomial_counter & POLYNOMIAL_COUNTER_STEP_WIDTH_MASK)
            == POLYNOMIAL_COUNTER_STEP_WIDTH_MASK
    }

    fn get_divisor_code(&self) -> u8 {
        const POLYNOMIAL_COUNTER_DIVISOR_CODE_MASK: u8 = 0b111;

        self.polynomial_counter & POLYNOMIAL_COUNTER_DIVISOR_CODE_MASK
    }

    fn get_divisor(&self) -> u8 {
        match self.get_divisor_code() {
            0 => 8,
            val @ 1..=7 => val << 4,
            _ => unreachable!(),
        }
    }

    fn stop_when_length_expires(&self) -> bool {
        const COUNTER_CONSECUTIVE_STOP_WHEN_LENGTH_EXPIRES_MASK: u8 = 0b0100_0000;
        (self.counter_consecutive & COUNTER_CONSECUTIVE_STOP_WHEN_LENGTH_EXPIRES_MASK)
            == COUNTER_CONSECUTIVE_STOP_WHEN_LENGTH_EXPIRES_MASK
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        if value {
            if self.length_counter == 0 {
                self.length_counter = 64;
            }
            self.noise_ticks_left =
                u16::from(self.get_divisor()) << u16::from(self.get_shift_clock_frequency());
            self.envelope_ticks_left = self.get_envelope_length();
            self.current_envelope_volume = self.get_initial_envelope_volume();

            self.linear_feedback_shift_register = !0;

            self.enabled = true
        } else {
            self.enabled = false
        }
    }

    pub fn set_power(&mut self, value: bool) {
        if value {
            self.frame_sequencer_idx = 0;
        } else {
            self.sound_length = 0;
            self.volume_envelope = 0;
            self.current_envelope_volume = 0;
            self.polynomial_counter = 0;
            self.linear_feedback_shift_register = 0;
            self.counter_consecutive = 0;

            self.set_enabled(false);
        }
    }
}
