#[derive(Default, Clone)]
pub struct Serial {
    data_written: String,
}

impl Serial {
    pub fn write_byte(&mut self, byte_written: u8) {
        let char_written = char::from(byte_written);
        self.data_written.push(char_written);

        #[cfg(test)]
        {
            print!("{}", char_written);
        }
    }

    pub fn get_data_written(&self) -> &str {
        self.data_written.as_str()
    }
}
