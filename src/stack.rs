use byteorder::{ByteOrder, NativeEndian};


pub struct GlulxStack {
    frame_ptr: u32,
    stack: Vec<u8>,
}


impl GlulxStack {
    pub fn new(size: u32) -> GlulxStack {
        GlulxStack {
            frame_ptr: 0x0,
            stack: vec![0x0; size as usize],
        }
    }

    pub fn push_call_stub(&mut self,
            dest_type: u32,
            dest_addr: u32,
            program_counter: u32) {
        self.push(dest_type);
        self.push(dest_addr);
        self.push(program_counter);
        let frame_ptr = self.frame_ptr;
        self.push(frame_ptr);
    }

    pub fn pop_call_stub(&mut self) -> (u32, u32, u32) {
        self.frame_ptr = self.pop();

        let (
            program_counter,
            dest_addr,
            dest_type,
        ) = (self.pop(), self.pop(), self.pop());

        (dest_type, dest_addr, program_counter)
    }

    pub fn pop_call_frame(&mut self) {
        let frame_ptr = self.frame_ptr;
        self.stack.resize(frame_ptr as usize, 0x0);
    }

    pub fn push_call_frame_c0(&mut self,
            locals: Vec<u8>,
            args: Vec<u32>) {
        self.frame_ptr = self.stack.len() as u32;

        let locals_len: u32 = locals.chunks(0x2)
            .map(|pair| pair.into_iter()
                .map(|&val| val as u32)
                .product::<u32>())
            .sum();

        let mut locals = locals;
        if locals.len() & 0x3 != 0 {
            locals.push(0x0);
            locals.push(0x0);
        }
        let locals = locals;

        let local_pos = locals.len() as u32;
        let frame_len = local_pos + locals_len;

        self.push(frame_len);
        self.push(local_pos);
        self.stack.extend(locals);

        for arg in args.into_iter().rev() {
            self.push(arg);
        }
    }

    pub fn push_call_frame_c1(&mut self,
            locals: Vec<u8>,
            args: Vec<u32>) {
        self.frame_ptr = self.stack.len() as u32;

        /// placeholder values
        self.push(0x0u32);
        self.push(0x0u32);

        self.stack.extend(locals.clone());

        while self.stack.len() & 0x3 != 0 {
            self.stack.push(0x0);
        }

        let local_pos = self.stack.len() as u32 - self.frame_ptr;
        NativeEndian::write_u32(
            &mut self.stack[self.frame_ptr as usize + 0x4..],
            local_pos);


        let mut args = args;
        for pair in locals.chunks(0x2) {
            let (local_type, local_count) = (pair[0], pair[1]);

            match local_type {
                1 => {
                    for _ in 0x0..local_count {
                        if args.len() != 0 {
                            self.push(args.pop().unwrap() as u8);
                        } else {
                            self.push(0x0u8);
                        }
                    }
                },
                2 => {
                    if self.stack.len() & 0x1  != 0 { self.stack.push(0x0) };
                    for _ in 0x0..local_count {
                        if args.len() != 0 {
                            self.push(args.pop().unwrap() as u16);
                        } else {
                            self.push(0x0u16);
                        }
                    }
                }
                4 => {
                    while self.stack.len() & 0x3  != 0x0 {
                        self.push(0x0);
                    };
                    for _ in 0x0..local_count {
                        if args.len() != 0 {
                            self.push(args.pop().unwrap());
                        } else {
                            self.push(0x0u32);
                        }
                    }
                },
                _ => {},
            }
        }
        let args = args;

        while self.stack.len() & 0x3 != 0x0 { self.push(0x0) };

        let frame_len = self.stack.len() as u32;
        NativeEndian::write_u32(
            &mut self.stack[self.frame_ptr as usize..],
            frame_len);
    }

    pub fn pop_args(&mut self, nargs: u32) -> Vec<u32> {
        let mut vec = Vec::new();
        for x in 0x0..nargs {
            vec.push(self.pop());
        }
        vec
    }

    pub fn local_pos(&self) -> u32 {
        NativeEndian::read_u32(&self.stack[(self.frame_ptr as usize + 0x4)..])
    }
}


pub trait Stack<T> {
    fn push(&mut self, val: T);
    fn pop(&mut self) -> T;
    fn peek(&self) -> T;
    fn read(& self, offset: u32) -> T;
    fn write(&mut self, offset: u32,  val: T);
}


impl Stack<u8> for GlulxStack {

    /// take a `u8` and push onto the stack.
    fn push(&mut self, val: u8) {
        self.stack.push(val);
    }

    /// pop a `u8` from the stack.
    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }

    fn peek(&self) -> u8 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_u32(&self.stack[pos..]) as u8
    }

    fn read(&self, offset: u32) -> u8 {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        self.stack[pos]
    }

    // Writes a u8 as normal
    fn write(&mut self, offset: u32, val: u8) {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        self.stack[pos] = val;
    }
}


impl Stack<u16> for GlulxStack {

    /// take a `u16` and push onto the stack.
    fn push(&mut self, val: u16) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_u16(&mut self.stack[pos..], val);
    }

    /// pop a `u16` from the stack.
    fn pop(&mut self) -> u16 {
        let pos = self.stack.len() - 0x2;
        let ret = NativeEndian::read_u16(&self.stack[pos..]);
        for _ in 0x0..0x2 { self.stack.pop(); };
        ret
    }

    fn peek(&self) -> u16 {
        let pos = self.stack.len() - 0x2;
        NativeEndian::read_u16(&self.stack[pos..])
    }

    fn read(&self, offset: u32) -> u16 {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::read_u16(&self.stack[pos..])
    }

    // Writes a u16 as normal
    fn write(&mut self, offset: u32, val: u16) {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::write_u16(&mut self.stack[pos..], val)
    }
}


impl Stack<i32> for GlulxStack {
    fn push(&mut self, val: i32) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_i32(&mut self.stack[pos..], val);
    }

    fn pop(&mut self) -> i32 {
        let pos = self.stack.len() - 0x4;
        let ret = NativeEndian::read_i32(&self.stack[pos..]);
        for _ in 0x0..0x4 { self.stack.pop(); };
        ret
    }

    fn peek(&self) -> i32 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_i32(&self.stack[pos..])
    }

    fn read(&self, offset: u32) -> i32 {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::read_i32(&self.stack[pos..])
    }

    fn write(&mut self, offset: u32, val: i32) {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::write_i32(&mut self.stack[pos..], val)
    }
}


impl Stack<u32> for GlulxStack {
    fn push(&mut self, val: u32) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_u32(&mut self.stack[pos..], val);
    }

    fn pop(&mut self) -> u32 {
        let pos = self.stack.len() - 0x4;
        let ret = NativeEndian::read_u32(&self.stack[pos..]);
        for _ in 0x0..0x4 { self.stack.pop(); };
        ret
    }

    fn peek(&self) -> u32 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_u32(&self.stack[pos..])
    }

    fn read(&self, offset: u32) -> u32 {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::read_u32(&self.stack[pos..])
    }

    fn write(&mut self, offset: u32, val: u32) {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::write_u32(&mut self.stack[pos..], val)
    }
}


impl Stack<f32> for GlulxStack {
    /// push a `f32` onto the stack.
    fn push(&mut self, val: f32) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_f32(&mut self.stack[pos..], val);
    }

    /// pop a `f32` off the stack.
    fn pop(&mut self) -> f32 {
        let pos = self.stack.len() - 0x4;
        let ret = NativeEndian::read_f32(&self.stack[pos..]);
        for _ in 0x0..0x4 { self.stack.pop(); };
        ret
    }

    fn peek(&self) -> f32 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_f32(&self.stack[pos..])
    }

    fn read(&self, offset: u32) -> f32 {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::read_f32(&self.stack[pos..])
    }

    fn write(&mut self, offset: u32, val: f32) {
        let pos = (self.frame_ptr + self.local_pos() + offset) as usize;
        NativeEndian::write_f32(&mut self.stack[pos..], val)
    }
}
