#[derive(Default)]
pub struct Joypad {
    action_buttons_selected: bool,
    direction_buttons_selected: bool,
    down_pressed: bool,
    up_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
    start_pressed: bool,
    select_pressed: bool,
    b_pressed: bool,
    a_pressed: bool,
    interrupt_waiting: bool,
}

impl Joypad {
    const SELECT_ACTION_BUTTONS_MASK: u8 = 1 << 5;
    const SELECT_DIRECTION_BUTTONS_MASK: u8 = 1 << 4;
    const INPUT_DOWN_START_MASK: u8 = 1 << 3;
    const INPUT_UP_SELECT_MASK: u8 = 1 << 2;
    const INPUT_LEFT_B_MASK: u8 = 1 << 1;
    const INPUT_RIGHT_A_MASK: u8 = 1 << 0;

    pub fn set_up_pressed(&mut self, val: bool) {
        self.up_pressed = val;
    }

    pub fn set_down_pressed(&mut self, val: bool) {
        self.down_pressed = val;
    }

    pub fn set_left_pressed(&mut self, val: bool) {
        self.left_pressed = val;
    }

    pub fn set_right_pressed(&mut self, val: bool) {
        self.right_pressed = val;
    }

    pub fn set_start_pressed(&mut self, val: bool) {
        self.start_pressed = val;
    }

    pub fn set_select_pressed(&mut self, val: bool) {
        self.select_pressed = val;
    }

    pub fn set_b_pressed(&mut self, val: bool) {
        self.b_pressed = val;
    }

    pub fn set_a_pressed(&mut self, val: bool) {
        self.a_pressed = val;
    }

    pub fn poll_interrupt(&mut self) -> bool {
        if self.interrupt_waiting {
            self.interrupt_waiting = false;
            true
        } else {
            false
        }
    }

    pub fn write(&mut self, data: u8) {
        self.action_buttons_selected = (data & Self::SELECT_ACTION_BUTTONS_MASK) == 0;
        self.direction_buttons_selected = (data & Self::SELECT_DIRECTION_BUTTONS_MASK) == 0;
    }

    pub fn read(&self) -> u8 {
        let mut result = 0;

        if self.action_buttons_selected {
            if !self.start_pressed {
                result |= Self::INPUT_DOWN_START_MASK;
            }

            if !self.select_pressed {
                result |= Self::INPUT_UP_SELECT_MASK;
            }

            if !self.b_pressed {
                result |= Self::INPUT_LEFT_B_MASK;
            }

            if !self.a_pressed {
                result |= Self::INPUT_RIGHT_A_MASK;
            }
        } else {
            result |= Self::SELECT_ACTION_BUTTONS_MASK;
        }

        if self.direction_buttons_selected {
            if !self.down_pressed {
                result |= Self::INPUT_DOWN_START_MASK;
            }

            if !self.up_pressed {
                result |= Self::INPUT_UP_SELECT_MASK;
            }

            if !self.left_pressed {
                result |= Self::INPUT_LEFT_B_MASK;
            }

            if !self.right_pressed {
                result |= Self::INPUT_RIGHT_A_MASK;
            }
        } else {
            result |= Self::SELECT_DIRECTION_BUTTONS_MASK;
        }

        result
    }
}
