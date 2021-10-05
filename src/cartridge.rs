use std::{convert::TryInto, error::Error, process::exit};

pub struct Cartridge {
    banks: Vec<[u8; 0x4000]>,
    selected_bank: u8,
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

        let mut result = Self {
            banks: Vec::new(),
            selected_bank: 1,
        };

        for bank_idx in 0..(data.len() / 0x4000) {
            let bank_data = data[bank_idx * 0x4000..][..0x4000].try_into().unwrap();
            result.banks.push(bank_data);
        }

        Ok(result)
    }
}

impl Cartridge {
    pub fn read_rom(&self, offset: u16) -> u8 {
        match offset {
            0x0000..=0x3FFF => self.banks[0][usize::from(offset)],
            0x4000..=0x7FFF => {
                self.banks[usize::from(self.selected_bank)][usize::from(offset - 0x4000)]
            }
            _ => unimplemented!(),
        }
    }

    pub fn write_rom(&mut self, value: u8, offset: u16) {
        match offset {
            0x2000..=0x3FFF => self.selected_bank = value & 0b0001_1111,
            _ => todo!(),
        }
    }
}
