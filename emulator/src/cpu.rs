#[derive(Debug)]
pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    fn zero_flag(&self) -> bool {
        (self.f & 0x01) != 0
    }

    fn subtraction_flag(&self) -> bool {
        (self.f & 0x02) != 0
    }

    fn half_carry_flag(&self) -> bool {
        (self.f & 0x04) != 0
    }

    fn carry_flag(&self) -> bool {
        (self.f & 0x08) != 0
    }

    // Sets the register to value
    fn set_reg(&mut self, reg_index: u8, value: u8) {
        assert!(reg_index < 8);

        match reg_index {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            // 6 =>
            7 => self.a = value,
            _ => panic!("Unexpected register index({reg_index}) in CPU::set_reg"),
        }
    }

    // Gets the register value
    fn get_reg_value(&self, reg_index: u8) -> u8 {
        assert!(reg_index < 8);

        match reg_index {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            // 6 =>
            7 => self.a,
            _ => panic!("Unexpected register index({reg_index}) in CPU::set_reg"),
        }
    }
}

pub fn decode(cpu: &mut CPU, byte: u8) {
    /*  ld r8, r8
        bit  7 = 0
        bit  6 = 1
        bits 543 = dest
        bits 210 = sour
    */
    if byte & 0xC0 == 0x40 {
        let source_reg = byte & 0x07;
        let dest_reg = (byte & 0x38) >> 3u8;
        let reg_value = cpu.get_reg_value(source_reg);
        cpu.set_reg(dest_reg, reg_value);
    } else {
        panic!("Unexpected instruction");
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    #[test]
    fn decode_ld_reg_to_reg() {
        // TODO::Add Checks for register index 6 also

        let mut cpu = CPU::new();

        cpu.a = rand::rng().random_range(0..255);
        cpu.b = rand::rng().random_range(0..255);
        cpu.c = rand::rng().random_range(0..255);
        cpu.d = rand::rng().random_range(0..255);
        cpu.e = rand::rng().random_range(0..255);
        cpu.h = rand::rng().random_range(0..255);
        cpu.l = rand::rng().random_range(0..255);

        for _ in 0..10 {
            let dest_reg: u8 = rand::rng().random_range(0..8);
            let sour_reg: u8 = rand::rng().random_range(0..8);

            if dest_reg == 6 || sour_reg == 6 {
                continue;
            }

            let byte = 0b0100_0000 | (dest_reg << 3u8) | sour_reg;
            decode(&mut cpu, byte);

            let value1 = cpu.get_reg_value(dest_reg);
            let value2 = cpu.get_reg_value(sour_reg);
            assert_eq!(value1, value2);
        }
    }
}
