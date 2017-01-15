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


pub struct Glulx {
    program_counter: u32,
    stack_ptr: u32,
    call_frame_ptr: u32,
    stack: GlulxStack,
    memory: GlulxMemory,
}


impl Glulx {
    /// Create a glulx machine with the given ROM loaded.
    pub fn from_rom(rom: Vec<u8>) -> Result<Glulx, String> {
        GlulxMemory::from_rom(rom).and_then(|memory| {
            let stack = GlulxStack::new(memory.stack_size());
            Ok(Glulx {
                program_counter: 0,
                stack_ptr: 0,
                call_frame_ptr: 0,
                stack: stack,
                memory: memory,
            })
        })
    }

    /// Run the glulx machine until completion
    pub fn run(&mut self) {
        let start = self.memory.start_func();
        self.call(start, 0x0, Save::Null);
        loop {
            let opcode = self.fetch();
            self.eval(opcode);

        }
    }

    /// Call a function with nargs off the stack.
    fn call(&mut self,
            address: u32,
            nargs: u32,
            save: Save) {
        let args = self.stack.pop_args(nargs);
        self.push_call_stub(save);
        self.call_func(address, args);
    }

    /// Call a function with no arguments.
    fn callf(&mut self,
            address: u32,
            save: Save) {
        self.push_call_stub(save);
        self.call_func(address, vec![]);
    }

    /// Call a function with one input argument.
    fn callfi(&mut self,
            address: u32,
            arg1: u32,
            save: Save) {
        self.push_call_stub(save);
        self.call_func(address, vec![arg1]);
    }

    /// Call a function with two input arguments.
    fn callfii(&mut self,
            address: u32,
            arg1: u32,
            arg2: u32,
            save: Save) {
        self.push_call_stub(save);
        self.call_func(address, vec![arg1, arg2]);
    }

    /// Call a function with three input arguments.
    fn callfiii(&mut self,
            address: u32,
            arg1: u32,
            arg2: u32,
            arg3: u32,
            save: Save) {
        self.push_call_stub(save);
        self.call_func(address, vec![arg1, arg2, arg3]);
    }

    /// Parses the save location to determine the destination type and
    /// address, and then pushes that information (along with the
    /// current program counter value) onto the stack.
    fn push_call_stub(&mut self, save: Save) {
        let (dest_type, dest_addr) = match save {
            Save::Null => (0, 0),
            Save::Addr(addr) => (1, addr),
            Save::Frame(addr) => (2, addr),
            Save::Push => (3, 0),
            x => panic!("save mode not supported: {:?}", x),
        };
        self.stack.push_call_stub(dest_type, dest_addr, self.program_counter);
    }

    fn call_func(&mut self, address: u32, args: Vec<u32>) {
        self.program_counter = address;

        let func_type: u8 = self.memory.read(self.program_counter);
        self.program_counter += 0x1;

        let locals = self.read_locals();

        match func_type {
            0xC0 => self.stack.push_call_frame_c0(locals, args),
            0xC1 => self.stack.push_call_frame_c1(locals, args),
            _ => panic!("unsupported function type called"),

        }
    }

    /// Loops through the loals and return a copy of them.
    fn read_locals(&mut self) -> Vec<u8> {
        let mut vec = Vec::new();

        loop {
            let (local_type, local_count) = (
                self.memory.read(self.program_counter),
                self.memory.read(self.program_counter + 0x1),
            );
            self.program_counter += 0x2;
            vec.push(local_type);
            vec.push(local_count);

            if let (0, 0) = (local_type, local_count) {
                return vec;
            }
        }
    }

    fn op_return(&mut self, val: u32) {
        self.stack.pop_call_frame();
        let (
            dest_type,
            dest_addr,
            program_counter,
        ) = self.stack.pop_call_stub();

        self.program_counter = program_counter;

        let save = match dest_type {
            0x0 => Save::Null,
            0x1 => Save::Addr(dest_addr),
            0x2 => Save::Frame(dest_addr),
            0x3 => Save::Push,
            x => panic!("invalid dest_type returned from stack: {:#X}", x),
        };

        self.save(save, val);
    }

    /// Offset the current program counter by the given value. An
    /// additional 0x2 is subtracted from the offset.
    fn offset_ptr(&mut self, ptr: u32) {
        self.program_counter += ptr - 0x2;
    }

    /// Split the byte at the current program counter address into two
    /// bytes, the first representing the lower 4 bits and the second
    /// representing the upper 4 bits.
    fn lo_hi(&mut self) -> (u8, u8) {
        let bytes: u8 = self.memory.read(self.program_counter);
        self.program_counter += 0x1;
        (bytes & 0x0F, (bytes & 0xF0) >> 0x4)
    }

    /// Takes a number indicating a type of store action and uses that
    /// to create `Save` information for an operation. This will then
    /// increment the program counter forward depending on how many
    /// bytes are used.
    fn store_op_data(&mut self, mode: u8) -> Save {
        match mode {
            0x0 => Save::Null,
            0x5 => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Save::Addr(data as u32)
            },
            0x6 => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Save::Addr(data as u32)
            },
            0x7 => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Save::Addr(data)
            },
            0x8 => Save::Push,
            0x9 => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Save::Frame(data as u32)
            },
            0xA => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Save::Frame(data as u32)
            },
            0xB => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Save::Frame(data)
            },
            0xD => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Save::Ram(data as u32)
            },
            0xE => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Save::Ram(data as u32)
            },
            0xF => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Save::Ram(data)
            },
            _ => panic!("unrecognized addressing mode {:#X}", mode),
        }
    }

    /// Takes a number indicating a type of load action and uses that
    /// to create `Load` information for an operation. This will then
    /// increment the program counter forward depending on how many
    /// bytes are used.
    fn load_op_data(&mut self, mode: u8) -> Load {
        match mode {
            0x0 => Load::Const(0),
            0x1 => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Load::Const(data as i8 as i32)
            },
            0x2 => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Load::Const(data as i16 as i32)
            },
            0x3 => {
                let data: i32 = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Load::Const(data)
            },
            0x5 => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Load::Addr(data as u32)
            },
            0x6 => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Load::Addr(data as u32)
            },
            0x7 => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Load::Addr(data)
            },
            0x8 => Load::Pop,
            0x9 => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Load::Frame(data as u32)
            },
            0xA => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Load::Frame(data as u32)
            },
            0xB => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Load::Frame(data)
            },
            0xD => {
                let data: u8 = self.memory.read(self.program_counter);
                self.program_counter += 0x1;
                Load::Ram(data as u32)
            },
            0xE => {
                let data: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                Load::Ram(data as u32)
            },
            0xF => {
                let data = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                Load::Ram(data)
            },
            _ => panic!("unrecognized addressing mode {:#X}", mode),
        }
    }

    /// Return the opcode number from memory, incrementing the program
    /// counter to the end of the opcode, and before operand identifiers.
    fn opcode_number(&mut self) -> u32 {
        let top: u8 = self.memory.read(self.program_counter);
        match top {
            _ if top < 0x80 => {
                self.program_counter += 0x1;
                top as u32
            },
            _ if top < 0xC0 => {
                let opcode: u16 = self.memory.read(self.program_counter);
                self.program_counter += 0x2;
                opcode as u32 - 0x8000
            },
            _ => {
                let opcode: u32 = self.memory.read(self.program_counter);
                self.program_counter += 0x4;
                opcode - 0xC000_0000
            },
        }
    }

    /// Reads and returns the opcode starting at the current memory
    /// location.
    fn fetch(&mut self) -> Opcode {
        use super::instructions::Opcode::*;

        match self.opcode_number() {
            0x00 => NOP,
            0x10 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ADD(l1, l2, s1)
            },
            0x11 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                SUB(l1, l2, s1)
            },
            0x12 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                MUL(l1, l2, s1)
            },
            0x13 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                DIV(l1, l2, s1)
            },
            0x14 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                MOD(l1, l2, s1)
            },
            0x15 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                NEG(l1, s1)
            },
            0x18 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                BITAND(l1, l2, s1)
            },
            0x19 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                BITOR(l1, l2, s1)
            },
            0x1A => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                BITXOR(l1, l2, s1)
            },
            0x1B => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                BITNOT(l1, s1)
            },
            0x1C => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                SHIFTL(l1, l2, s1)
            },
            0x1D => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                SSHIFTR(l1, l2, s1)
            },
            0x1E => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                USHIFTR(l1, l2, s1)
            },
            0x20 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                JUMP(l1)
            },
            0x22 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                JZ(l1, l2)
            },
            0x23 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                JNZ(l1, l2)
            },
            0x24 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JEQ(l1, l2, l3)
            },
            0x25 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JNE(l1, l2, l3)
            },
            0x26 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JLT(l1, l2, l3)
            },
            0x27 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JGE(l1, l2, l3)
            },
            0x28 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JGT(l1, l2, l3)
            },
            0x29 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JLE(l1, l2, l3)
            },
            0x2A => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JLTU(l1, l2, l3)
            },
            0x2B => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JGEU(l1, l2, l3)
            },
            0x2C => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JGTU(l1, l2, l3)
            },
            0x2D => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JLEU(l1, l2, l3)
            },
            0x30 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                CALL(l1, l2, s1)
            },
            0x31 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                RETURN(l1)
            },
            0x32 => {
                let (s1, l1) = self.lo_hi();

                let s1 = self.store_op_data(s1);
                let l1 = self.load_op_data(l1);

                CATCH(s1, l1)
            },
            0x33 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                THROW(l1, l2)
            },
            0x34 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                TAILCALL(l1, l2)
            },
            0x40 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                COPY(l1, s1)
            },
            0x41 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                COPYS(l1, s1)
            },
            0x42 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                COPYB(l1, s1)
            },
            0x44 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SEXS(l1, s1)
            },
            0x45 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SEXB(l1, s1)
            },
            0x48 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ALOAD(l1, l2, s1)
            },
            0x49 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ALOADS(l1, l2, s1)
            },
            0x4A => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ALOADB(l1, l2, s1)
            },
            0x4B => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ALOADBIT(l1, l2, s1)
            },
            0x4C => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                ASTORE(l1, l2, l3)
            },
            0x4D => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                ASTORES(l1, l2, l3)
            },
            0x4E => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                ASTOREB(l1, l2, l3)
            },
            0x4F => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                ASTOREBIT(l1, l2, l3)
            },
            0x50 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                STKCOUNT(s1)
            },
            0x51 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                STKPEEK(l1, s1)
            },
            0x52 => STKSWAP,
            0x53 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                STKROLL(l1, l2)
            },
            0x54 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                STKCOPY(l1)
            },
            0x70 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                STREAMCHAR(l1)
            },
            0x71 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                STREAMNUM(l1)
            },
            0x72 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                STREAMSTR(l1)
            },
            0x73 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                STREAMUNICHAR(l1)
            },
            0x100 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                GESTALT(l1, l2, s1)
            },
            0x101 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                DEBUGTRAP(l1)
            },
            0x102 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                GETMEMSIZE(s1)
            },
            0x103 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SETMEMSIZE(l1, s1)
            },
            0x104 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                JUMPABS(l1, l2, l3)
            },
            0x110 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                RANDOM(l1, s1)
            },
            0x111 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                SETRANDOM(s1)
            },
            0x120 => QUIT,
            0x121 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                VERIFY(s1)
            },
            0x122 => RESTART,
            0x123 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SAVE(l1, s1)
            },
            0x124 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                RESTORE(l1, s1)
            },
            0x125 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                SAVEUNDO(s1)
            },
            0x126 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                RESTOREUNDO(s1)
            },
            0x127 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                PROTECT(l1, l2)
            },
            0x130 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                GLK(l1, l2, s1)
            },
            0x140 => {
                let (s1, _) = self.lo_hi();

                let s1 = self.store_op_data(s1);

                GETSTRINGTBL(s1)
            },
            0x141 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                SETSTRINGTBL(l1)
            },
            0x148 => {
                let (s1, s2) = self.lo_hi();

                let s1 = self.store_op_data(s1);
                let s2 = self.store_op_data(s2);

                GETIOSYS(s1, s2)
            },
            0x149 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                SETIOSYS(l1, l2)
            },
            0x150 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();
                let (l5, l6) = self.lo_hi();
                let (l7, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);
                let l5 = self.load_op_data(l5);
                let l6 = self.load_op_data(l6);
                let l7 = self.load_op_data(l7);
                let s1 = self.store_op_data(s1);

                LINEARSEARCH(l1, l2, l3, l4, l5, l6, l7, s1)
            }
            0x151 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();
                let (l5, l6) = self.lo_hi();
                let (l7, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);
                let l5 = self.load_op_data(l5);
                let l6 = self.load_op_data(l6);
                let l7 = self.load_op_data(l7);
                let s1 = self.store_op_data(s1);

                BINARYSEARCH(l1, l2, l3, l4, l5, l6, l7, s1)
            }
            0x152 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();
                let (l5, l6) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);
                let l5 = self.load_op_data(l5);
                let l6 = self.load_op_data(l6);
                let s1 = self.store_op_data(s1);

                LINKEDSEARCH(l1, l2, l3, l4, l5, l6, s1)
            }
            0x160 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                CALLF(l1, s1)
            },
            0x161 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                CALLFI(l1, l2, s1)
            },
            0x162 => {
                let (l1, l2) = self.lo_hi();
                let (l3, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let s1 = self.store_op_data(s1);

                CALLFII(l1, l2, l3, s1)
            },
            0x163 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);
                let s1 = self.store_op_data(s1);

                CALLFIII(l1, l2, l3, l4, s1)
            },
            0x170 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                MZERO(l1, l2)
            },
            0x171 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                MCOPY(l1, l2, l3)
            },
            0x178 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                MALLOC(l1, s1)
            },
            0x179 => {
                let (l1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);

                MFREE(l1)
            },
            0x180 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                ACCELFUNC(l1, l2)
            },
            0x181 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                ACCELPARAM(l1, l2)
            },
            0x190 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                NUMTOF(l1, s1)
            },
            0x191 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                FTONUMZ(l1, s1)
            },
            0x192 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                FTONUMN(l1, s1)
            },
            0x198 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                CEIL(l1, s1)
            },
            0x199 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                FLOOR(l1, s1)
            },
            0x1A0 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                FADD(l1, l2, s1)
            },
            0x1A1 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                FSUB(l1, l2, s1)
            },
            0x1A2 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                FMUL(l1, l2, s1)
            },
            0x1A3 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                FDIV(l1, l2, s1)
            },
            0x1A4 => {
                let (l1, l2) = self.lo_hi();
                let (s1, s2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);
                let s2 = self.store_op_data(s2);

                FMOD(l1, l2, s1, s2)
            },
            0x1A8 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SQRT(l1, s1)
            },
            0x1A9 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                EXP(l1, s1)
            },
            0x1AA => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                LOG(l1, s1)
            },
            0x1AB => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                POW(l1, l2, s1)
            },
            0x1B0 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                SIN(l1, s1)
            },
            0x1B1 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                COS(l1, s1)
            },
            0x1B2 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                TAN(l1, s1)
            },
            0x1B3 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                ASIN(l1, s1)
            },
            0x1B4 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                ACOS(l1, s1)
            },
            0x1B5 => {
                let (l1, s1) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let s1 = self.store_op_data(s1);

                ATAN(l1, s1)
            },
            0x1B6 => {
                let (l1, l2) = self.lo_hi();
                let (s1, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let s1 = self.store_op_data(s1);

                ATAN2(l1, l2, s1)
            },
            0x1C0 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);

                JFEQ(l1, l2, l3, l4)
            },
            0x1C1 => {
                let (l1, l2) = self.lo_hi();
                let (l3, l4) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);
                let l4 = self.load_op_data(l4);

                JFNE(l1, l2, l3, l4)
            },
            0x1C2 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JFLT(l1, l2, l3)
            },
            0x1C3 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JFLE(l1, l2, l3)
            },
            0x1C4 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JFGT(l1, l2, l3)
            },
            0x1C5 => {
                let (l1, l2) = self.lo_hi();
                let (l3, _) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);
                let l3 = self.load_op_data(l3);

                JFGE(l1, l2, l3)
            },
            0x1C8 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                JISNAN(l1, l2)
            },
            0x1C9 => {
                let (l1, l2) = self.lo_hi();

                let l1 = self.load_op_data(l1);
                let l2 = self.load_op_data(l2);

                JISINF(l1, l2)
            },
            x => panic!("instruction not implemented: {:#X}", x),
        }
    }

    fn eval(&mut self, opcode: Opcode) {
        use super::instructions::Opcode::*;
        use super::opcodes::*;

        match opcode {
            NOP => {},
            ADD(l1, l2, s1) => {
                let ret = op_add(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            SUB(l1, l2, s1) => {
                let ret = op_sub(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            MUL(l1, l2, s1) => {
                let ret = op_mul(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            DIV(l1, l2, s1) => {
                let ret = op_mod(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            NEG(l1, s1) => {
                let ret = op_neg(self.load(l1));
                self.save(s1, ret)
            },
            BITAND(l1, l2, s1) => {
                let ret = op_bitand(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            BITOR(l1, l2, s1) => {
                let ret = op_bitor(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            BITXOR(l1, l2, s1) => {
                let ret = op_bitxor(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            BITNOT(l1, s1) => {
                let ret = op_bitnot(self.load(l1));
                self.save(s1, ret)
            },
            SHIFTL(l1, l2, s1) => {
                let ret = op_shiftl(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            SSHIFTR(l1, l2, s1) => {
                let ret = op_sshiftr(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            USHIFTR(l1, l2, s1) => {
                let ret = op_ushiftr(self.load(l1), self.load(l2));
                self.save(s1, ret)
            },
            JUMP(l1) => {
                let nxt = self.load(l1);
                self.offset_ptr(nxt);
            },
            JZ(l1, l2) => {
                if op_jz(self.load(l1)) {
                    let nxt = self.load(l2);
                    self.offset_ptr(nxt);
                }
            },
            JNZ(l1, l2) => {
                if op_jnz(self.load(l1)) {
                    let nxt = self.load(l2);
                    self.offset_ptr(nxt);
                }
            },
            JEQ(l1, l2, l3) => {
                if op_jeq(self.load(l1), self.load(l2)) {
                    let nxt: u32 = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JNE(l1, l2, l3) => {
                if op_jne(self.load(l1), self.load(l2)) {
                    let nxt: u32 = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JLT(l1, l2, l3) => {
                if op_jlt(self.load(l1), self.load(l2)) {
                    let nxt: u32 = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JGE(l1, l2, l3) => {
                if op_jge(self.load(l1), self.load(l2)) {
                    let nxt: u32 = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JGT(l1, l2, l3) => {
                if op_jgt(self.load(l1), self.load(l2)) {
                    let nxt: u32 = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JLE(l1, l2, l3) => {
                if op_jle(self.load(l1), self.load(l2)) {
                    let nxt = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JLTU(l1, l2, l3) => {
                if op_jltu(self.load(l1), self.load(l2)) {
                    let nxt = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JGEU(l1, l2, l3) => {
                if op_jgeu(self.load(l1), self.load(l2)) {
                    let nxt = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JGTU(l1, l2, l3) => {
                if op_jgtu(self.load(l1), self.load(l2)) {
                    let nxt = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            JLEU(l1, l2, l3) => {
                if op_jleu(self.load(l1), self.load(l2)) {
                    let nxt = self.load(l3);
                    self.offset_ptr(nxt);
                }
            },
            CALL(l1, l2, s1) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);
                self.call(l1, l2, s1);
            },
            RETURN(l1) => {
                let l1 = self.load(l1);
                self.op_return(l1);
            },
            // TODO: CATCH(s1, l1),
            // TODO: THROW(l1, l2),
            // TODO: TAILCALL(l1, l2),
            COPY(l1, s1) => {
                let ret = op_copy(self.load(l1));
                self.save(s1, ret);
            },
            COPYS(l1, s1) => {
                let ret = op_copys(self.load(l1));
                self.save(s1, ret);
            },
            COPYB(l1, s1) => {
                let ret = op_copyb(self.load(l1));
                self.save(s1, ret);
            },
            SEXS(l1, s1) => {
                let ret = op_sexs(self.load(l1));
                self.save(s1, ret);
            },
            SEXB(l1, s1) => {
                let ret = op_sexs(self.load(l1));
                self.save(s1, ret);
            },
            ALOAD(l1, l2, s1) => {
                let (l1, l2): (u32, u32) = (self.load(l1), self.load(l2));
                let ret: u32 = self.memory.read(l1 + 4*l2);
                self.save(s1, ret);
            },
            // TODO: ALOADS(l1, l2, s1),
            // TODO: ALOADB(l1, l2, s1),
            // TODO: ALOADBIT(l1, l2, s1),
            ASTORE(l1, l2, l3) => {
                let address = op_astore(self.load(l1), self.load(l2));
                let l3: u32 = self.load(l3);
                self.memory.write(address, l3)
            },
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
            GETMEMSIZE(s1) => {
                let ret = self.memory.get_mem_size();
                self.save(s1, ret);
            },
            SETMEMSIZE(l1, s1) => {
                let l1 = self.load(l1);
                let ret = self.memory.set_mem_size(l1);
                self.save(s1, ret);
            },
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
            CALLF(l1, s1) => {
                let l1 = self.load(l1);
                self.callf(l1, s1);
            }
            CALLFI(l1, l2, s1) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);

                self.callfi(l1, l2, s1);
            },
            CALLFII(l1, l2, l3, s1) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);
                let l3 = self.load(l3);

                self.callfii(l1, l2, l3, s1);
            },
            CALLFIII(l1, l2, l3, l4, s1) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);
                let l3 = self.load(l3);
                let l4 = self.load(l4);

                self.callfiii(l1, l2, l3, l4, s1);
            },
            MZERO(l1, l2) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);
                self.memory.zero_range(l1, l2)
            },
            MCOPY(l1, l2, l3) => {
                let l1 = self.load(l1);
                let l2 = self.load(l2);
                let l3 = self.load(l3);
                self.memory.copy_range(l1, l2, l3)
            },
            // TODO: MALLOC(l1, s1),
            // TODO: MFREE(l1),
            // TODO: ACCELFUNC(l1, l2),
            // TODO: ACCELPARAM(l1, l2),
            NUMTOF(l1, s1) => {
                let ret = op_numtof(self.load(l1));
                self.save(s1, ret);
            },
            FTONUMZ(l1, s1) => {
                let ret = op_ftonumz(self.load(l1));
                self.save(s1, ret);
            },
            FTONUMN(l1, s1) => {
                let ret = op_ftonumn(self.load(l1));
                self.save(s1, ret);
            },
            FADD(l1, l2, s1) => {
                let ret = op_fadd(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            FSUB(l1, l2, s1) => {
                let ret = op_fsub(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            FMUL(l1, l2, s1) => {
                let ret = op_fmul(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            FDIV(l1, l2, s1) => {
                let ret = op_fdiv(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            FMOD(l1, l2, s1, s2) => {
                let ret = op_fmod(self.load(l1), self.load(l2));
                self.save(s1, ret.0);
                self.save(s2, ret.1);
            }
            SQRT(l1, s1) => {
                let ret = op_sqrt(self.load(l1));
                self.save(s1, ret);
            },
            EXP(l1, s1) => {
                let ret = op_exp(self.load(l1));
                self.save(s1, ret);
            },
            LOG(l1, s1) => {
                let ret = op_log(self.load(l1));
                self.save(s1, ret);
            },
            POW(l1, l2, s1) => {
                let ret = op_pow(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            SIN(l1, s1) => {
                let ret = op_sin(self.load(l1));
                self.save(s1, ret);
            },
            COS(l1, s1) => {
                let ret = op_cos(self.load(l1));
                self.save(s1, ret);
            },
            TAN(l1, s1) => {
                let ret = op_tan(self.load(l1));
                self.save(s1, ret);
            },
            ASIN(l1, s1) => {
                let ret = op_asin(self.load(l1));
                self.save(s1, ret);
            },
            ACOS(l1, s1) => {
                let ret = op_acos(self.load(l1));
                self.save(s1, ret);
            },
            ATAN(l1, s1) => {
                let ret = op_atan(self.load(l1));
                self.save(s1, ret);
            },
            ATAN2(l1, l2, s1) => {
                let ret = op_atan2(self.load(l1), self.load(l2));
                self.save(s1, ret);
            },
            JFEQ(l1, l2, l3, l4) => {
                if op_jfeq(self.load(l1), self.load(l2), self.load(l3)){
                    let ret = self.load(l4);
                    self.offset_ptr(ret);
                }
            },
            JFNE(l1, l2, l3, l4) => {
                if op_jfne(self.load(l1), self.load(l2), self.load(l3)){
                    let ret = self.load(l4);
                    self.offset_ptr(ret);
                }
            },
            JFLT(l1, l2, l3) => {
                if op_jflt(self.load(l1), self.load(l2)) {
                    let ret = self.load(l3);
                    self.offset_ptr(ret);
                }
            },
            JFLE(l1, l2, l3) => {
                if op_jfle(self.load(l1), self.load(l2)) {
                    let ret = self.load(l3);
                    self.offset_ptr(ret);
                }
            },
            JFGT(l1, l2, l3) => {
                if op_jfgt(self.load(l1), self.load(l2)) {
                    let ret = self.load(l3);
                    self.offset_ptr(ret);
                }
            },
            JFGE(l1, l2, l3) => {
                if op_jfge(self.load(l1), self.load(l2)) {
                    let ret = self.load(l3);
                    self.offset_ptr(ret);
                }
            },
            JISNAN(l1, l2) => {
                if op_jisnan(self.load(l1)) {
                    let ret = self.load(l2);
                    self.offset_ptr(ret);
                }
            },
            JISINF(l1, l2) => {
                if op_jisinf(self.load(l1)) {
                    let ret = self.load(l2);
                    self.offset_ptr(ret);
                }
            },
            x => panic!("opcode not implemented: {:?}", x),
        }
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
            Pop => {
                let val: u32 = self.stack.pop();
                val as u8
            },
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: u8){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value as u32),
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
            Pop => {
                let val: u32 = self.stack.pop();
                val as u16
            },
            Frame(ptr) => self.stack.read(ptr),
            Ram(ptr) => self.memory.ram_read(ptr),
        }
    }

    fn save(&mut self, save: Save, value: u16){
        use super::instructions::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value as u32),
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
            Const(val) => val as u32,
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
