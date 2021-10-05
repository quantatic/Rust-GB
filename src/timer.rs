enum TimerControlCpuDivide {
    Divide16,
    Divide64,
    Divide256,
    Divide1024,
}

pub struct Timer {
    divider: u8,
    timer_counter: u8,
    timer_modulo: u8,
    timer_control_enable: bool,
    timer_control_cpu_divide: TimerControlCpuDivide,
    tick_counter: u16,
    interrupt_waiting: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            divider: Default::default(),
            timer_counter: Default::default(),
            timer_modulo: Default::default(),
            timer_control_enable: Default::default(),
            timer_control_cpu_divide: TimerControlCpuDivide::Divide16,
            tick_counter: 0,
            interrupt_waiting: false,
        }
    }
}

impl Timer {
    const DIVIDER_REGISTER_CPU_DIVIDE_RATIO: u16 = 256;

    pub fn step(&mut self) {
        if self.timer_control_enable {
            let timer_control_interval = match self.timer_control_cpu_divide {
                TimerControlCpuDivide::Divide16 => 16,
                TimerControlCpuDivide::Divide64 => 64,
                TimerControlCpuDivide::Divide256 => 256,
                TimerControlCpuDivide::Divide1024 => 1024,
            };

            self.tick_counter = self.tick_counter.wrapping_add(1);
            if self.tick_counter % timer_control_interval == 0 {
                let (new_timer_counter, overflow) = self.timer_counter.overflowing_add(1);
                if overflow {
                    self.interrupt_waiting = true;
                    self.timer_counter = self.timer_modulo;
                } else {
                    self.timer_counter = new_timer_counter;
                }
            }
        }

        if self.tick_counter % Self::DIVIDER_REGISTER_CPU_DIVIDE_RATIO == 0 {
            self.divider = self.divider.wrapping_add(1);
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

    // Writing any value to this register resets it to 0.
    pub fn set_divider_register(&mut self, _value: u8) {
        self.divider = 0;
    }

    pub fn get_divider_register(&self) -> u8 {
        self.divider
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

    pub fn set_timer_control(&mut self, value: u8) {
        self.timer_control_cpu_divide = match value & 0b0000_0011 {
            0b00 => TimerControlCpuDivide::Divide1024,
            0b01 => TimerControlCpuDivide::Divide16,
            0b10 => TimerControlCpuDivide::Divide64,
            0b11 => TimerControlCpuDivide::Divide256,
            _ => unreachable!(),
        };

        self.timer_control_enable = (value & 0b0000_0100) != 0;
    }

    pub fn get_timer_control(&self) -> u8 {
        let mut result = match self.timer_control_cpu_divide {
            TimerControlCpuDivide::Divide1024 => 0b00,
            TimerControlCpuDivide::Divide16 => 0b01,
            TimerControlCpuDivide::Divide64 => 0b10,
            TimerControlCpuDivide::Divide256 => 0b11,
        };

        if self.timer_control_enable {
            result |= 0b0000_0100;
        }

        result
    }
}
