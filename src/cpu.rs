use rand::random;
use quicksilver::log::debug;

pub struct CPU {
    pub registers: [u8; 16],
    pub vi: u16,
    pub pc: usize,
    pub memory: [u8; 4096],
    pub stack: [u16; 16],
    pub stack_pointer: usize,
    pub dt: u8,
    pub st: u8,
    pub screen: [bool; 2048],
    pub keyboard: [bool; 16],
    pub waiting_for_input: bool,
}

impl CPU {
    pub fn new() -> CPU {
        let mut cpu = CPU {
            registers: [0; 16],
            vi: 0,
            memory: [0; 4096],
            pc: 0,
            stack: [0; 16],
            stack_pointer: 0,
            dt: 0,
            st: 0,
            screen: [false; 2048],
            keyboard: [false; 16],
            waiting_for_input: false,
        };

        let chars = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for (cursor, line) in chars.iter().enumerate() {
            cpu.memory[cursor] = *line;
        }

        cpu
    }

    pub fn run(&mut self) {
        let op_byte1 = self.memory[self.pc] as u16;
        let op_byte2 = self.memory[self.pc + 1] as u16;
        let opcode = op_byte1 << 8 | op_byte2;

        let x        = ((opcode & 0x0F00) >>  8) as u8;
        let y        = ((opcode & 0x00F0) >>  4) as u8;
        let op_minor =  (opcode & 0x000F) as u8;
        let addr     =   opcode & 0x0FFF;
        let kk       =  (opcode & 0x00FF) as u8;

        if !self.waiting_for_input {
            self.pc += 2;
        }

        match opcode {
            0x00E0 => self.cls(),
            0x00EE => self.ret(),
            0x1000..=0x1FFF => self.jp(addr),
            0x2000..=0x2FFF => self.call(addr),
            0x3000..=0x3FFF => self.se_byte(x, kk),
            0x4000..=0x4FFF => self.sne_byte(x, kk),
            0x5000..=0x5FFF => self.se_xy(x, y),
            0x6000..=0x6FFF => self.ld_byte(x, kk),
            0x7000..=0x7FFF => self.add_byte(x, kk),
            0x8000..=0x8FFF => {
                match op_minor {
                    0x0 => self.ld_xy(x, y),
                    0x1 => self.or_xy(x, y),
                    0x2 => self.and_xy(x, y),
                    0x3 => self.xor_xy(x, y),
                    0x4 => self.add_xy(x, y),
                    0x5 => self.sub_xy(x, y),
                    0x6 => self.shr_x(x),
                    0x7 => self.subn_xy(x, y),
                    0xE => self.shl_x(x),
                    _ => unimplemented!("opcode: {:04x}", opcode),
                }
            },
            0x9000..=0x9FFF => self.sne_xy(x, y),
            0xA000..=0xAFFF => self.ld(addr),
            0xB000..=0xBFFF => self.jp_v0(addr),
            0xC000..=0xCFFF => self.rnd_byte(x, kk),
            0xD000..=0xDFFF => self.drw_k(x, y, op_minor),
            0xE000..=0xEFFF => {
                match (y, op_minor) {
                    (0x9, 0xE) => self.skp_x(x),
                    (0xA, 0x1) => self.sknp_x(x),
                    _ => unimplemented!("opcode {:04x}", opcode),
                }
            },

            0xF000..=0xFFFF => {
                match (y, op_minor) {
                    (0x0, 0x7) => self.ld_x_dt(x),
                    (0x0, 0xA) => self.ld_x_k(x),
                    (0x1, 0x5) => self.ld_dt_x(x),
                    (0x1, 0x8) => self.ld_st_x(x),
                    (0x1, 0xE) => self.add_i_x(x),
                    (0x2, 0x9) => self.ld_f_x(x),
                    (0x3, 0x3) => self.ld_b_x(x),
                    (0x5, 0x5) => self.ld_i_x(x),
                    (0x6, 0x5) => self.ld_x_i(x),
                    _ => unimplemented!("opcode {:04x}", opcode),
                }
            },
            _ => unimplemented!("opcode {:04x}", opcode),
        }
    }

    pub fn update_timers(&mut self) {
        if self.dt > 0 { self.dt -= 1; }
        if self.st > 0 { self.st -= 1; }
    }

    fn cls (&mut self) {
        self.screen = [false; 2048];
        debug!("CLS");
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        self.pc = self.stack[self.stack_pointer] as usize;
        debug!("RET sp {}", self.stack_pointer);
    }

    fn jp(&mut self, addr: u16) {
        self.pc = addr as usize;
        debug!("JP addr {:03x}", addr);
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow")
        }

        stack[sp] = self.pc as u16;
        self.stack_pointer += 1;
        self.pc = addr as usize;
        debug!("CALL addr {:03x}", addr);
    }

    fn se_byte(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
        debug!("SE_BYTE x {:02x} kk {:04x}", x, kk);
        debug!("dt : {}", self.dt);
    }

    fn sne_byte(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
        debug!("SNE_BYTE x {:02x} kk {:04x}", x, kk);
    }

    fn se_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
        debug!("SE_XY x {:02x} y {:02x}", x, y);
    }

    fn ld_byte(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
        debug!("LD_BYTE x {:02x} kk {:04x}", x, kk);
    }

    fn add_byte(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = self.registers[x as usize].wrapping_add(kk);
        debug!("ADD_BYTE x {:02x} kk {:04x}", x, kk);
    }

    fn ld_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize];
        debug!("LD_XY x {:02x} y {:02x}", x, y);
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
        debug!("OR_XY x {:02x} y {:02x}", x, y);
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
        debug!("AND_XY x {:02x} y {:02x}", x, y);
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
        debug!("XOR_XY x {:02x} y {:02x}", x, y);
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let (vx, vy) = (self.registers[x as usize], self.registers[y as usize]);
        let (vx, carry) = vx.overflowing_add(vy);
        self.registers[x as usize] = vx;
        self.registers[0xF as usize] = if carry { 0x01 } else { 0x00 };
        debug!("ADD_XY x {:02x} y {:02x}", x, y);
    }

    fn sub_xy(&mut self, x: u8, y: u8) {
        let (mut vx, vy) = (self.registers[x as usize], self.registers[y as usize]);
        self.registers[0xF as usize] = if vx > vy  { 0x01 } else { 0x00 };
        vx = vx.wrapping_sub(vy);
        self.registers[x as usize] = vx;
        debug!("SUB_XY x {:02x} y {:02x}", x, y);
    }

    fn shr_x(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.registers[0xF as usize] = vx & 0x01;
        self.registers[x as usize] = vx >> 1;
        debug!("SHR_X x {:02x}", x);
    }

    fn subn_xy(&mut self, x: u8, y: u8) {
        let (vx, vy) = (self.registers[x as usize], self.registers[y as usize]);
        self.registers[0xF as usize] = if vy > vx  { 0x01 } else { 0x00 };
        self.registers[x as usize] = vy.wrapping_sub(vx);
        debug!("SUBN_XY x {:02x} y {:02x}", x, y);
    }

    fn shl_x(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        self.registers[0xF as usize] = (vx & 0x80) >> 7;
        self.registers[x as usize] = vx << 1;
        debug!("SHL_X x {:02x}", x);
    }

    fn sne_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
        debug!("SNE_XY x {:02x} y {:02x}", x, y);
    }

    fn ld(&mut self, addr: u16) {
        self.vi = addr;
        debug!("LD addr {:03x}", addr);
    }

    fn jp_v0(&mut self, addr: u16) {
        self.pc = (addr.wrapping_add(self.registers[0x00] as u16)) as usize;
        debug!("JP_V0 addr {:03x}", addr);
    }

    fn rnd_byte(&mut self, x: u8, kk: u8) {
        let rnd: u8 = random();
        self.registers[x as usize] = rnd & kk;
        debug!("RND_BYTE x {:02x} kk {:04x}", x, kk);
    }

    fn drw_k(&mut self, x: u8, y: u8, k: u8) {
        self.registers[0xF as usize] = 0;
        let (vx, vy) = (self.registers[x as usize], self.registers[y as usize]);
        for i in 0..k {
            let byte = self.memory[self.vi as usize + i as usize];
            for j in 0..8 {
                let cur_screen = (vx + j) as usize % 64 + ((vy + i) % 32) as usize * 64;
                let existing = self.screen[cur_screen];
                let cur_bit = (byte >> (7 - j)) & 0x01 == 1;
                if cur_bit && existing {
                    self.registers[0xF as usize] = 1;
                }
                self.screen[cur_screen] = existing ^ cur_bit;
            }
        }
        debug!("DRW_K x {:02x} y {:02x} k {:02x}", x, y, k);
    }

    fn skp_x(&mut self, x: u8) {
        if self.keyboard[self.registers[x as usize] as usize] {
            self.pc += 2;
        }
        debug!("SKP_X x {:02x}", x);
    }

    fn sknp_x(&mut self, x: u8) {
        if !self.keyboard[self.registers[x as usize] as usize] {
            self.pc += 2;
        }
        debug!("SKNP_X x {:02x}", x);
    }

    fn ld_x_dt(&mut self, x: u8) {
        self.registers[x as usize] = self.dt;
        debug!("LD_X_DT x {:02x}", x);
    }

    fn ld_x_k(&mut self, x: u8) {
        self.waiting_for_input = true;

        let mut key_pressed: Option<u8> = None;
        for (index, key) in self.keyboard.iter().enumerate() {
            if *key {
                key_pressed = Some(index as u8)
            }
        }

        if key_pressed.is_some() {
            self.waiting_for_input = false;
            self.registers[x as usize] = key_pressed.unwrap();
        }
        debug!("LD_X_K x {:02x}", x);
    }

    fn ld_dt_x(&mut self, x: u8) {
        self.dt = self.registers[x as usize];
        debug!("LD_DT_X dt {:02x} x {:02x}", self.dt, x);
    }

    fn ld_st_x(&mut self, x: u8) {
        self.st = self.registers[x as usize];
        debug!("LD_ST_X st {:02x} x {:02x}", self.st, x);
    }

    fn add_i_x(&mut self, x: u8) {
        self.vi = self.vi.wrapping_add(self.registers[x as usize] as u16);
        debug!("ADD_I_X vi {:02x} x {:02x}", self.vi, x);
    }

    fn ld_f_x(&mut self, x: u8) {
        self.vi = (self.registers[x as usize] * 5) as u16;
        debug!("LD_F_X x {:02x}", x);
    }

    fn ld_b_x(&mut self, x: u8) {
        let vx = self.registers[x as usize];
        let hundreds = vx / 100;
        let tens = (vx - hundreds * 100) / 10;
        let units = vx - hundreds * 100 - tens * 10;
        self.memory[self.vi as usize] = hundreds;
        self.memory[self.vi as usize + 1] = tens;
        self.memory[self.vi as usize + 2] = units;
        debug!("LD_B_X x {:02x}", x);
    }

    fn ld_i_x(&mut self, x: u8) {
        for i in 0..=x {
            self.memory[(self.vi + i as u16) as usize] = self.registers[i as usize];
        }
        debug!("LD_I_X x {:02x}", x);
    }

    fn ld_x_i(&mut self, x: u8) {
        for i in 0..=x {
            self.registers[i as usize] = self.memory[(self.vi + i as u16) as usize];
        }
        debug!("LD_X_I x {:02x}", x);
    }
}
