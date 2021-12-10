#[derive(Clone, Copy, Debug)]
enum InputClockSelect {
    Bit3,
    Bit5,
    Bit7,
    Bit9,
}

#[derive(Clone, Default)]
pub struct Timer {
    pub timer_counter: u8,
    timer_counter_reload_delay: u8,
    timer_modulo: u8,
    timer_control: u8,
    pub tick_counter: u16,
    interrupt_waiting: bool,
}

impl Timer {
    const TIMER_COUNTER_RELOAD_DELAY: u8 = 4;

    pub fn step(&mut self) {
        if self.timer_counter_reload_delay > 0 {
            self.timer_counter_reload_delay -= 1;
            // We can prevent the reload and interrupt by writing manually to
            // the timer counter during the reload delay.
            if self.timer_counter_reload_delay == 0 && self.timer_counter == 0 {
                self.timer_counter = self.timer_modulo;
                self.interrupt_waiting = true;
            }
        }

        let input_clock_select_mask = if self.get_timer_enable() {
            match self.get_input_clock_select() {
                InputClockSelect::Bit3 => (1 << 3),
                InputClockSelect::Bit5 => (1 << 5),
                InputClockSelect::Bit7 => (1 << 7),
                InputClockSelect::Bit9 => (1 << 9),
            }
        } else {
            0
        };

        let old_timer_increment_bit = (self.tick_counter & input_clock_select_mask) != 0;
        self.tick_counter = self.tick_counter.wrapping_add(1);
        let new_timer_increment_bit = (self.tick_counter & input_clock_select_mask) != 0;

        // Timer counter gets incremented on falling edge.
        //
        // When timer counter overflows, there is a 4 clock delay before
        // actually being reloaded.
        if old_timer_increment_bit && !new_timer_increment_bit {
            let (new_timer_counter, overflow) = self.timer_counter.overflowing_add(1);
            if overflow {
                self.timer_counter_reload_delay = Self::TIMER_COUNTER_RELOAD_DELAY;
            }

            self.timer_counter = new_timer_counter;
        }
    }

    pub fn poll_interrupt(&mut self) -> bool {
        if self.interrupt_waiting {
            self.interrupt_waiting = false;
            true
        } else {
            false
        }
    }

    // Writing any value to divider register resets it and tick counter to 0.
    //
    // Timer increment is actually tied to a certain bit in divider register
    // going low (depending on the CPU divide value). When divider register
    // is reset via manually writing to it, timer may still be incremented, if
    // the relevant bit goes low as a result of this write.
    pub fn set_divider_register(&mut self, _value: u8) {
        let input_clock_select_mask = if self.get_timer_enable() {
            match self.get_input_clock_select() {
                InputClockSelect::Bit3 => (1 << 3),
                InputClockSelect::Bit5 => (1 << 5),
                InputClockSelect::Bit7 => (1 << 7),
                InputClockSelect::Bit9 => (1 << 9),
            }
        } else {
            0
        };

        let old_timer_increment_bit = (self.tick_counter & input_clock_select_mask) != 0;
        self.tick_counter = 0;
        let new_timer_increment_bit = (self.tick_counter & input_clock_select_mask) != 0;

        // timer gets incremented on falling edge
        if old_timer_increment_bit && !new_timer_increment_bit {
            let (new_timer_counter, overflow) = self.timer_counter.overflowing_add(1);
            if overflow {
                self.timer_counter = self.timer_modulo;
                self.interrupt_waiting = true;
            } else {
                self.timer_counter = new_timer_counter;
            }
        }
    }

    pub fn get_divider_register(&self) -> u8 {
        let [divider_register, _] = self.tick_counter.to_be_bytes();
        divider_register
    }

    pub fn set_timer_counter(&mut self, value: u8) {
        self.timer_counter = value;
    }

    pub fn get_timer_counter(&self) -> u8 {
        self.timer_counter
    }

    pub fn set_timer_modulo(&mut self, value: u8) {
        self.timer_modulo = value;
    }

    pub fn get_timer_modulo(&self) -> u8 {
        self.timer_modulo
    }

    const TIMER_CONTROL_INPUT_CLOCK_SELECT_MASK: u8 = 0b11;
    const TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_3_MASK: u8 = 0b01;
    const TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_5_MASK: u8 = 0b10;
    const TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_7_MASK: u8 = 0b11;
    const TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_9_MASK: u8 = 0b00;

    const TIMER_CONTROL_ENABLE_MASK: u8 = 1 << 2;

    fn get_timer_enable(&self) -> bool {
        (self.timer_control & Self::TIMER_CONTROL_ENABLE_MASK) != 0
    }

    fn get_input_clock_select(&self) -> InputClockSelect {
        match self.timer_control & Self::TIMER_CONTROL_INPUT_CLOCK_SELECT_MASK {
            Self::TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_3_MASK => InputClockSelect::Bit3,
            Self::TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_5_MASK => InputClockSelect::Bit5,
            Self::TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_7_MASK => InputClockSelect::Bit7,
            Self::TIMER_CONTROL_INPUT_CLOCK_SELECT_BIT_9_MASK => InputClockSelect::Bit9,
            _ => unreachable!(),
        }
    }

    pub fn set_timer_control(&mut self, value: u8) {
        // If timer has been disabled or multiplexer output changes from 1 -> 0,
        // divider register also effectively goes low wrt. falling edge detector
        // timer increment.
        let old_input_clock_select_mask = if self.get_timer_enable() {
            match self.get_input_clock_select() {
                InputClockSelect::Bit3 => (1 << 3),
                InputClockSelect::Bit5 => (1 << 5),
                InputClockSelect::Bit7 => (1 << 7),
                InputClockSelect::Bit9 => (1 << 9),
            }
        } else {
            0
        };
        let old_timer_increment_bit = (self.tick_counter & old_input_clock_select_mask) != 0;

        self.timer_control = value;

        let new_input_clock_select_mask = if self.get_timer_enable() {
            match self.get_input_clock_select() {
                InputClockSelect::Bit3 => (1 << 3),
                InputClockSelect::Bit5 => (1 << 5),
                InputClockSelect::Bit7 => (1 << 7),
                InputClockSelect::Bit9 => (1 << 9),
            }
        } else {
            0
        };
        let new_timer_increment_bit = (self.tick_counter & new_input_clock_select_mask) != 0;

        if old_timer_increment_bit && !new_timer_increment_bit {
            println!("increment from timer control change");
            let (new_timer_counter, overflow) = self.timer_counter.overflowing_add(1);
            if overflow {
                self.timer_counter = self.timer_modulo;
                self.interrupt_waiting = true;
                println!("timer interrupt waiting");
            } else {
                self.timer_counter = new_timer_counter;
            }
        }
    }

    pub fn get_timer_control(&self) -> u8 {
        self.timer_control
    }
}
