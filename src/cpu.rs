#![allow(non_snake_case)]
use crate::font::FONT;
use rand::Rng;
enum Flow {
    Next,
    Skip,
    Jump(usize)
}

pub struct CpuOutput {
    pub screen: Option<[[u8; 64]; 32]>,
    pub sound: bool
}
pub struct Cpu {
    ram: [u8; 4096],
    stack: [usize; 16],
    i: usize,
    pc: usize,
    sp: usize,
    dt: u8,
    st: u8,
    v: [u8; 16],
    keys: [bool; 16],
    paused: bool,
    keyv: u8,
    vram: [[u8; 64]; 32],
    screen_refresh: bool
}

impl Flow {
    fn skip_if(condition: bool) -> Flow {
        if condition {
            Flow::Skip
        } else {
            Flow::Next
        }
    }
}

impl Cpu {
    pub fn new() -> Self {
        let mut ram = [0u8; 4096];
        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        Cpu {
            ram: ram,
            stack: [0; 16],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            dt: 0,
            st: 0,
            keys: [false; 16],
            paused: false,
            keyv: 0,
            vram: [[0; 64]; 32],
            screen_refresh: false
        }
    }

    pub fn load(&mut self, data: &Vec<u8>) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < 4096 {
                self.ram[0x200 +i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn tick(&mut self, keys: [bool; 16]) -> CpuOutput {
        self.keys = keys;
        self.screen_refresh = false;
        if self.dt > 0 {
            self.dt -= 1
        }
        if self.st > 0 {
            self.st -=1
        }

        if self.paused {
            for i in 0..keys.len() {
                if keys[i] {
                    self.paused = false;
                    self.v[self.keyv as usize] = i as u8;
                    break;
                }
            }
        } else {
            self.execute(self.fetch());
        }
        let mut output = CpuOutput {
            screen: None,
            sound: (self.st > 0)
        };
        if self.screen_refresh {
            output.screen = Some(self.vram);
        };
        return output;
    }

    fn fetch(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc +1] as u16)
    }

    fn execute(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8
        );
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        let next = match nibbles {
            (0x0, 0x0, 0xE, 0x0) => self.CLS(),
            (0x0, 0x0, 0xE, 0xE) => self.RET(),
            (0x1, _, _, _)       => self.JP(nnn),
            (0x2, _, _, _)       => self.CALL(nnn),
            (0x3, _, _, _)       => self.SE(x, kk),
            (0x4, _, _, _)       => self.SNE(x, kk),
            (0x5, _, _, _)       => self.SEV(x, y),
            (0x6, _, _, _)       => self.LD(x, kk),
            (0x7, _, _, _)       => self.ADD(x, kk),
            (0x8, _, _, 0x0)     => self.LDV(x, y),
            (0x8, _, _, 0x1)     => self.OR(x, y),
            (0x8, _, _, 0x2)     => self.AND(x, y),
            (0x8, _, _, 0x3)     => self.XOR(x, y),
            (0x8, _, _, 0x4)     => self.ADDV(x, y),
            (0x8, _, _, 0x5)     => self.SUB(x, y),
            (0x8, _, _, 0x6)     => self.SHR(x),
            (0x8, _, _, 0x7)     => self.SUBN(x, y),
            (0x8, _, _, 0xE)     => self.SHL(x),
            (0x9, _, _, 0x0)     => self.SNEV(x, y),
            (0xA, _, _, _)       => self.LDI(nnn),
            (0xB, _, _, _)       => self.JPV(nnn),
            (0xC, _, _, _)       => self.RAND(x, kk),
            (0xD, _, _, _)       => self.DRAW(x, y, n),
            (0xE, _, 0x9, 0xE)   => self.SKP(x),
            (0xE, _, 0xA, 0x1)   => self.SKNP(x),
            (0xF, _, 0x0, 0x7)   => self.LDVDT(x),
            (0xF, _, 0x0, 0xA)   => self.PAUSE(x),
            (0xF, _, 0x1, 0x5)   => self.LDDTV(x),
            (0xF, _, 0x1, 0x8)   => self.LDST(x),
            (0xF, _, 0x1, 0xE)   => self.ADDI(x),
            (0xF, _, 0x2, 0x9)   => self.LDF(x),
            (0xF, _, 0x3, 0x3)   => self.LDB(x),
            (0xF, _, 0x5, 0x5)   => self.LDIV(x),
            (0xF, _, 0x6, 0x5)   => self.LDVI(x),
            _                    => self.unknown_opcode(opcode)
        };

        match next {
            Flow::Next => self.pc += 2,
            Flow::Skip => self.pc += 4,
            Flow::Jump(addr) => self.pc = addr
        }
    }

    fn CLS(&mut self) -> Flow {
        for y in 0..32 {
            for x in 0..64 {
                self.vram[y][x] = 0;
            }
        }
        self.screen_refresh = true;
        Flow::Next
    }

    fn RET(&mut self) -> Flow {
        self.sp -= 1;
        Flow::Jump(self.stack[self.sp])
    }

    fn JP(&mut self, nnn: usize) -> Flow {
        Flow::Jump(nnn)
    }

    fn CALL(&mut self, nnn: usize) -> Flow {
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        Flow::Jump(nnn)
    }

    fn SE(&mut self, x: usize, kk: u8) -> Flow {
        Flow::skip_if(self.v[x] == kk)
    }

    fn SNE(&mut self, x: usize, kk: u8) -> Flow {
        Flow::skip_if(self.v[x] != kk)
    }

    fn SEV(&mut self, x: usize, y: usize) -> Flow {
        Flow::skip_if(self.v[x] == self.v[y])
    }

    fn LD(&mut self, x: usize, kk: u8) -> Flow {
        self.v[x] = kk;
        Flow::Next
    }

    fn ADD(&mut self, x: usize, kk: u8) -> Flow {
        let vx = self.v[x] as u16;
        let val = kk as u16;
        let result = vx + val;
        self.v[x] = result as u8;
        Flow::Next
    }

    fn LDV(&mut self, x: usize, y: usize) -> Flow {
        self.v[x] = self.v[y];
        Flow::Next
    }

    fn OR(&mut self, x: usize, y: usize) -> Flow {
        self.v[x] |= self.v[y];
        Flow::Next
    }

    fn AND(&mut self, x: usize, y: usize) -> Flow {
        self.v[x] &= self.v[y];
        Flow::Next
    }

    fn XOR(&mut self, x: usize, y: usize) -> Flow {
        self.v[x] ^= self.v[y];
        Flow::Next
    }

    fn ADDV(&mut self, x: usize, y: usize) -> Flow {
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let result = vx + vy;
        self.v[x] = result as u8;
        self.v[0xf] = if result > 0xFF {1} else {0};
        Flow::Next
    }

    fn SUB(&mut self, x: usize, y: usize) -> Flow {
        self.v[0xF] = if self.v[x] > self.v[y] {1} else {0};
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        Flow::Next
    }

    fn SHR(&mut self, x: usize) -> Flow {
        self.v[0xF] = self.v[x] & 1;
        self.v[x] >>= 1;
        Flow::Next
    }

    fn SUBN(&mut self, x: usize, y: usize) -> Flow {
        self.v[0xF] = if self.v[y] > self.v[x] {1} else {0};
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        Flow::Next
    }

    fn SHL(&mut self, x: usize) -> Flow {
        self.v[0xF] = (self.v[x] &0b10000000) >> 7;
        self.v[x] <<= 1;
        Flow::Next
    }

    fn SNEV(&mut self, x: usize, y: usize) -> Flow {
        Flow::skip_if(self.v[x] != self.v[y])
    }

    fn LDI(&mut self, nnn: usize) -> Flow {
        self.i = nnn;
        Flow::Next
    }

    fn JPV(&mut self, nnn: usize) -> Flow {
        Flow::Jump((self.v[0] as usize) + nnn)
    }

    fn RAND(&mut self, x: usize, kk: u8) -> Flow {
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
        Flow::Next
    }

    fn DRAW(&mut self, x: usize, y: usize, n: usize) -> Flow {
        self.v[0xF] = 0;
        for byte in 0..n {
            let y = (self.v[y] as usize + byte) % 32;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % 64;
                let color = (self.ram[self.i + byte] >> (7 - bit)) & 1;
                self.v[0xF] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }
        self.screen_refresh = true;
        Flow::Next
    }

    fn SKP(&mut self, x: usize) -> Flow {
        Flow::skip_if(self.keys[self.v[x] as usize])
    }

    fn SKNP(&mut self, x: usize) -> Flow {
        Flow::skip_if(!self.keys[self.v[x] as usize])
    }

    fn LDVDT(&mut self, x: usize) -> Flow {
        self.v[x] = self.dt;
        Flow::Next
    }

    fn PAUSE(&mut self, x: usize) -> Flow {
        self.paused = true;
        self.keyv = x as u8;
        Flow::Next
    }

    fn LDDTV(&mut self, x: usize) -> Flow {
        self.dt = self.v[x];
        Flow::Next
    }

    fn LDST(&mut self, x: usize) -> Flow {
        self.st = self.v[x];
        Flow::Next
    }

    fn ADDI(&mut self, x: usize) -> Flow {
        self.i += self.v[x] as usize;
        self.v[0xF] = if self.i > 0x0F00 {1} else {0};
        Flow::Next
    }

    fn LDF(&mut self, x: usize) -> Flow {
        self.i = (self.v[x] as usize) * 5;
        Flow::Next
    }

    fn LDB(&mut self, x: usize) -> Flow {
        self.ram[self.i] = self.v[x] / 100;
        self.ram[self.i + 1] = (self.v[x] % 100) / 10;
        self.ram[self.i + 2] = self.v[x] % 10;
        Flow::Next
    }

    fn LDIV(&mut self, x: usize) -> Flow {
        for i in 0..x + 1 {
            self.ram[self.i + i] = self.v[i]
        }
        Flow::Next
    }

    fn LDVI(&mut self, x: usize) -> Flow {
        for i in 0..x + 1 {
            self.v[i] = self.ram[self.i + 1];
        }
        Flow::Next
    }

    fn unknown_opcode(&mut self, opcode: u16) -> Flow {
        println!("{}", opcode);
        Flow::Next
    }
}