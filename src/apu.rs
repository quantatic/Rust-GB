#[derive(Clone, Default)]
pub struct Apu {
    nr10: u8,
    nr11: u8,
    nr12: u8,
    nr13: u8,
    nr14: u8,
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,
    nr30: u8,
    nr31: u8,
    nr32: u8,
    nr33: u8,
    nr34: u8,
    nr41: u8,
    nr42: u8,
    nr43: u8,
    nr44: u8,
    nr50: u8,
    nr51: u8,
    nr52: u8,
}

impl Apu {
    pub fn read_nr10(&self) -> u8 {
        self.nr10
    }

    pub fn write_nr10(&mut self, value: u8) {
        self.nr10 = value
    }

    pub fn read_nr11(&self) -> u8 {
        self.nr11
    }

    pub fn write_nr11(&mut self, value: u8) {
        self.nr11 = value
    }

    pub fn read_nr12(&self) -> u8 {
        self.nr12
    }

    pub fn write_nr12(&mut self, value: u8) {
        self.nr12 = value
    }

    pub fn read_nr13(&self) -> u8 {
        self.nr13
    }

    pub fn write_nr13(&mut self, value: u8) {
        self.nr13 = value
    }

    pub fn read_nr14(&self) -> u8 {
        self.nr14
    }

    pub fn write_nr14(&mut self, value: u8) {
        self.nr14 = value
    }

    pub fn read_nr21(&self) -> u8 {
        self.nr21
    }

    pub fn write_nr21(&mut self, value: u8) {
        self.nr21 = value
    }

    pub fn read_nr22(&self) -> u8 {
        self.nr22
    }

    pub fn write_nr22(&mut self, value: u8) {
        self.nr22 = value
    }

    pub fn read_nr23(&self) -> u8 {
        self.nr23
    }

    pub fn write_nr23(&mut self, value: u8) {
        self.nr23 = value
    }

    pub fn read_nr24(&self) -> u8 {
        self.nr24
    }

    pub fn write_nr24(&mut self, value: u8) {
        self.nr24 = value
    }

    pub fn read_nr30(&self) -> u8 {
        self.nr30
    }

    pub fn write_nr30(&mut self, value: u8) {
        self.nr30 = value
    }

    pub fn read_nr31(&self) -> u8 {
        self.nr31
    }

    pub fn write_nr31(&mut self, value: u8) {
        self.nr31 = value
    }

    pub fn read_nr32(&self) -> u8 {
        self.nr32
    }

    pub fn write_nr32(&mut self, value: u8) {
        self.nr32 = value
    }

    pub fn read_nr33(&self) -> u8 {
        self.nr33
    }

    pub fn write_nr33(&mut self, value: u8) {
        self.nr33 = value
    }

    pub fn read_nr34(&self) -> u8 {
        self.nr34
    }

    pub fn write_nr34(&mut self, value: u8) {
        self.nr34 = value
    }

    pub fn read_nr41(&self) -> u8 {
        self.nr41
    }

    pub fn write_nr41(&mut self, value: u8) {
        self.nr41 = value
    }

    pub fn read_nr42(&self) -> u8 {
        self.nr42
    }

    pub fn write_nr42(&mut self, value: u8) {
        self.nr42 = value
    }

    pub fn read_nr43(&self) -> u8 {
        self.nr43
    }

    pub fn write_nr43(&mut self, value: u8) {
        self.nr43 = value
    }

    pub fn read_nr44(&self) -> u8 {
        self.nr44
    }

    pub fn write_nr44(&mut self, value: u8) {
        self.nr44 = value
    }

    pub fn read_nr50(&self) -> u8 {
        self.nr50
    }

    pub fn write_nr50(&mut self, value: u8) {
        self.nr50 = value
    }

    pub fn read_nr51(&self) -> u8 {
        self.nr51
    }

    pub fn write_nr51(&mut self, value: u8) {
        self.nr51 = value
    }

    pub fn read_nr52(&self) -> u8 {
        self.nr52
    }

    pub fn write_nr52(&mut self, value: u8) {
        self.nr52 = value
    }
}
