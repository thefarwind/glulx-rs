use instructions::{
    Load,
    Opcode,
    Save,
};

use memory::{
    GlulxMemory,
    Memory,
};

use stack::{
    GlulxStack,
    Stack,
};


struct Glulx {
    program_counter: u32,
    stack_ptr: u32,
    call_frame_ptr: u32,
    stack: GlulxStack,
    memory: GlulxMemory,
}


/*

fn read(glulx: &Glulx) -> Opcode {
    use self::Opcode::*;

    let top = glulx.memory.read_u8(glulx.program_counter);
    match top {
        _ if top < 0x80 => {
            let opcode_number = top as u32;
            // TODO: program counter + 1;
        },
        _ if top < 0xC0 => {
            let opcode_number = glulx.memory
                    .read_u16(glulx.program_counter) as u32 - 0x8000;
            // TODO: program counter + 2;
        },
        _ => {
            let opcode_number = glulx.memory
                    .read_u32(glulx.program_counter) - 0xC0000000;
            // TODO: program counter + 4;
        },
    };

    ADD(0, 1, 2)
}
*/


impl Glulx {
    fn offset_ptr(mut self, ptr: u32) -> Glulx {
        self.program_counter += ptr - 0x2;
        self
    }
}


trait Machine<T> {
    fn load(&mut self, load: Load) -> T;
    fn save(&mut self, save: Save, value: T);
}


impl Machine<u8> for Glulx {
    fn load(&mut self, load: Load) -> u8 {
        use super::instructions::Load::*;

        match load {
            Const(val) => val as u8,
            Addr(ptr) => self.memory.read(ptr),
            Pop => self.stack.pop(),
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: u8){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl Machine<u16> for Glulx {
    fn load(&mut self, load: Load) -> u16 {
        use super::instructions::Load::*;

        match load {
            Const(val) => val as u16,
            Addr(ptr) => self.memory.read(ptr),
            Pop => self.stack.pop(),
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: u16){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl Machine<i32> for Glulx {
    fn load(&mut self, load: Load) -> i32 {
        use super::instructions::Load::*;

        match load {
            Const(val) => val,
            Addr(ptr) => self.memory.read(ptr),
            Pop => self.stack.pop(),
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: i32){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl Machine<u32> for Glulx {
    fn load(&mut self, load: Load) -> u32 {
        use super::instructions::Load::*;

        match load {
            Const(val) => panic!("const u32 not supported"),
            Addr(ptr) => self.memory.read(ptr),
            Pop => self.stack.pop(),
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: u32){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl Machine<f32> for Glulx {
    fn load(&mut self, load: Load) -> f32 {
        use super::instructions::Load::*;

        match load {
            Const(val) => panic!("const f32 not supported"),
            Addr(ptr) => self.memory.read(ptr),
            Pop => self.stack.pop(),
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: f32){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


fn eval(glulx: Glulx, opcode: Opcode) {
    let mut glulx = glulx;
    use super::instructions::Opcode::*;
    use super::opcodes::*;

    match opcode {
        NOP => {},
        ADD(l1, l2, s1) => {
            let ret = op_add(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        SUB(l1, l2, s1) => {
            let ret = op_sub(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        MUL(l1, l2, s1) => {
            let ret = op_mul(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        DIV(l1, l2, s1) => {
            let ret = op_mod(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        NEG(l1, s1) => {
            let ret = op_neg(glulx.load(l1));
            glulx.save(s1, ret)
        },
        BITAND(l1, l2, s1) => {
            let ret = op_bitand(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        BITOR(l1, l2, s1) => {
            let ret = op_bitor(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        BITXOR(l1, l2, s1) => {
            let ret = op_bitxor(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        BITNOT(l1, s1) => {
            let ret = op_bitnot(glulx.load(l1));
            glulx.save(s1, ret)
        },
        SHIFTL(l1, l2, s1) => {
            let ret = op_shiftl(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        SSHIFTR(l1, l2, s1) => {
            let ret = op_sshiftr(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        USHIFTR(l1, l2, s1) => {
            let ret = op_ushiftr(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret)
        },
        JUMP(l1) => {
            let nxt = glulx.load(l1);
            glulx = glulx.offset_ptr(nxt);
        },
        JZ(l1, l2) => {
            glulx = if op_jz(glulx.load(l1)) {
                let nxt = glulx.load(l2);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JNZ(l1, l2) => {
            glulx = if op_jnz(glulx.load(l1)) {
                let nxt = glulx.load(l2);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JEQ(l1, l2, l3) => {
            glulx = if op_jeq(glulx.load(l1), glulx.load(l2)) {
                let nxt: u32 = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JNE(l1, l2, l3) => {
            glulx = if op_jne(glulx.load(l1), glulx.load(l2)) {
                let nxt: u32 = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JLT(l1, l2, l3) => {
            glulx = if op_jlt(glulx.load(l1), glulx.load(l2)) {
                let nxt: u32 = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JGE(l1, l2, l3) => {
            glulx = if op_jge(glulx.load(l1), glulx.load(l2)) {
                let nxt: u32 = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JGT(l1, l2, l3) => {
            glulx = if op_jgt(glulx.load(l1), glulx.load(l2)) {
                let nxt: u32 = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JLE(l1, l2, l3) => {
            glulx = if op_jle(glulx.load(l1), glulx.load(l2)) {
                let nxt = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JLTU(l1, l2, l3) => {
            glulx = if op_jltu(glulx.load(l1), glulx.load(l2)) {
                let nxt = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JGEU(l1, l2, l3) => {
            glulx = if op_jgeu(glulx.load(l1), glulx.load(l2)) {
                let nxt = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JGTU(l1, l2, l3) => {
            glulx = if op_jgtu(glulx.load(l1), glulx.load(l2)) {
                let nxt = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        JLEU(l1, l2, l3) => {
            glulx = if op_jleu(glulx.load(l1), glulx.load(l2)) {
                let nxt = glulx.load(l3);
                glulx.offset_ptr(nxt)
            } else {
                glulx
            };
        },
        // TODO: CALL(l1, l2, s1),
        // TODO: RETURN(l1),
        // TODO: CATCH(s1, l1),
        // TODO: THROW(l1, l2),
        // TODO: TAILCALL(l1, l2),
        COPY(l1, s1) => {
            let ret = op_copy(glulx.load(l1));
            glulx.save(s1, ret);
        },
        COPYS(l1, s1) => {
            let ret = op_copys(glulx.load(l1));
            glulx.save(s1, ret);
        },
        COPYB(l1, s1) => {
            let ret = op_copyb(glulx.load(l1));
            glulx.save(s1, ret);
        },
        SEXS(l1, s1) => {
            let ret = op_sexs(glulx.load(l1));
            glulx.save(s1, ret);
        },
        SEXB(l1, s1) => {
            let ret = op_sexs(glulx.load(l1));
            glulx.save(s1, ret);
        },
        // TODO: ALOAD(l1, l2, s1),
        // TODO: ALOADS(l1, l2, s1),
        // TODO: ALOADB(l1, l2, s1),
        // TODO: ALOADBIT(l1, l2, s1),
        // TODO: ASTORE(l1, l2, l3),
        // TODO: ASTORES(l1, l2, l3),
        // TODO: ASTOREB(l1, l2, l3),
        // TODO: ASTOREBIT(l1, l2, l3),
        // TODO: STKCOUNT(s1),
        // TODO: STKPEEK(l1, s1),
        // TODO: STKSWAP,
        // TODO: STKROLL(l1, l2),
        // TODO: STKCOPY(l1),
        // TODO: STREAMCHAR(l1),
        // TODO: STREAMNUM(l1),
        // TODO: STREAMSTR(l1),
        // TODO: STREAMUNICHAR(l1),
        // TODO: GESTALT(l1, l2, s1),
        // TODO: DEBUGTRAP(l1),
        // TODO: GETMEMSIZE(s1),
        // TODO: SETMEMSIZE(l1, s1),
        // TODO: JUMPABS(l1, l2, l3),
        // TODO: RANDOM(l1, s1),
        // TODO: SETRANDOM(s1),
        // TODO: QUIT,
        // TODO: VERIFY(s1),
        // TODO: RESTART,
        // TODO: SAVE(l1, s1),
        // TODO: RESTORE(l1, s1),
        // TODO: SAVEUNDO(s1),
        // TODO: RESTOREUNDO(s1),
        // TODO: PROTECT(l1, l2),
        // TODO: GLK(l1, l2, s1),
        // TODO: GETSTRINGTBL(s1),
        // TODO: SETSTRINGTBL(l1),
        // TODO: GETIOSYS(s1, s2),
        // TODO: SETIOSYS(l1, l2),
        // TODO: LINEARSEARCH(l1, l2, l3, l4, l5, l6, l7, s1),
        // TODO: BINARYSEARCH(l1, l2, l3, l4, l5, l6, l7, s1),
        // TODO: LINKEDSEARCH(l1, l2, l3, l4, l5, l6, s1),
        // TODO: CALLF(l1, s1),
        // TODO: CALLFI(l1, l2, s1),
        // TODO: CALLFII(l1, l2, l3, s1),
        // TODO: CALLFIII(l1, l2, l3, l4, s1),
        MZERO(l1, l2) => {
            let l1 = glulx.load(l1);
            let l2 = glulx.load(l2);
            glulx.memory.zero_range(l1, l2)
        },
        MCOPY(l1, l2, l3) => {
            let l1 = glulx.load(l1);
            let l2 = glulx.load(l2);
            let l3 = glulx.load(l3);
            glulx.memory.copy_range(l1, l2, l3)
        },
        // TODO: MALLOC(l1, s1),
        // TODO: MFREE(l1),
        // TODO: ACCELFUNC(l1, l2),
        // TODO: ACCELPARAM(l1, l2),
        NUMTOF(l1, s1) => {
            let ret = op_numtof(glulx.load(l1));
            glulx.save(s1, ret);
        },
        FTONUMZ(l1, s1) => {
            let ret = op_ftonumz(glulx.load(l1));
            glulx.save(s1, ret);
        },
        FTONUMN(l1, s1) => {
            let ret = op_ftonumn(glulx.load(l1));
            glulx.save(s1, ret);
        },
        FADD(l1, l2, s1) => {
            let ret = op_fadd(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        FSUB(l1, l2, s1) => {
            let ret = op_fsub(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        FMUL(l1, l2, s1) => {
            let ret = op_fmul(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        FDIV(l1, l2, s1) => {
            let ret = op_fdiv(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        FMOD(l1, l2, s1, s2) => {
            let ret = op_fmod(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret.0);
            glulx.save(s2, ret.1);
        }
        SQRT(l1, s1) => {
            let ret = op_sqrt(glulx.load(l1));
            glulx.save(s1, ret);
        },
        EXP(l1, s1) => {
            let ret = op_exp(glulx.load(l1));
            glulx.save(s1, ret);
        },
        LOG(l1, s1) => {
            let ret = op_log(glulx.load(l1));
            glulx.save(s1, ret);
        },
        POW(l1, l2, s1) => {
            let ret = op_pow(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        SIN(l1, s1) => {
            let ret = op_sin(glulx.load(l1));
            glulx.save(s1, ret);
        },
        COS(l1, s1) => {
            let ret = op_cos(glulx.load(l1));
            glulx.save(s1, ret);
        },
        TAN(l1, s1) => {
            let ret = op_tan(glulx.load(l1));
            glulx.save(s1, ret);
        },
        ASIN(l1, s1) => {
            let ret = op_asin(glulx.load(l1));
            glulx.save(s1, ret);
        },
        ACOS(l1, s1) => {
            let ret = op_acos(glulx.load(l1));
            glulx.save(s1, ret);
        },
        ATAN(l1, s1) => {
            let ret = op_atan(glulx.load(l1));
            glulx.save(s1, ret);
        },
        ATAN2(l1, l2, s1) => {
            let ret = op_atan2(glulx.load(l1), glulx.load(l2));
            glulx.save(s1, ret);
        },
        JFEQ(l1, l2, l3, l4) => {
            glulx = if op_jfeq(glulx.load(l1), glulx.load(l2), glulx.load(l3)){
                let ret = glulx.load(l4);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JFNE(l1, l2, l3, l4) => {
            glulx = if op_jfne(glulx.load(l1), glulx.load(l2), glulx.load(l3)){
                let ret = glulx.load(l4);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JFLT(l1, l2, l3) => {
            glulx = if op_jflt(glulx.load(l1), glulx.load(l2)) {
                let ret = glulx.load(l3);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JFLE(l1, l2, l3) => {
            glulx = if op_jfle(glulx.load(l1), glulx.load(l2)) {
                let ret = glulx.load(l3);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JFGT(l1, l2, l3) => {
            glulx = if op_jfgt(glulx.load(l1), glulx.load(l2)) {
                let ret = glulx.load(l3);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JFGE(l1, l2, l3) => {
            glulx = if op_jfge(glulx.load(l1), glulx.load(l2)) {
                let ret = glulx.load(l3);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JISNAN(l1, l2) => {
            glulx = if op_jisnan(glulx.load(l1)) {
                let ret = glulx.load(l2);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        JISINF(l1, l2) => {
            glulx = if op_jisinf(glulx.load(l1)) {
                let ret = glulx.load(l2);
                glulx.offset_ptr(ret)
            } else {
                glulx
            }
        },
        _ => {},
    }
}
