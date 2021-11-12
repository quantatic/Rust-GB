mod channel_1;
mod channel_2;
mod channel_3;
mod channel_4;

use channel_1::Channel1;
use channel_2::Channel2;
use channel_3::Channel3;
use channel_4::Channel4;

use std::convert::TryFrom;

#[derive(Clone, Debug)]
enum WaveDuty {
    Eighth,
    Fourth,
    Half,
    ThreeQuarters,
}

const EIGHTH_WAVE_DUTY_WAVEFORM: [bool; 8] =
    [false, false, false, false, false, false, false, true];
const FOURTH_WAVE_DUTY_WAVEFORM: [bool; 8] = [true, false, false, false, false, false, false, true];
const HALF_WAVE_DUTY_WAVEFORM: [bool; 8] = [true, false, false, false, false, true, true, true];
const THREE_QUARTERS_WAVE_DUTY_WAVEFORM: [bool; 8] =
    [false, true, true, true, true, true, true, false];

#[derive(Clone, Default)]
pub struct Apu {
    channel_1: Channel1,
    channel_2: Channel2,
    channel_3: Channel3,
    channel_4: Channel4,
    channel_control: u8,
    output_terminal_selection: u8,
    all_sound_on: bool,
}

impl Apu {
    pub fn step(&mut self) {
        self.channel_1.step();
        self.channel_2.step();
        self.channel_3.step();
        self.channel_4.step();
    }

    pub fn sample(&mut self) -> [f32; 2] {
        fn digital_to_analog(value: u8) -> f32 {
            ((f32::from(value) / 15.0) * 2.0) - 1.0
        }

        if self.all_sound_on {
            let channel_1_sample = self.channel_1.sample();
            let channel_2_sample = self.channel_2.sample();
            let channel_3_sample = self.channel_3.sample();
            let channel_4_sample = self.channel_4.sample();

            let mut left_output = 0.0;
            let mut right_output = 0.0;

            if self.output_sound_1_left() {
                left_output += digital_to_analog(channel_1_sample);
            }

            if self.output_sound_2_left() {
                left_output += digital_to_analog(channel_2_sample);
            }

            if self.output_sound_3_left() {
                left_output += digital_to_analog(channel_3_sample);
            }

            if self.output_sound_4_left() {
                left_output += digital_to_analog(channel_4_sample);
            }

            if self.output_sound_1_right() {
                right_output += digital_to_analog(channel_1_sample);
            }

            if self.output_sound_2_right() {
                right_output += digital_to_analog(channel_2_sample);
            }

            if self.output_sound_3_right() {
                right_output += digital_to_analog(channel_3_sample);
            }

            if self.output_sound_4_right() {
                right_output += digital_to_analog(channel_4_sample);
            }

            left_output *= f32::from(self.get_left_output_volume() + 1);
            right_output *= f32::from(self.get_right_output_volume() + 1);

            [left_output / 32.0, right_output / 32.0]
        } else {
            [-1.0; 2]
        }
    }

    pub fn read_nr10(&self) -> u8 {
        self.channel_1.read_sweep()
    }

    pub fn write_nr10(&mut self, value: u8) {
        self.channel_1.write_sweep(value);
    }

    pub fn read_nr11(&self) -> u8 {
        self.channel_1.read_sound_length_wave_duty()
    }

    pub fn write_nr11(&mut self, value: u8) {
        self.channel_1.write_sound_length_wave_duty(value)
    }

    pub fn read_nr12(&self) -> u8 {
        self.channel_1.read_volume_envelope()
    }

    pub fn write_nr12(&mut self, value: u8) {
        self.channel_1.write_volume_envelope(value)
    }

    pub fn read_nr13(&self) -> u8 {
        self.channel_1.read_frequency_low()
    }

    pub fn write_nr13(&mut self, value: u8) {
        self.channel_1.write_frequency_low(value)
    }

    pub fn read_nr14(&self) -> u8 {
        self.channel_1.read_frequency_high()
    }

    pub fn write_nr14(&mut self, value: u8) {
        self.channel_1.write_frequency_high(value)
    }

    pub fn read_nr21(&self) -> u8 {
        self.channel_2.read_sound_length_wave_duty()
    }

    pub fn write_nr21(&mut self, value: u8) {
        self.channel_2.write_sound_length_wave_duty(value)
    }

    pub fn read_nr22(&self) -> u8 {
        self.channel_2.read_volume_envelope()
    }

    pub fn write_nr22(&mut self, value: u8) {
        self.channel_2.write_volume_envelope(value)
    }

    pub fn read_nr23(&self) -> u8 {
        self.channel_2.read_frequency_low()
    }

    pub fn write_nr23(&mut self, value: u8) {
        self.channel_2.write_frequency_low(value)
    }

    pub fn read_nr24(&self) -> u8 {
        self.channel_2.read_frequency_high()
    }

    pub fn write_nr24(&mut self, value: u8) {
        self.channel_2.write_frequency_high(value)
    }

    pub fn read_nr30(&self) -> u8 {
        self.channel_3.read_sound_on_off()
    }

    pub fn write_nr30(&mut self, value: u8) {
        self.channel_3.write_sound_on_off(value);
    }

    pub fn read_nr31(&self) -> u8 {
        self.channel_3.read_sound_length()
    }

    pub fn write_nr31(&mut self, value: u8) {
        self.channel_3.write_sound_length(value)
    }

    pub fn read_nr32(&self) -> u8 {
        self.channel_3.read_output_level()
    }

    pub fn write_nr32(&mut self, value: u8) {
        self.channel_3.write_output_level(value)
    }

    pub fn read_nr33(&self) -> u8 {
        self.channel_3.read_frequency_low()
    }

    pub fn write_nr33(&mut self, value: u8) {
        self.channel_3.write_frequency_low(value)
    }

    pub fn read_nr34(&self) -> u8 {
        self.channel_3.read_frequency_high()
    }

    pub fn write_nr34(&mut self, value: u8) {
        self.channel_3.write_frequency_high(value)
    }

    pub fn read_wave_pattern_ram(&self, offset: u16) -> u8 {
        self.channel_3.read_wave_pattern_ram(offset)
    }

    pub fn write_wave_pattern_ram(&mut self, value: u8, offset: u16) {
        self.channel_3.write_wave_pattern_ram(value, offset)
    }

    pub fn read_nr41(&self) -> u8 {
        self.channel_4.read_sound_length_register()
    }

    pub fn write_nr41(&mut self, value: u8) {
        self.channel_4.write_sound_length_register(value)
    }

    pub fn read_nr42(&self) -> u8 {
        self.channel_4.read_volume_envelope()
    }

    pub fn write_nr42(&mut self, value: u8) {
        self.channel_4.write_volume_envelope(value)
    }

    pub fn read_nr43(&self) -> u8 {
        self.channel_4.read_polynomial_counter()
    }

    pub fn write_nr43(&mut self, value: u8) {
        self.channel_4.write_polynomial_counter(value)
    }

    pub fn read_nr44(&self) -> u8 {
        self.channel_4.read_counter_consecutive()
    }

    pub fn write_nr44(&mut self, value: u8) {
        self.channel_4.write_counter_consecutive(value)
    }

    pub fn read_nr50(&self) -> u8 {
        self.channel_control
    }

    pub fn write_nr50(&mut self, value: u8) {
        self.channel_control = value
    }

    pub fn read_nr51(&self) -> u8 {
        self.output_terminal_selection
    }

    pub fn write_nr51(&mut self, value: u8) {
        self.output_terminal_selection = value
    }

    const ALL_SOUND_ON_OFF_FLAG: u8 = 1 << 7;
    const SOUND_4_ON_OFF_FLAG: u8 = 1 << 3;
    const SOUND_3_ON_OFF_FLAG: u8 = 1 << 2;
    const SOUND_2_ON_OFF_FLAG: u8 = 1 << 1;
    const SOUND_1_ON_OFF_FLAG: u8 = 1 << 0;

    pub fn read_nr52(&self) -> u8 {
        let mut result = 0;
        if self.all_sound_on {
            result |= Self::ALL_SOUND_ON_OFF_FLAG;
        }

        if self.channel_1.get_enabled() {
            result |= Self::SOUND_1_ON_OFF_FLAG;
        }

        if self.channel_2.get_enabled() {
            result |= Self::SOUND_2_ON_OFF_FLAG;
        }

        if self.channel_3.get_enabled() {
            result |= Self::SOUND_3_ON_OFF_FLAG;
        }

        if self.channel_4.get_enabled() {
            result |= Self::SOUND_4_ON_OFF_FLAG;
        }

        result
    }

    pub fn write_nr52(&mut self, value: u8) {
        let sound_setting = (value & Self::ALL_SOUND_ON_OFF_FLAG) == Self::ALL_SOUND_ON_OFF_FLAG;
        self.all_sound_on = sound_setting;
        self.channel_1.set_enabled(sound_setting);
        self.channel_2.set_enabled(sound_setting);
        self.channel_3.set_enabled(sound_setting);
        self.channel_4.set_enabled(sound_setting);
    }
}

impl Apu {
    fn get_left_output_volume(&self) -> u8 {
        const CHANNEL_CONTROL_LEFT_OUTPUT_VOLUME_SHIFT: usize = 4;
        const CHANNEL_CONTROL_LEFT_OUTPUT_VOLUME_MASK: u8 =
            0b111 << CHANNEL_CONTROL_LEFT_OUTPUT_VOLUME_SHIFT;

        (self.channel_control & CHANNEL_CONTROL_LEFT_OUTPUT_VOLUME_MASK)
            >> CHANNEL_CONTROL_LEFT_OUTPUT_VOLUME_SHIFT
    }

    fn get_right_output_volume(&self) -> u8 {
        const CHANNEL_CONTROL_RIGHT_OUTPUT_VOLUME_MASK: u8 = 0b111;

        self.channel_control & CHANNEL_CONTROL_RIGHT_OUTPUT_VOLUME_MASK
    }

    fn output_sound_1_left(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_LEFT_MASK: u8 = 1 << 4;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_LEFT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_LEFT_MASK
    }

    fn output_sound_2_left(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_LEFT_MASK: u8 = 1 << 5;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_LEFT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_LEFT_MASK
    }

    fn output_sound_3_left(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_LEFT_MASK: u8 = 1 << 6;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_LEFT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_LEFT_MASK
    }

    fn output_sound_4_left(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_LEFT_MASK: u8 = 1 << 7;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_LEFT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_LEFT_MASK
    }

    fn output_sound_1_right(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_RIGHT_MASK: u8 = 1 << 0;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_RIGHT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_1_RIGHT_MASK
    }

    fn output_sound_2_right(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_RIGHT_MASK: u8 = 1 << 1;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_RIGHT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_2_RIGHT_MASK
    }

    fn output_sound_3_right(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_RIGHT_MASK: u8 = 1 << 2;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_RIGHT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_3_RIGHT_MASK
    }

    fn output_sound_4_right(&self) -> bool {
        const OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_RIGHT_MASK: u8 = 1 << 3;
        (self.output_terminal_selection & OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_RIGHT_MASK)
            == OUTPUT_TERMINAL_SELECTION_OUTPUT_SOUND_4_RIGHT_MASK
    }
}
