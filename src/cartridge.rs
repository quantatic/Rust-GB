use std::convert::TryFrom;
use std::{convert::TryInto, error::Error, time::Instant};

#[derive(Clone)]
pub struct Cartridge {
    cartridge_type: CartridgeType,
    title: String,
}

#[derive(Clone)]
enum CartridgeType {
    NoMbc(NoMbc),
    Mbc1(Mbc1),
    Mbc3(Mbc3),
}

impl Cartridge {
    pub fn read(&self, address: u16) -> u8 {
        match &self.cartridge_type {
            CartridgeType::NoMbc(no_mbc) => no_mbc.read(address),
            CartridgeType::Mbc1(mbc_1) => mbc_1.read(address),
            CartridgeType::Mbc3(mbc_3) => mbc_3.read(address),
        }
    }

    pub fn write(&mut self, value: u8, address: u16) {
        match &mut self.cartridge_type {
            CartridgeType::NoMbc(no_mbc) => no_mbc.write(value, address),
            CartridgeType::Mbc1(mbc_1) => mbc_1.write(value, address),
            CartridgeType::Mbc3(mbc_3) => mbc_3.write(value, address),
        }
    }

    pub fn step(&mut self) {
        match &mut self.cartridge_type {
            CartridgeType::NoMbc(_) => {}
            CartridgeType::Mbc1(_) => {}
            CartridgeType::Mbc3(mbc_3) => mbc_3.step(),
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }
}

#[derive(Clone)]
struct NoMbc {
    rom: Vec<u8>,
    ram: Vec<[u8; 0x2000]>,
}

impl NoMbc {
    fn new(data: &[u8], ram_size: usize) -> Result<Self, Box<dyn Error>> {
        let ram = if ram_size == 0x0000 {
            Vec::new()
        } else if ram_size == 0x2000 {
            vec![[0; 0x2000]]
        } else {
            return Err(format!(
                "expected ram size of 0x0000 or 0x2000, but got 0x{:04X}",
                ram_size
            )
            .into());
        };

        Ok(Self {
            rom: data.to_vec(),
            ram,
        })
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.rom[usize::from(address)],
            0xA000..=0xBFFF => self.ram[0][usize::from(address - 0xA000)],
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x7FFF => {} // writing to ROM does nothing with no MBC
            0xA000..=0xBFFF => self.ram[0][usize::from(address - 0xA000)] = value,
            _ => unreachable!(),
        };
    }
}

#[derive(Clone)]
struct Mbc1 {
    rom: Vec<[u8; 0x4000]>,
    rom_banks: usize,
    bank_1: usize,
    ram: Vec<[u8; 0x2000]>,
    ram_banks: usize,
    bank_2: usize,
    ram_enabled: bool,
    simple_rom_banking: bool,
}

impl Mbc1 {
    const EXPECTED_RAM_SIZES: [usize; 3] = [0x0000, 0x2000, 0x8000];

    fn new(data: &[u8], ram_size: usize) -> Result<Self, Box<dyn Error>> {
        if !Self::EXPECTED_RAM_SIZES.contains(&ram_size) {
            let expected_string = format!(
                "[{}]",
                Self::EXPECTED_RAM_SIZES
                    .map(|size| format!("0x{:04X}", size))
                    .join(", ")
            );

            return Err(format!(
                "expected ram size to be one of {}, but got 0x{:04X}",
                expected_string, ram_size
            )
            .into());
        }

        let rom: Vec<[u8; 0x4000]> = data
            .chunks(0x4000)
            .map(<[u8; 0x4000]>::try_from)
            .collect::<Result<_, _>>()?;

        let ram: Vec<[u8; 0x2000]> = vec![[0; 0x2000]; ram_size / 0x2000];

        Ok(Self {
            rom_banks: rom.len(),
            rom,
            bank_1: 1,
            ram_banks: ram.len(),
            ram,
            bank_2: 0,
            ram_enabled: false,
            simple_rom_banking: true,
        })
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let bank_number = if self.simple_rom_banking {
                    0
                } else {
                    self.bank_2 << 5
                };
                self.rom[bank_number % self.rom_banks][usize::from(address)]
            }
            0x4000..=0x7FFF => {
                let bank_number = self.bank_1 | (self.bank_2 << 5);
                let result = self.rom[bank_number % self.rom_banks][usize::from(address - 0x4000)];
                result
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.simple_rom_banking {
                        self.ram[0][usize::from(address - 0xA000)]
                    } else {
                        self.ram[self.bank_2 % self.ram_banks][usize::from(address - 0xA000)]
                    }
                } else {
                    0xFF
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = (value & 0xF) == 0x0A,
            0x2000..=0x3FFF => {
                self.bank_1 = usize::from(value) & 0b11111;
                // Bank 1 is not allowed to contain the value 0
                if self.bank_1 == 0 {
                    self.bank_1 = 1;
                }
            }
            0x4000..=0x5FFF => {
                self.bank_2 = usize::from(value & 0b11);
            }
            0x6000..=0x7FFF => {
                self.simple_rom_banking = (value & 0b1) == 0;
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.simple_rom_banking {
                        self.ram[0][usize::from(address - 0xA000)] = value;
                    } else {
                        self.ram[self.bank_2 % self.ram_banks][usize::from(address - 0xA000)] =
                            value;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
struct Mbc3 {
    rom: Vec<[u8; 0x4000]>,
    rom_bank: usize,
    ram: Vec<[u8; 0x2000]>,
    ram_bank: usize,
    ram_enabled: bool,
    rtc_secs: u8,
    rtc_mins: u8,
    rtc_hours: u8,
    rtc_dl: u8,
    rtc_dh: u8,
    latch_state: RtcLatchState,
    last_step_time: Instant,
    background_secs: f64,
}

#[derive(Clone, Copy, Debug)]
enum RtcLatchState {
    Unlatched,
    PartialLatch,
    Latched,
}

impl Mbc3 {
    fn new(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let rom = data
            .chunks(0x4000)
            .map(<[u8; 0x4000]>::try_from)
            .collect::<Result<_, _>>()?;

        let ram = data
            .chunks(0x2000)
            .map(<[u8; 0x2000]>::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            rom,
            rom_bank: 1,
            ram,
            ram_bank: 0,
            ram_enabled: false,
            rtc_secs: 0,
            rtc_mins: 0,
            rtc_hours: 0,
            rtc_dl: 0,
            rtc_dh: 0,
            latch_state: RtcLatchState::Unlatched,
            last_step_time: Instant::now(),
            background_secs: 0.0,
        })
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[0][usize::from(address)],
            0x4000..=0x7FFF => self.rom[self.rom_bank][usize::from(address - 0x4000)],
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    match self.ram_bank {
                        0x00..=0x03 => self.ram[self.ram_bank][usize::from(address - 0xA000)],
                        0x08 => self.rtc_secs,
                        0x09 => self.rtc_mins,
                        0x0A => self.rtc_hours,
                        0x0B => self.rtc_dl,
                        0x0C => self.rtc_dh,
                        _ => unreachable!(),
                    }
                } else {
                    0xFF
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, address: u16) {
        match address {
            0x0000..=0x1FFF => self.ram_enabled = value != 0,
            0x2000..=0x3FFF => self.rom_bank = if value == 0 { 1 } else { usize::from(value) },
            0x4000..=0x5FFF => self.ram_bank = usize::from(value),
            0x6000..=0x7FFF => self.write_latch(value),
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    match self.ram_bank {
                        0x00..=0x03 => {
                            self.ram[self.ram_bank][usize::from(address - 0xA000)] = value
                        }
                        0x08..=0x0C => {
                            match self.ram_bank {
                                0x08 => self.rtc_secs = value & 0x3F,
                                0x09 => self.rtc_mins = value & 0x3F,
                                0x0A => self.rtc_hours = value & 0x1F,
                                0x0B => self.rtc_dl = value,
                                0x0C => self.rtc_dh = value & 0xC1,
                                _ => unreachable!(),
                            };

                            self.background_secs %= 1.0;
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn step(&mut self) {
        let elapsed_secs = if !self.read_halt() {
            self.last_step_time.elapsed().as_secs_f64()
        } else {
            0.0
        };
        self.background_secs += elapsed_secs;

        if self.background_secs >= 1.0 && !matches!(self.latch_state, RtcLatchState::Latched) {
            let new_secs = u64::from(self.rtc_secs) + (self.background_secs as u64);
            let extra_mins = if self.rtc_secs >= 60 {
                self.rtc_secs = (new_secs & 0x3F) as u8;
                0
            } else {
                self.rtc_secs = (new_secs % 60) as u8;
                new_secs / 60
            };

            let new_mins = u64::from(self.rtc_mins) + extra_mins;
            let extra_hours = if self.rtc_mins >= 60 {
                self.rtc_mins = (new_mins & 0x3F) as u8;
                0
            } else {
                self.rtc_mins = (new_mins % 60) as u8;
                new_mins / 60
            };

            let new_hours = u64::from(self.rtc_hours) + extra_hours;
            let extra_days = if self.rtc_hours >= 24 {
                self.rtc_hours = (new_hours & 0x1F) as u8;
                0
            } else {
                self.rtc_hours = (new_hours % 24) as u8;
                (new_hours / 24) as u16
            };

            let new_days = self.read_day_counter() + extra_days;
            self.write_day_counter(new_days);

            self.background_secs %= 1.0;
        }

        self.last_step_time = Instant::now();
    }

    fn write_latch(&mut self, value: u8) {
        if value == 0 {
            self.latch_state = RtcLatchState::PartialLatch;
        } else {
            if value == 1 && matches!(self.latch_state, RtcLatchState::PartialLatch) {
                self.latch_state = RtcLatchState::Latched;
            } else {
                self.latch_state = RtcLatchState::Unlatched;
            }
        }
    }

    const DAY_COUNTER_MSB_MASK: u8 = 1 << 0;
    const HALT_MASK: u8 = 1 << 6;
    const DAY_COUNTER_CARRY_MASK: u8 = 1 << 7;

    fn read_halt(&self) -> bool {
        (self.rtc_dh & Self::HALT_MASK) != 0
    }

    fn read_day_counter(&self) -> u16 {
        let day_counter_low = self.rtc_dl;
        let day_counter_high = if (self.rtc_dh & Self::DAY_COUNTER_MSB_MASK) != 0 {
            1
        } else {
            0
        };

        u16::from_be_bytes([day_counter_high, day_counter_low])
    }

    fn write_day_counter(&mut self, value: u16) {
        let [day_counter_high, day_counter_low] = value.to_be_bytes();

        self.rtc_dl = day_counter_low;

        // Once set, carry bit remains set until unset by manual write to rtc_dh.
        if (day_counter_high & 0b0000_0010) != 0 {
            self.rtc_dh |= Self::DAY_COUNTER_CARRY_MASK;
        }

        if (day_counter_high & 0b0000_0001) != 0 {
            self.rtc_dh |= Self::DAY_COUNTER_MSB_MASK;
        } else {
            self.rtc_dh &= !Self::DAY_COUNTER_MSB_MASK;
        }
    }
}

impl Cartridge {
    pub fn new(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        assert!(data.len() >= 0x8000);
        let expected_rom_size = match data[0x148] {
            0x00 => 0x008000,
            0x01 => 0x010000,
            0x02 => 0x020000,
            0x03 => 0x040000,
            0x04 => 0x080000,
            0x05 => 0x100000,
            0x06 => 0x200000,
            0x07 => 0x400000,
            0x08 => 0x800000,
            0x52 => 0x120000,
            0x53 => 0x140000,
            0x54 => 0x180000,
            _ => unimplemented!("ROM size value of 0x{:02X}", data[0x148]),
        };

        if data.len() != expected_rom_size {
            return Err(format!(
                "expected rom size of 0x{:06X}, but got 0x{:06X}",
                expected_rom_size,
                data.len()
            )
            .into());
        }

        let ram_size = match data[0x149] {
            0x00 | 0x01 => 0x00000,
            0x02 => 0x02000,
            0x03 => 0x08000,
            0x04 => 0x20000,
            0x05 => 0x10000,
            _ => unreachable!(),
        };

        let mut actual_header_checksum: u8 = 0;
        for byte in data[0x134..=0x14C].iter().copied() {
            actual_header_checksum = actual_header_checksum.wrapping_sub(byte).wrapping_sub(1);
        }
        let expected_header_checksum = data[0x14D];
        if actual_header_checksum != expected_header_checksum {
            return Err(format!(
                "header checksum expected 0x{:02X}, but got 0x{:02X}",
                expected_header_checksum, actual_header_checksum
            )
            .into());
        }

        let mut actual_global_checksum: u16 = 0;
        for (i, byte) in data.iter().copied().enumerate() {
            if i != 0x14E && i != 0x14F {
                actual_global_checksum = actual_global_checksum.wrapping_add(u16::from(byte));
            }
        }
        let expected_global_checksum = u16::from_be_bytes([data[0x14E], data[0x14F]]);
        if actual_global_checksum != expected_global_checksum {
            eprintln!(
                "global checksum expected 0x{:02X}, but got 0x{:02X}",
                expected_global_checksum, actual_global_checksum
            )
        }

        let title: String = data[0x134..=0x143]
            .iter()
            .copied()
            .take_while(|val| *val != 0)
            .map(char::from)
            .collect();

        println!("you are playing: {}", title);

        let cartridge_type_code = data[0x147];
        println!("cartridge type code: ${:02X}", cartridge_type_code);

        let cartridge_impl = match cartridge_type_code {
            0x00 => CartridgeType::NoMbc(NoMbc::new(data, ram_size)?),
            0x01 | 0x02 | 0x03 => CartridgeType::Mbc1(Mbc1::new(data, ram_size)?),
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => CartridgeType::Mbc3(Mbc3::new(data)?),
            _ => todo!(),
        };

        Ok(Cartridge {
            cartridge_type: cartridge_impl,
            title,
        })
    }
}
