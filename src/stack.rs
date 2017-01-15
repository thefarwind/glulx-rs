use byteorder::{ByteOrder, NativeEndian};


pub struct GlulxStack {
    frame_ptr: u32,
    local_pos: u32,
    frame_len: u32,
    stack: Vec<u8>,
}


impl GlulxStack {
    pub fn new(size: u32) -> GlulxStack {
        GlulxStack {
            frame_ptr: 0x0,
            local_pos: 0x0,
            frame_len: 0x0,
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

        if locals.len() > 0x2 { unimplemented!() };

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
    }

    pub fn pop_args(&mut self, nargs: u32) -> Vec<u32> {
        let mut vec = Vec::new();
        for x in 0x0..nargs {
            vec.push(self.pop());
        }
        vec
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

    /// take a `u8` and push onto the stack as a `u32`.
    fn push(&mut self, val: u8) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_u32(&mut self.stack[pos..], val as u32);
    }

    /// pop a `u32` and return as a `u8`.
    fn pop(&mut self) -> u8 {
        let pos = self.stack.len() - 0x4;
        let ret = NativeEndian::read_u32(&self.stack[pos..]);
        for _ in 0x0..0x4 { self.stack.pop(); };
        ret as u8
    }

    fn peek(&self) -> u8 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_u32(&self.stack[pos..]) as u8
    }

    fn read(&self, offset: u32) -> u8 {
        0
    }

    // Writes a u8 as normal
    fn write(&mut self, offset: u32, val: u8) {
    }
}


impl Stack<u16> for GlulxStack {

    /// take a `u16` and push onto the stack as a `u32`.
    fn push(&mut self, val: u16) {
        let pos = self.stack.len();
        for _ in 0x0..0x4 { self.stack.push(0) };
        NativeEndian::write_u32(&mut self.stack[pos..], val as u32);
    }

    /// pop a `u32` and return as a `u16`.
    fn pop(&mut self) -> u16 {
        let pos = self.stack.len() - 0x4;
        let ret = NativeEndian::read_u32(&self.stack[pos..]);
        for _ in 0x0..0x4 { self.stack.pop(); };
        ret as u16
    }

    fn peek(&self) -> u16 {
        let pos = self.stack.len() - 0x4;
        NativeEndian::read_u32(&self.stack[pos..]) as u16
    }

    fn read(&self, offset: u32) -> u16 {
        0
    }

    // Writes a u16 as normal
    fn write(&mut self, offset: u32, val: u16) {
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
        0
    }

    fn write(&mut self, offset: u32, val: i32) {
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
        0
    }

    fn write(&mut self, offset: u32, val: u32) {
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
        0.
    }

    fn write(&mut self, offset: u32, val: f32) {
    }
}
