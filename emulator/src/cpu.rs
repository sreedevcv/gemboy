use std::f32::consts::E;

use crate::memory::{LinearMemory, MemoryInterface};

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
    fn set_reg8(&mut self, reg_index: u8, value: u8) {
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
            _ => panic!("Unexpected register index({reg_index}) in CPU::set_reg8"),
        }
    }

    // Gets the register value
    fn get_reg8(&self, reg_index: u8) -> u8 {
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
            _ => panic!("Unexpected register index({reg_index}) in CPU::get_reg8"),
        }
    }

    fn set_reg16(&mut self, reg_index: u8, value: u16) {
        assert!(reg_index < 4);

        match reg_index {
            0 => {
                self.b = ((value & 0xFF00) >> 8u8) as u8;
                self.c = (value & 0x00FF) as u8;
            }
            1 => {
                self.d = ((value & 0xFF00) >> 8u8) as u8;
                self.e = (value & 0x00FF) as u8;
            }
            2 => {
                self.h = ((value & 0xFF00) >> 8u8) as u8;
                self.l = (value & 0x00FF) as u8;
            }
            3 => {
                self.sp = value;
            }
            _ => {
                panic!("Unexpected register index({reg_index}) in CPU::set_reg16")
            }
        }
    }

    fn get_reg16(&mut self, reg_index: u8) -> u16 {
        assert!(reg_index < 4);

        match reg_index {
            0 => ((self.b as u16) << 8u8) | (self.c as u16),
            1 => ((self.d as u16) << 8u8) | (self.e as u16),
            2 => ((self.h as u16) << 8u8) | (self.l as u16),
            3 => self.sp,
            _ => {
                panic!("Unexpected register index({reg_index}) in CPU::get_reg16")
            }
        }
    }

    fn read(&mut self, memory: &impl MemoryInterface) -> u8 {
        let byte = memory.get_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn write(&mut self, memory: &mut impl MemoryInterface, addr: u16, byte: u8) {
        memory.set_byte(addr, byte);
        // No need to increment pc;
    }

    fn ld_reg8_to_reg8(&mut self, opcode: u8) {
        /*  ld r8, r8
            bit  7 = 0
            bit  6 = 1
            bits 543 = dest reg8
            bits 210 = sour reg8
            bytes  1
            cycles 1
        */
        let source_reg = opcode & 0x07;
        let dest_reg = (opcode & 0x38) >> 3u8;
        let reg_value = self.get_reg8(source_reg);
        self.set_reg8(dest_reg, reg_value);
        println!("LD r{dest_reg}, r{source_reg}");
    }

    fn ld_imm8_to_reg8(&mut self, opcode: u8, memory: &impl MemoryInterface) {
        /*  ld r8, imm8
            bits 76 = 00
            bits 543 = dest reg8
            bits 210 = 110
            bytes  2
            cycles 2
        */
        let dest_reg = (opcode & 0x38) >> 3u8;
        let imm8 = self.read(memory);
        self.set_reg8(dest_reg, imm8);
        println!("LD r{dest_reg}, {imm8}");
    }

    fn ld_imm16_to_reg16(&mut self, opcode: u8, memory: &impl MemoryInterface) {
        /*  ld r16, imm16
            bits 76 = 00
            bits 54 = dest reg16
            bits 3210 = 0001
            bytes  3
            cycles 3
        */

        let dest_reg = (opcode & 0x30) >> 4u8;
        let byte1 = self.read(memory);
        let byte2 = self.read(memory);
        let imm16 = ((byte2 as u16) << 8u8) | (byte1 as u16);
        self.set_reg16(dest_reg, imm16);
        println!("LD r16{dest_reg}, #{imm16}");
    }

    fn ld_between_hl_and_r8(&mut self, opcode: u8, memory: &mut impl MemoryInterface) {
        /*  ld (r8|[hl]), (r8|[hl])
            bit  7 = 0
            bit  6 = 1
            bits 543 = dest reg8
            bits 210 = sour reg8
            bytes  1
            cycles 2
        */
        let source_reg = opcode & 0x07;
        let dest_reg = (opcode & 0x38) >> 3u8;
        let addr = ((self.h as u16) << 8u8) | (self.l as u16);

        if source_reg == 6 {
            self.set_reg8(dest_reg, memory.get_byte(addr));
            println!("LD r{dest_reg} [hl]");
        } else {
            self.write(memory, addr, self.get_reg8(source_reg));
            println!("LD [hl] r{source_reg}");
        }
    }

    fn ld_imm8_to_hl(&mut self, opcode: u8, memory: &mut impl MemoryInterface) {
        /*  ld [hl], imm8
            opcode = 00_110_110
            bytes  2
            cycles 3
        */
        let imm8 = self.read(memory);
        let addr = ((self.h as u16) << 8u8) | (self.l as u16);
        self.write(memory, addr, imm8);
    }

    fn ld_r16_mem_to_a(&mut self, opcode: u8, memory: &mut impl MemoryInterface) {
        /*  ld a, [r16]
            bit  7 = 0
            bit  6 = 0
            bits 54 = source reg16
            bits 3210 = 1010
            bytes  1
            cycles 2
        */

        let source_reg = (opcode & 0x30) >> 4u8;
        self.a = memory.get_byte(self.get_reg16(source_reg));
    }

    fn ld_a_to_r16_mem(&mut self, opcode: u8, memory: &mut impl MemoryInterface) {
        /*  ld [r16], a
            bit  7 = 0
            bit  6 = 0
            bits 54 = dest reg16
            bits 3210 = 0010
            bytes  1
            cycles 2
        */
        let source_reg = (opcode & 0x30) >> 4u8;
        memory.set_byte(self.get_reg16(source_reg), self.a);
    }
}

pub fn decode(cpu: &mut CPU, memory: &mut impl MemoryInterface) {
    let opcode = cpu.read(memory);

    if opcode == 0u8 {
        (); // Nop
    } else if opcode == 0b0011_0110 {
        cpu.ld_imm8_to_hl(opcode, memory);
    } else if opcode & 0xC0 == 0x40 {
        // The case where one of the operands is not [hl]
        if (opcode & 0x07 != 0x06) && ((opcode & 0x38) >> 3u8 != 0x06) {
            cpu.ld_reg8_to_reg8(opcode);
        } else {
            cpu.ld_between_hl_and_r8(opcode, memory);
        }
    } else if ((opcode & 0xC0) >> 6 == 0) && (opcode & 0x07 == 0x06) {
        cpu.ld_imm8_to_reg8(opcode, memory);
    } else if ((opcode & 0xC0) >> 6 == 0) && (opcode & 0x0F == 0x1) {
        cpu.ld_imm16_to_reg16(opcode, memory);
    } else if (opcode & 0xC0 == 0) && (opcode & 0x0F == 0xA) {
        cpu.ld_r16_mem_to_a(opcode, memory);
    } else if (opcode & 0xC0 == 0) && (opcode & 0x0F == 0x2) {
        cpu.ld_a_to_r16_mem(opcode, memory);
    } else {
        panic!("Unexpected instruction {opcode:0>8b} {opcode:0>2x}");
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    #[test]
    fn decode_ld_reg8_to_reg8() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

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

            let byte: u8 = 0b0100_0000 | (dest_reg << 3u8) | sour_reg;
            cpu.pc = 0;
            memory.set_byte(0, byte);
            decode(&mut cpu, &mut memory);

            let value1 = cpu.get_reg8(dest_reg);
            let value2 = cpu.get_reg8(sour_reg);
            assert_eq!(value1, value2);
        }
    }

    #[test]
    fn decode_ld_imm8_to_reg8() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        for reg in 0u8..8u8 {
            if reg == 6 {
                continue;
            }

            let byte: u8 = 0b0000_0110 | (reg << 3u8);
            let data = rand::rng().random_range(0..255);
            cpu.pc = 0;
            memory.set_byte(0, byte);
            memory.set_byte(1, data);
            decode(&mut cpu, &mut memory);

            let reg_value = cpu.get_reg8(reg);
            assert_eq!(reg_value, data);
        }
    }

    #[test]
    fn decode_ld_imm16_to_reg16() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        for reg16 in 0..4 {
            let byte: u8 = 0b0000_0001 | (reg16 << 4u8);
            let data1 = rand::rng().random_range(0..255);
            let data2 = rand::rng().random_range(0..255);
            cpu.pc = 0;
            memory.set_byte(0, byte);
            memory.set_byte(1, data1);
            memory.set_byte(2, data2);
            decode(&mut cpu, &mut memory);

            let reg_value = cpu.get_reg16(reg16);
            assert_eq!(reg_value, (data2 as u16) << 8u8 | (data1 as u16));
        }
    }

    #[test]
    fn decode_ld_between_hl_and_r8() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        let h_value = rand::rng().random_range(0..255u8);
        let l_value = rand::rng().random_range(0..255u8);
        let data = rand::rng().random_range(0..255u8);

        cpu.h = h_value;
        cpu.l = l_value;
        cpu.pc = 0;
        let addr = ((h_value as u16) << 8u8) | (l_value as u16);
        cpu.c = data;

        // ld [hl], c
        let opcode1 = 0b0100_0000 | 0x70 | 0x1;
        memory.set_byte(0, opcode1);
        decode(&mut cpu, &mut memory);

        assert_eq!(memory.get_byte(addr), data);

        // ld d, [hl]
        let opcode2 = 0b0100_0000 | 0x20 | 0x06;
        memory.set_byte(0, opcode2);
        cpu.pc = 0;
        cpu.d = data;
        decode(&mut cpu, &mut memory);

        assert_eq!(memory.get_byte(addr), cpu.d)
    }

    #[test]
    fn decode_ld_imm8_to_hl() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        let h_value = rand::rng().random_range(0..255u8);
        let l_value = rand::rng().random_range(0..255u8);
        let imm8 = rand::rng().random_range(0..255u8);

        cpu.h = h_value;
        cpu.l = l_value;
        cpu.pc = 0;
        let addr = ((h_value as u16) << 8u8) | (l_value as u16);

        let opcode: u8 = 0b0011_0110;
        memory.set_byte(0, opcode);
        memory.set_byte(1, imm8);

        decode(&mut cpu, &mut memory);

        let byte_in_memory = memory.get_byte(addr);
        assert_eq!(byte_in_memory, imm8);
    }

    #[test]
    fn decode_ld_r16_mem_to_a() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        let addr1 = rand::rng().random_range(0..((255 * 255) as u16));
        let reg_value1 = rand::rng().random_range(0..255u8);

        let b_reg = ((addr1 & 0xFF00) >> 0x8) as u8;
        let c_reg = (addr1 & 0x00FF) as u8;

        // ld a, [bc]
        let opcode = 0b0000_1010;

        memory.set_byte(0, opcode);
        memory.set_byte(addr1, reg_value1);
        cpu.pc = 0;
        cpu.b = b_reg;
        cpu.c = c_reg;

        decode(&mut cpu, &mut memory);

        assert_eq!(reg_value1, cpu.a);

        let addr2 = rand::rng().random_range(0..((255 * 255) as u16));
        let reg_value2 = rand::rng().random_range(0..255u8);

        let d_reg = ((addr2 & 0xFF00) >> 0x8) as u8;
        let e_reg = (addr2 & 0x00FF) as u8;

        // ld a, [de]
        let opcode2 = 0b0000_1010 | 0x10;

        cpu.pc = 0;
        cpu.d = d_reg;
        cpu.e = e_reg;
        memory.set_byte(0, opcode2);
        memory.set_byte(addr2, reg_value2);

        decode(&mut cpu, &mut memory);

        assert_eq!(reg_value2, cpu.a);
    }

    #[test]
    fn decode_ld_a_to_r16_mem() {
        let mut cpu = CPU::new();
        let mut memory = LinearMemory::new();

        let addr1 = rand::rng().random_range(0..((255 * 255) as u16));
        let reg_value1 = rand::rng().random_range(0..255u8);

        let b_reg = ((addr1 & 0xFF00) >> 0x8) as u8;
        let c_reg = (addr1 & 0x00FF) as u8;

        // ld [bc], a
        let opcode = 0b0000_0010;

        memory.set_byte(0, opcode);
        cpu.pc = 0;
        cpu.a = reg_value1;
        cpu.b = b_reg;
        cpu.c = c_reg;

        decode(&mut cpu, &mut memory);

        assert_eq!(reg_value1, memory.get_byte(addr1));

        let addr2 = rand::rng().random_range(0..((255 * 255) as u16));
        let reg_value2 = rand::rng().random_range(0..255u8);

        let d_reg = ((addr2 & 0xFF00) >> 0x8) as u8;
        let e_reg = (addr2 & 0x00FF) as u8;

        // ld [de], a
        let opcode2 = 0b0000_0010 | 0x10;

        cpu.pc = 0;
        cpu.a = reg_value2;
        cpu.d = d_reg;
        cpu.e = e_reg;
        memory.set_byte(0, opcode2);

        decode(&mut cpu, &mut memory);

        assert_eq!(reg_value2, memory.get_byte(addr2));
    }
}
