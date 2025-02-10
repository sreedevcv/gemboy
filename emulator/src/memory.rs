pub trait MemoryInterface {
    fn set_byte(&mut self, addr: u16, byte: u8);
    fn get_byte(&self, addr: u16) -> u8;
}

pub struct LinearMemory {
    data: Vec<u8>,
}

impl LinearMemory {
    pub fn new() -> Self {
        let data = vec![0u8; 1 << 16];

        LinearMemory { data }
    }
}

impl MemoryInterface for LinearMemory {
    fn set_byte(&mut self, addr: u16, byte: u8) {
        self.data[addr as usize] = byte;
    }

    fn get_byte(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
}
