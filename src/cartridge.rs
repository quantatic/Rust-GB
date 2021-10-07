use std::{
    convert::{TryFrom, TryInto},
    error::Error,
};

pub enum Cartridge {
    NoMbc(NoMbc),
    Mbc1(Mbc1),
    Mbc3(Mbc3),
}

impl Cartridge {
    pub fn read(&self, offset: u16) -> u8 {
        match self {
            Cartridge::NoMbc(no_mbc) => no_mbc.read(offset),
            Cartridge::Mbc1(mbc_1) => mbc_1.read(offset),
            Cartridge::Mbc3(mbc_3) => mbc_3.read(offset),
        }
    }

    pub fn write(&mut self, value: u8, offset: u16) {
        match self {
            Cartridge::NoMbc(no_mbc) => no_mbc.write(value, offset),
            Cartridge::Mbc1(mbc_1) => mbc_1.write(value, offset),
            Cartridge::Mbc3(mbc_3) => mbc_3.write(value, offset),
        }
    }
}

struct NoMbc {
    rom: Vec<u8>,
    ram: Box<[u8; 0x2000]>,
}

impl NoMbc {
    fn new(data: &[u8]) -> Self {
        Self {
            rom: data.to_vec(),
            ram: Box::new([0; 0x2000]),
        }
    }

    fn read(&self, offset: u16) -> u8 {
        match offset {
            0x0000..=0x7FFF => self.rom[usize::from(offset)],
            0xA000..=0xBFFF => self.ram[usize::from(offset - 0xA000)],
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, offset: u16) {
        match offset {
            0x0000..=0x7FFF => self.rom[usize::from(offset)] = value,
            0xA000..=0xBFFF => self.ram[usize::from(offset - 0xA000)] = value,
            _ => unreachable!(),
        };
    }
}

struct Mbc1 {
    rom: Vec<[u8; 0x4000]>,
    rom_bank: usize,
    ram: Vec<[u8; 0x2000]>,
    ram_bank: usize,
}

impl Mbc1 {
    fn new(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let rom = data
            .chunks(0x4000)
            .map(|chunk| chunk.try_into())
            .collect::<Result<_, _>>()?;

        let ram = data
            .chunks(0x2000)
            .map(|chunk| chunk.try_into())
            .collect::<Result<_, _>>()?;

        Ok(Self {
            rom,
            rom_bank: 1,
            ram,
            ram_bank: 0,
        })
    }

    fn read(&self, offset: u16) -> u8 {
        match offset {
            0x0000..=0x3FFF => self.rom[0][usize::from(offset)],
            0x4000..=0x7FFF => self.rom[self.rom_bank][usize::from(offset - 0x4000)],
            0xA000..=0xBFFF => self.ram[self.ram_bank][usize::from(offset - 0xA000)],
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, offset: u16) {
        match offset {
            0x0000..=0x1FFF => {} // RAM enable
            0x2000..=0x3FFF => {
                self.rom_bank = if value == 0 {
                    1
                } else {
                    usize::from(value) % self.rom.len()
                }
            }
            0x4000..=0x5FFF => self.ram_bank = usize::from(value & 0b11),
            0x6000..=0x7FFF => todo!("banking mode select"),
            0xA000..=0xBFFF => self.ram[self.ram_bank][usize::from(offset - 0xA000)] = value,
            _ => unreachable!(),
        }
    }
}

struct Mbc3 {
    rom: Vec<[u8; 0x4000]>,
    rom_bank: usize,
    ram: Vec<[u8; 0x2000]>,
    ram_bank: usize,
    rtc_secs: u8,
    rtc_mins: u8,
    rtc_hours: u8,
    rtc_dl: u8,
    rtc_dh: u8,
}

impl Mbc3 {
    fn new(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let rom = data
            .chunks(0x4000)
            .map(|chunk| chunk.try_into())
            .collect::<Result<_, _>>()?;

        let ram = data
            .chunks(0x2000)
            .map(|chunk| chunk.try_into())
            .collect::<Result<_, _>>()?;

        Ok(Self {
            rom,
            rom_bank: 1,
            ram,
            ram_bank: 0,
            rtc_secs: 0,
            rtc_mins: 0,
            rtc_hours: 0,
            rtc_dl: 0,
            rtc_dh: 0,
        })
    }

    fn read(&self, offset: u16) -> u8 {
        match offset {
            0x0000..=0x3FFF => self.rom[0][usize::from(offset)],
            0x4000..=0x7FFF => self.rom[self.rom_bank][usize::from(offset - 0x4000)],
            0xA000..=0xBFFF => match self.ram_bank {
                0x00..=0x03 => self.ram[self.ram_bank][usize::from(offset - 0xA000)],
                0x08 => self.rtc_secs,
                0x09 => self.rtc_mins,
                0x0A => self.rtc_hours,
                0x0B => self.rtc_dl,
                0x0C => self.rtc_dh,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn write(&mut self, value: u8, offset: u16) {
        match offset {
            0x0000..=0x1FFF => {} // RAM and timer enable
            0x2000..=0x3FFF => self.rom_bank = if value == 0 { 1 } else { usize::from(value) },
            0x4000..=0x5FFF => self.ram_bank = usize::from(value),
            0x6000..=0x7FFF => todo!("latch clock data"),
            0xA000..=0xBFFF => match self.ram_bank {
                0x00..=0x03 => self.ram[self.ram_bank][usize::from(offset - 0xA000)] = value,
                0x08 => self.rtc_secs = value,
                0x09 => self.rtc_mins = value,
                0x0A => self.rtc_hours = value,
                0x0B => self.rtc_dl = value,
                0x0C => self.rtc_dh = value,
                _ => unreachable!(),
            },
            _ => unreachable!(),
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
            return Err(format!(
                "global checksum expected 0x{:02X}, but got 0x{:02X}",
                expected_global_checksum, actual_global_checksum
            )
            .into());
        }

        let title: String = data[0x134..=0x143]
            .iter()
            .copied()
            .map(char::from)
            .collect();

        println!("you are playing: {}", title);

        let cartridge_type_code = data[0x147];
        println!("cartridge type code: ${:02X}", cartridge_type_code);

        match cartridge_type_code {
            0x00 => Ok(Cartridge::NoMbc(NoMbc::new(data))),
            0x01 | 0x02 | 0x03 => Ok(Cartridge::Mbc1(Mbc1::new(data)?)),
            0x11 | 0x12 | 0x13 => Ok(Cartridge::Mbc3(Mbc3::new(data)?)),
            _ => todo!(),
        }
    }
}
