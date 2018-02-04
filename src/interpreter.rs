use std::mem::size_of;

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
    stack: GlulxStack,
    memory: GlulxMemory,
    running: bool,
}


macro_rules! opcode_match {
    (@inner $self_:ident, $arg1:ident $arg2:ident $($args:ident)*) => {
        let ($arg1, $arg2) = $self_.lo_hi();
        opcode_match!(@inner $self_, $($args)*);
    };
    (@inner $self_:ident, $arg:ident) => {
        let ($arg, _) = $self_.lo_hi();
    };
    (@inner $self_:ident,) => (());
    ($self_:ident,
        $val:expr,
        $($num:pat => $opcode:ident ( $($args:ident),* $(,)* ) ),* $(,)*
    ) => (
        match $val {
            $(
                $num => {
                    opcode_match!(@inner $self_, $($args)*);
                    $(let $args = $self_.read_register($args);)*
                    $self_.$opcode($($args),*)
                },
            )*
            x => panic!("unsupported opcode: {:#X}", x),
        }
    )
}


impl Glulx {
    /// Create a glulx machine with the given ROM loaded.
    pub fn from_rom(rom: Vec<u8>) -> Result<Glulx, String> {
        GlulxMemory::from_rom(rom).and_then(|memory| {
            let stack = GlulxStack::new(memory.stack_size());
            Ok(Glulx {
                program_counter: 0,
                stack: stack,
                memory: memory,
                running: false,
            })
        })
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

    /// Does nothing.
    pub fn op_nop(&mut self) {}
    /// Add l1 to l2 and save the result in s1.
    pub fn op_add(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1.wrapping_add(l2));
    }
    /// Subtract l2 from l1 and save the result in s1.
    pub fn op_sub(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1.wrapping_sub(l2));
    }
    /// Multiply l1 and l2 and save the result in s1.
    pub fn op_mul(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1.wrapping_mul(l2));
    }
    /// Divide l1 by l2 and save the result in s1.
    pub fn op_div(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1.wrapping_div(l2));
    }
    /// Mod l1 by l2 and save the result in s1.
    pub fn op_mod(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1.wrapping_rem(l2));
    }
    /// Negate l1 and save the result in s1.
    pub fn op_neg(&mut self, l1: i32, s1: Save) {
        self.save(s1, l1.wrapping_neg());
    }
    /// TODO
    pub fn op_bitand(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1 & l2);
    }
    /// TODO
    pub fn op_bitor(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1 | l2);
    }
    /// TODO
    pub fn op_bitxor(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1 ^ l2);
    }
    /// TODO
    pub fn op_bitnot(&mut self, l1: i32, s1: Save) {
        self.save(s1, !l1);
    }
    /// TODO
    pub fn op_shiftl(&mut self, l1: u32, l2: u32, s1: Save) {
        self.save(s1, l1 << l2);
    }
    /// TODO
    pub fn op_sshiftr(&mut self, l1: i32, l2: i32, s1: Save) {
        self.save(s1, l1 >> l2);
    }
    /// TODO
    pub fn op_ushiftr(&mut self, l1: u32, l2: u32, s1: Save) {
        self.save(s1, l1 >> l2);
    }
    /// TODO
    pub fn op_jump(&mut self, l1: u32) {
        self.program_counter = self.program_counter
            .wrapping_add(l1)
            .wrapping_sub(0x2);
    }
    /// If l1 is 0x0, jump to l2.
    pub fn op_jz(&mut self, l1: i32, l2: u32) {
        if l1 == 0x0 { self.op_jump(l2) };
    }
    /// If l1 is not 0x0, jump to l2.
    pub fn op_jnz(&mut self, l1: i32, l2: u32) {
        if l1 != 0x0 { self.op_jump(l2) };
    }
    /// If l1 equals l2, jump to l3.
    pub fn op_jeq(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 == l2 { self.op_jump(l3) };
    }
    /// If l1 is not equal to l2, jump to l3.
    pub fn op_jne(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 != l2 { self.op_jump(l3) };
    }
    /// If l1 is less than l2, jump to l3.
    pub fn op_jlt(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 < l2 { self.op_jump(l3) };
    }
    /// If l1 is greater than or equal to l2, jump to l3.
    pub fn op_jge(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 >= l2 { self.op_jump(l3) };
    }
    /// If l1 is greater than l2, jump to l3.
    pub fn op_jgt(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 > l2 { self.op_jump(l3) };
    }
    /// If l1 is less than or equal to l2, jump to l3.
    pub fn op_jle(&mut self, l1: i32, l2: i32, l3: u32) {
        if l1 <= l2 { self.op_jump(l3) };
    }
    /// If unsigned l1 is less than unsigned l2, jump to l3.
    pub fn op_jltu(&mut self, l1: u32, l2: u32, l3: u32) {
        if l1 < l2 { self.op_jump(l3) };
    }
    /// If unsigned l1 is greater than or equal to unsigned l2, jump to l3.
    pub fn op_jgeu(&mut self, l1: u32, l2: u32, l3: u32) {
        if l1 >= l2 { self.op_jump(l3) };
    }
    /// If unsigned l1 is greather than unsigned l2, jump to l3.
    pub fn op_jgtu(&mut self, l1: u32, l2: u32, l3: u32) {
        if l1 > l2 { self.op_jump(l3) };
    }
    /// If unsigned l1 is less than or equal to unsigned l2, jump to l3.
    pub fn op_jleu(&mut self, l1: u32, l2: u32, l3: u32) {
        if l1 <= l2 { self.op_jump(l3) };
    }
    /// Call function at address l1 with l2 arguments.
    pub fn op_call(&mut self, l1: u32, l2: u32, s1: Save) {
        let args = self.stack.pop_args(l2);
        self.push_call_stub(s1);
        self.call_func(l1, args);
    }
    /// Return l1 from a function call.
    pub fn op_return(&mut self, l1: u32) {
        self.stack.pop_call_frame();
        let (dest_type, dest_addr, program_counter) = self.stack.pop_call_stub();
        self.program_counter = program_counter;
        let save = match dest_type {
            0x0 => Save::Null,
            0x1 => Save::Addr(dest_addr),
            0x2 => Save::Frame(dest_addr),
            0x3 => Save::Push,
            x => panic!("invalid dest_type returned from stack: {:#X}", x),
        };
        self.save(save, l1);
    }
    /// TODO
    pub fn op_catch(&mut self, s1: Save, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_throw(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// Tailcall the function at l1 with l2 number of args
    pub fn op_tailcall(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// Copy a u32 to s1.
    pub fn op_copy(&mut self, l1: u32, s1: Save) {
        self.save(s1, l1)
    }
    /// Copy a u16 to s1.
    pub fn op_copys(&mut self, l1: u16, s1: Save) {
        self.save(s1, l1)
    }
    /// Copy a u8 to s1.
    pub fn op_copyb(&mut self, l1: u8, s1: Save) {
        self.save(s1, l1)
    }
    /// Load a u32, and sign extend the lower 0x10 bits to a i32.
    pub fn op_sexs(&mut self, l1: u32, s1: Save) {
        self.save(s1, l1 as i16 as i32)
    }
    /// Load a u32, and sign extend the lower 0x8 bits to a i32.
    pub fn op_sexb(&mut self, l1: u32, s1: Save) {
        self.save(s1, l1 as i8 as i32)
    }
    /// Load the value at l1 + 4*l2 and save at s1.
    pub fn op_aload(&mut self, l1: u32, l2: u32, s1: Save) {
        let ret: u32 = self.memory.read(l1 + (l2 << 0x2));
        self.save(s1, ret);
    }
    /// Load a u16 from l1 + 2*l2 and store at s1 as a u32.
    pub fn op_aloads(&mut self, l1: u32, l2: u32, s1: Save) {
        let ret: u16 = self.memory.read(l1 + (l2 << 0x1));
        self.save(s1, ret);
    }
    /// Load a u8 from l1 + l2 and store at s1 as a u32.
    pub fn op_aloadb(&mut self, l1: u32, l2: u32, s1: Save) {
        let ret: u8 = self.memory.read(l1 + l2);
        self.save(s1, ret as u32);
    }
    /// Load a bit from l1 + l2/8 and store at s1 as a u32.
    pub fn op_aloadbit(&mut self, l1: u32, l2: u32, s1: Save) {
        unimplemented!()
    }
    /// Store l3 as a u32 at memory location l1 + 4 * l2
    pub fn op_astore(&mut self, l1: u32, l2: u32, l3: u32) {
        self.memory.write(l1 + (l2 << 0x2), l3)
    }
    /// Store l3 as a u16 at memory location l1 + 2 * l2
    pub fn op_astores(&mut self, l1: u32, l2: u32, l3: u32) {
        self.memory.write(l1 + (l2 << 0x1), l3 as u16)
    }
    /// Store l3 as a u8 at memory location l1 + l2
    pub fn op_astoreb(&mut self, l1: u32, l2: u32, l3: u32) {
        self.memory.write(l1 + l2, l3 as u8)
    }
    /// TODO
    pub fn op_astorebit(&mut self, l1: u32, l2: u32, l3: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_stkcount(&mut self, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_stkpeek(&mut self, l1: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_stkswap(&mut self) {
        unimplemented!()
    }
    /// TODO
    pub fn op_stkroll(&mut self, l1: u32, l2: i32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_stkcopy(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_streamchar(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_streamnum(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_streamstr(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_streamunichar(&mut self, l1: u32) {
        unimplemented!()
    }
    /// Returns a value indicating if vm features are implemented.
    pub fn op_gestalt(&mut self, l1: u16, l2: u16, s1: Save) {
        let ret = match (l1, l2) {
            (0x0, _) => self.memory.glulx_version(),
            (0x1, _) => 0x1, // interpreter version
            (0x2, _) => 0x0, // setmemsize implemented
            (0x3, _) => 0x0, // saveundo and restoreundo implemented
            (0x4, 0x0) => 0x1, // iosystem null implemented
            (0x4, 0x1) => 0x0, // iosystem filter implemented
            (0x4, 0x2) => 0x0, // iosystem gtk implemented
            (0x4, 0x20) => 0x0, // iosystem fyrevm implemented
            (0x5, _) => 0x0, // unicode support implemented
            (0x6, _) => 0x1, // mzero and mcopy implemented
            (0x7, _) => 0x0, // malloc and mfree implemented
            (0x8, _) => 0x0, // accelfunc and accelparam implemented
            (0x9, _) => 0x0, // heap start address implemented
            (0xA, x) => 0x0, // accelfunc `x` implemented
            (0xB, _) => 0x1, // float implemented
            _ => 0x0, // default to 0x0
        };
        self.save(s1, ret)
    }
    /// TODO
    pub fn op_debugtrap(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_getmemsize(&mut self, s1: Save) {
        let mem_size = self.memory.get_mem_size();
        self.save(s1, mem_size)
    }
    /// TODO
    pub fn op_setmemsize(&mut self, l1: u32, s1: Save) {
        let mem_resized = self.memory.set_mem_size(l1);
        self.save(s1, mem_resized)
    }
    /// Jump to address l1 without treating it like an offset.
    pub fn op_jumpabs(&mut self, l1: u32) {
        self.program_counter = l1
    }
    /// TODO
    pub fn op_random(&mut self, l1: i32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_setrandom(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_quit(&mut self) {
        self.running = false
    }
    /// TODO
    pub fn op_verify(&mut self, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_restart(&mut self) {
        unimplemented!()
    }
    /// TODO
    pub fn op_save(&mut self, l1: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_restore(&mut self, l1: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_saveundo(&mut self, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_restoreundo(&mut self, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_protect(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_glk(&mut self, l1: u32, l2: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_getstringtbl(&mut self, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_setstringtbl(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_getiosys(&mut self, s1: Save, s2: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_setiosys(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_linearsearch(&mut self, l1: u32, l2: u32, l3: u32, l4: u32, l5: u32, l6: u32, l7: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_binarysearch(&mut self, l1: u32, l2: u32, l3: u32, l4: u32, l5: u32, l6: u32, l7: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_linkedsearch(&mut self, l1: u32, l2: u32, l3: u32, l4: u32, l5: u32, l6: u32, s1: Save) {
        unimplemented!()
    }
    /// Call the function at l1 and save the result at s1.
    pub fn op_callf(&mut self, l1: u32, s1: Save) {
        self.push_call_stub(s1);
        self.call_func(l1, vec![]);
    }
    /// Call the function at l1 with one input and save the result at s1.
    pub fn op_callfi(&mut self, l1: u32, l2: u32, s1: Save) {
        self.push_call_stub(s1);
        self.call_func(l1, vec![l2]);
    }
    /// Call the function at l1 with two inputs and save the result at s1.
    pub fn op_callfii(&mut self, l1: u32, l2: u32, l3: u32, s1: Save) {
        self.push_call_stub(s1);
        self.call_func(l1, vec![l2, l3]);
    }
    /// Call the function at l1 with three inputs and save the result at s1.
    pub fn op_callfiii(&mut self, l1: u32, l2: u32, l3: u32, l4: u32, s1: Save) {
        self.push_call_stub(s1);
        self.call_func(l1, vec![l2, l3, l4]);
    }
    /// TODO
    pub fn op_mzero(&mut self, l1: u32, l2: u32) {
        self.memory.zero_range(l1, l2)
    }
    /// TODO
    pub fn op_mcopy(&mut self, l1: u32, l2: u32, l3: u32) {
        self.memory.copy_range(l1, l2, l3)
    }
    /// TODO
    pub fn op_malloc(&mut self, l1: u32, s1: Save) {
        unimplemented!()
    }
    /// TODO
    pub fn op_mfree(&mut self, l1: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_accelfunc(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_accelparam(&mut self, l1: u32, l2: u32) {
        unimplemented!()
    }
    /// TODO
    pub fn op_numtof(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1 as f32)
    }
    /// TODO
    pub fn op_ftonumz(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.trunc() as i32)
    }
    /// TODO
    pub fn op_ftonumn(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.round() as i32)
    }
    /// TODO
    pub fn op_ceil(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.ceil())
    }
    /// TODO
    pub fn op_floor(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.floor())
    }
    /// TODO
    pub fn op_fadd(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1 + l2)
    }
    /// TODO
    pub fn op_fsub(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1 - l2)
    }
    /// TODO
    pub fn op_fmul(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1 * l2)
    }
    /// TODO
    pub fn op_fdiv(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1 / l2)
    }
    /// TODO
    pub fn op_fmod(&mut self, l1: f32, l2: f32, s1: Save, s2: Save) {
        let (ret1, ret2) = ((l1 / l2).trunc(), l1 % l2);
        self.save(s1, ret1);
        self.save(s2, ret2)
    }
    /// TODO
    pub fn op_sqrt(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.sqrt())
    }
    /// TODO
    pub fn op_exp(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.exp())
    }
    /// TODO
    pub fn op_log(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.ln())
    }
    /// TODO
    pub fn op_pow(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1.powf(l2))
    }
    /// TODO
    pub fn op_sin(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.sin())
    }
    /// TODO
    pub fn op_cos(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.cos())
    }
    /// TODO
    pub fn op_tan(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.tan())
    }
    /// TODO
    pub fn op_asin(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.asin())
    }
    /// TODO
    pub fn op_acos(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.acos())
    }
    /// TODO
    pub fn op_atan(&mut self, l1: f32, s1: Save) {
        self.save(s1, l1.atan())
    }
    /// TODO
    pub fn op_atan2(&mut self, l1: f32, l2: f32, s1: Save) {
        self.save(s1, l1.atan2(l2))
    }
    /// If l2 is between l1 += l3 jump to l4.
    pub fn op_jfeq(&mut self, l1: f32, l2: f32, l3: f32, l4: u32) {
        let l3 = l3.abs();
        if (l1 + l3 > l2) && (l2 > l1 - l3) { self.op_jump(l4) }
    }
    /// If l2 is not between l1 += l3 jump to l4.
    pub fn op_jfne(&mut self, l1: f32, l2: f32, l3: f32, l4: u32) {
        let l3 = l3.abs();
        if !(l1 + l3 > l2) || !(l2 > l1 - l3) { self.op_jump(l4) }
    }
    /// If l1 is less than l2 jump to l3.
    pub fn op_jflt(&mut self, l1: f32, l2: f32, l3: u32) {
        if l1 < l2 { self.op_jump(l3) }
    }
    /// If l1 is less than or equal to l2 jump to l3.
    pub fn op_jfle(&mut self, l1: f32, l2: f32, l3: u32) {
        if l1 <= l2 { self.op_jump(l3) }
    }
    /// If l1 is greater than l2 jump to l3.
    pub fn op_jfgt(&mut self, l1: f32, l2: f32, l3: u32) {
        if l1 > l2 { self.op_jump(l3) }
    }
    /// If l1 is greater than or equal to l2 jump to l3.
    pub fn op_jfge(&mut self, l1: f32, l2: f32, l3: u32) {
        if l1 >= l2 { self.op_jump(l3) }
    }
    /// If l1 is `f32::NAN` jump to l2.
    pub fn op_jisnan(&mut self, l1: f32, l2: u32) {
        if l1.is_nan() { self.op_jump(l2) }
    }
    /// If l1 is `f32::INFINITY` jump to l2.
    pub fn op_jisinf(&mut self, l1: f32, l2: u32) {
        if l1.is_infinite() { self.op_jump(l2) }
    }

    /// Split the byte at the current program counter address into two
    /// bytes, the first representing the lower 4 bits and the second
    /// representing the upper 4 bits.
    fn lo_hi(&mut self) -> (u8, u8) {
        let bytes: u8 = self.memory.read(self.program_counter);
        self.program_counter += 0x1;
        (bytes & 0x0F, (bytes & 0xF0) >> 0x4)
    }

    /// Return the opcode number from memory, incrementing the program
    /// counter to the end of the opcode, and before operand identifiers.
    fn opcode_number(&mut self) -> u32 {
        let top: u8 = self.memory.read(self.program_counter);
        match top {
            _ if top < 0x80 => {
                //println!("raw opcode: {:#X}", top);
                self.program_counter += 0x1;
                top as u32
            },
            _ if top < 0xC0 => {
                let opcode: u16 = self.memory.read(self.program_counter);
                //println!("raw opcode: {:#X}", opcode);
                self.program_counter += 0x2;
                opcode as u32 - 0x8000
            },
            _ => {
                let opcode: u32 = self.memory.read(self.program_counter);
                //println!("raw opcode: {:#X}", opcode);
                self.program_counter += 0x4;
                opcode - 0xC000_0000
            },
        }
    }

    fn eval(&mut self, opcode: u32) {
        opcode_match!(self, opcode,
            0x00 => op_nop(),
            0x10 => op_add(l1, l2, s1),
            0x11 => op_sub(l1, l2, s1),
            0x12 => op_mul(l1, l2, s1),
            0x13 => op_div(l1, l2, s1),
            0x14 => op_mod(l1, l2, s1),
            0x15 => op_neg(l1, s1),
            0x18 => op_bitand(l1, l2, s1),
            0x19 => op_bitor(l1, l2, s1),
            0x1A => op_bitxor(l1, l2, s1),
            0x1B => op_bitnot(l1, s1),
            0x1C => op_shiftl(l1, l2, s1),
            0x1D => op_sshiftr(l1, l2, s1),
            0x1E => op_ushiftr(l1, l2, s1),
            0x20 => op_jump(l1),
            0x22 => op_jz(l1, l2),
            0x23 => op_jnz(l1, l2),
            0x24 => op_jeq(l1, l2, l3),
            0x25 => op_jne(l1, l2, l3),
            0x26 => op_jlt(l1, l2, l3),
            0x27 => op_jge(l1, l2, l3),
            0x28 => op_jgt(l1, l2, l3),
            0x29 => op_jle(l1, l2, l3),
            0x2A => op_jltu(l1, l2, l3),
            0x2B => op_jgeu(l1, l2, l3),
            0x2C => op_jgtu(l1, l2, l3),
            0x2D => op_jleu(l1, l2, l3),
            0x30 => op_call(l1, l2, s1),
            0x31 => op_return(l1),
            0x32 => op_catch(s1, l1),
            0x33 => op_throw(l1, l2),
            0x34 => op_tailcall(l1, l2),
            0x40 => op_copy(l1, s1),
            0x41 => op_copys(l1, s1),
            0x42 => op_copyb(l1, s1),
            0x44 => op_sexs(l1, s1),
            0x45 => op_sexb(l1, s1),
            0x48 => op_aload(l1, l2, s1),
            0x49 => op_aloads(l1, l2, s1),
            0x4A => op_aloadb(l1, l2, s1),
            0x4B => op_aloadbit(l1, l2, s1),
            0x4C => op_astore(l1, l2, l3),
            0x4D => op_astores(l1, l2, l3),
            0x4E => op_astoreb(l1, l2, l3),
            0x4F => op_astorebit(l1, l2, l3),
            0x50 => op_stkcount(s1),
            0x51 => op_stkpeek(l1, s1),
            0x52 => op_stkswap(),
            0x53 => op_stkroll(l1, l2),
            0x54 => op_stkcopy(l1),
            0x70 => op_streamchar(l1),
            0x71 => op_streamnum(l1),
            0x72 => op_streamstr(l1),
            0x73 => op_streamunichar(l1),
            0x100 => op_gestalt(l1, l2, s1),
            0x101 => op_debugtrap(l1),
            0x102 => op_getmemsize(s1),
            0x103 => op_setmemsize(l1, s1),
            0x104 => op_jumpabs(l1),
            0x110 => op_random(l1, s1),
            0x111 => op_setrandom(l1),
            0x120 => op_quit(),
            0x121 => op_verify(s1),
            0x122 => op_restart(),
            0x123 => op_save(l1, s1),
            0x124 => op_restore(l1, s1),
            0x125 => op_saveundo(s1),
            0x126 => op_restoreundo(s1),
            0x127 => op_protect(l1, l2),
            0x130 => op_glk(l1, l2, s1),
            0x140 => op_getstringtbl(s1),
            0x141 => op_setstringtbl(l1),
            0x148 => op_getiosys(s1, s2),
            0x149 => op_setiosys(l1, l2),
            0x150 => op_linearsearch(l1, l2, l3, l4, l5, l6, l7, s1),
            0x151 => op_binarysearch(l1, l2, l3, l4, l5, l6, l7, s1),
            0x152 => op_linkedsearch(l1, l2, l3, l4, l5, l6, s1),
            0x160 => op_callf(l1, s1),
            0x161 => op_callfi(l1, l2, s1),
            0x162 => op_callfii(l1, l2, l3, s1),
            0x163 => op_callfiii(l1, l2, l3, l4, s1),
            0x170 => op_mzero(l1, l2),
            0x171 => op_mcopy(l1, l2, l3),
            0x178 => op_malloc(l1, s1),
            0x179 => op_mfree(l1),
            0x180 => op_accelfunc(l1, l2),
            0x181 => op_accelparam(l1, l2),
            0x190 => op_numtof(l1, s1),
            0x191 => op_ftonumz(l1, s1),
            0x192 => op_ftonumn(l1, s1),
            0x198 => op_ceil(l1, s1),
            0x199 => op_floor(l1, s1),
            0x1A0 => op_fadd(l1, l2, s1),
            0x1A1 => op_fsub(l1, l2, s1),
            0x1A2 => op_fmul(l1, l2, s1),
            0x1A3 => op_fdiv(l1, l2, s1),
            0x1A4 => op_fmod(l1, l2, s1, s2),
            0x1A8 => op_sqrt(l1, s1),
            0x1A9 => op_exp(l1, s1),
            0x1AA => op_log(l1, s1),
            0x1AB => op_pow(l1, l2, s1),
            0x1B0 => op_sin(l1, s1),
            0x1B1 => op_cos(l1, s1),
            0x1B2 => op_tan(l1, s1),
            0x1B3 => op_asin(l1, s1),
            0x1B4 => op_acos(l1, s1),
            0x1B5 => op_atan(l1, s1),
            0x1B6 => op_atan2(l1, l2, s1),
            0x1C0 => op_jfeq(l1, l2, l3, l4),
            0x1C1 => op_jfne(l1, l2, l3, l4),
            0x1C2 => op_jflt(l1, l2, l3),
            0x1C3 => op_jfle(l1, l2, l3),
            0x1C4 => op_jfgt(l1, l2, l3),
            0x1C5 => op_jfge(l1, l2, l3),
            0x1C8 => op_jisnan(l1, l2),
            0x1C9 => op_jisinf(l1, l2),
        );
    }

    pub fn init(&mut self) {
        let start = self.memory.start_func();
        self.op_call(start, 0x0, Save::Null);
        self.running = true;
    }

    /// Returns flag which indicates whether the quit opcode has been
    /// called. Should be checked every cycle.
    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn step(&mut self) {
        let opcode = self.opcode_number();
        self.eval(opcode);
    }

    pub fn run(&mut self) {
        self.init();
        while self.running {
            self.step();
        }
    }
}


/// Data save location information for an opcode
#[derive(Debug)]
pub enum Save {

    /// Discard result.
    Null,

    /// Save result at the contained memory address.
    Addr(u32),

    /// Push result onto the stack.
    Push,

    /// Save result at the contained stack frame address.
    Frame(u32),

    /// Save result at the contained RAM address.
    Ram(u32),
}


trait ReadRegister<T> {
    fn read_register(&mut self, value: u8) -> T;
}


macro_rules! read_operand {
    (@const $self_:ident, $rtype:ty) => {{
        let data: $rtype = $self_.memory.read($self_.program_counter);
        $self_.program_counter += size_of::<$rtype>() as u32;
        data as _
    }};
    (@addr $self_:ident, $rtype:ty) => {{
        let address: $rtype = $self_.memory.read($self_.program_counter);
        $self_.program_counter += size_of::<$rtype>() as u32;
        $self_.memory.read(address as u32)
    }};
    (@pop $self_:ident, $rtype:ty) => {{
        let value: $rtype = $self_.stack.pop();
        value as _
    }};
    (@frame $self_:ident, $rtype:ty) => {{
        let address: $rtype = $self_.memory.read($self_.program_counter);
        $self_.program_counter += size_of::<$rtype>() as u32;
        $self_.stack.read(address as u32)
    }};
    (@ram $self_:ident, $rtype:ty) => {{
        let address: $rtype = $self_.memory.read($self_.program_counter);
        $self_.program_counter += size_of::<$rtype>() as u32;
        $self_.memory.ram_read(address as u32)
    }}
}


impl ReadRegister<u8> for Glulx {
    fn read_register(&mut self, mode: u8) -> u8 {
        match mode {
            0x0 => 0x0,
            0x1 => read_operand!(@const self, i8),
            0x2 => panic!("cannot load two byte u8"),
            0x3 => panic!("cannot load four byte u8"),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => read_operand!(@addr self, u8),
            0x6 => read_operand!(@addr self, u16),
            0x7 => read_operand!(@addr self, u32),
            0x8 => read_operand!(@pop self, u32),
            0x9 => read_operand!(@frame self, u8),
            0xA => read_operand!(@frame self, u16),
            0xC => panic!("mode 0xC not supported"),
            0xB => read_operand!(@frame self, u32),
            0xD => read_operand!(@ram self, u8),
            0xE => read_operand!(@ram self, u16),
            0xF => read_operand!(@ram self, u32),
            _ => unreachable!(),
        }
    }
}


impl ReadRegister<u16> for Glulx {
    fn read_register(&mut self, mode: u8) -> u16 {
        match mode {
            0x0 => 0x0,
            0x1 => read_operand!(@const self, i8),
            0x2 => read_operand!(@const self, i16),
            0x3 => panic!("cannot load four byte u16"),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => read_operand!(@addr self, u8),
            0x6 => read_operand!(@addr self, u16),
            0x7 => read_operand!(@addr self, u32),
            0x8 => read_operand!(@pop self, u32),
            0x9 => read_operand!(@frame self, u8),
            0xA => read_operand!(@frame self, u16),
            0xB => read_operand!(@frame self, u32),
            0xC => panic!("mode 0xC not supported"),
            0xD => read_operand!(@ram self, u8),
            0xE => read_operand!(@ram self, u16),
            0xF => read_operand!(@ram self, u32),
            _ => unreachable!(),
        }
    }
}


impl ReadRegister<u32> for Glulx {
    fn read_register(&mut self, mode: u8) -> u32 {
        match mode {
            0x0 => 0x0,
            0x1 => read_operand!(@const self, i8),
            0x2 => read_operand!(@const self, i16),
            0x3 => read_operand!(@const self, i32),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => read_operand!(@addr self, u8),
            0x6 => read_operand!(@addr self, u16),
            0x7 => read_operand!(@addr self, u32),
            0x8 => read_operand!(@pop self, u32),
            0x9 => read_operand!(@frame self, u8),
            0xA => read_operand!(@frame self, u16),
            0xB => read_operand!(@frame self, u32),
            0xC => panic!("mode 0xC not supported"),
            0xD => read_operand!(@ram self, u8),
            0xE => read_operand!(@ram self, u16),
            0xF => read_operand!(@ram self, u32),
            _ => unreachable!(),
        }
    }
}


impl ReadRegister<i32> for Glulx {
    fn read_register(&mut self, mode: u8) -> i32 {
        match mode {
            0x0 => 0x0,
            0x1 => read_operand!(@const self, i8),
            0x2 => read_operand!(@const self, i16),
            0x3 => read_operand!(@const self, i32),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => read_operand!(@addr self, u8),
            0x6 => read_operand!(@addr self, u16),
            0x7 => read_operand!(@addr self, u32),
            0x8 => read_operand!(@pop self, i32),
            0x9 => read_operand!(@frame self, u8),
            0xA => read_operand!(@frame self, u16),
            0xB => read_operand!(@frame self, u32),
            0xC => panic!("mode 0xC not supported"),
            0xD => read_operand!(@ram self, u8),
            0xE => read_operand!(@ram self, u16),
            0xF => read_operand!(@ram self, u32),
            _ => unreachable!(),
        }
    }
}


impl ReadRegister<f32> for Glulx {
    fn read_register(&mut self, mode: u8) -> f32 {
        match mode {
            0x0 => 0.0,
            0x1 => panic!("cannot load single byte f32"),
            0x2 => panic!("cannot load two byte f32"),
            0x3 => read_operand!(@const self, f32),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => read_operand!(@addr self, u8),
            0x6 => read_operand!(@addr self, u16),
            0x7 => read_operand!(@addr self, u32),
            0x8 => read_operand!(@pop self, f32),
            0x9 => read_operand!(@frame self, u8),
            0xA => read_operand!(@frame self, u16),
            0xB => read_operand!(@frame self, u32),
            0xC => panic!("mode 0xC not supported"),
            0xD => read_operand!(@ram self, u8),
            0xE => read_operand!(@ram self, u16),
            0xF => read_operand!(@ram self, u32),
            _ => unreachable!(),
        }
    }
}


impl ReadRegister<Save> for Glulx {
    fn read_register(&mut self, mode: u8) -> Save {
        match mode {
            0x0 => Save::Null,
            0x1 => panic!("cannot save to one byte constant"),
            0x2 => panic!("cannot save to two byte constant"),
            0x3 => panic!("cannot save to four byte constant"),
            0x4 => panic!("mode 0x4 not supported"),
            0x5 => Save::Addr(read_operand!(@const self, u8)),
            0x6 => Save::Addr(read_operand!(@const self, u16)),
            0x7 => Save::Addr(read_operand!(@const self, u32)),
            0x8 => Save::Push,
            0x9 => Save::Frame(read_operand!(@const self, u8)),
            0xA => Save::Frame(read_operand!(@const self, u16)),
            0xB => Save::Frame(read_operand!(@const self, u32)),
            0xC => panic!("mode 0xC not supported"),
            0xD => Save::Ram(read_operand!(@const self, u8)),
            0xE => Save::Ram(read_operand!(@const self, u16)),
            0xF => Save::Ram(read_operand!(@const self, u32)),
            _ => unreachable!(),
        }
    }
}


trait SaveRegister<T> {
    fn save(&mut self, save: Save, value: T);
}


impl SaveRegister<u8> for Glulx {
    fn save(&mut self, save: Save, value: u8){
        use self::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value as u32),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl SaveRegister<u16> for Glulx {
    fn save(&mut self, save: Save, value: u16){
        use self::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value as u32),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl SaveRegister<i32> for Glulx {
    fn save(&mut self, save: Save, value: i32){
        use self::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl SaveRegister<u32> for Glulx {
    fn save(&mut self, save: Save, value: u32){
        use self::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}


impl SaveRegister<f32> for Glulx {
    fn save(&mut self, save: Save, value: f32){
        use self::Save::*;

        match save {
            Null => {},
            Addr(ptr) => self.memory.write(ptr, value),
            Push => self.stack.push(value),
            Frame(ptr) => self.stack.write(ptr, value),
            Ram(ptr) => self.memory.ram_write(ptr, value),
        }
    }
}
