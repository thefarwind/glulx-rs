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
            frame_ptr: 0,
            local_pos: 0,
            frame_len: 0,
            stack: vec![0x0; size as usize],
        }
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
    // TODO: Convert to u32 and push
    fn push(&mut self, val: u8) {
    }

    // TODO: Pop u32 and return as u8
    fn pop(&mut self) -> u8 {
        0
    }

    fn peek(&self) -> u8 {
        0
    }

    fn read(&self, offset: u32) -> u8 {
        0
    }

    // Writes a u8 as normal
    fn write(&mut self, offset: u32, val: u8) {
    }
}


impl Stack<u16> for GlulxStack {
    // TODO: Convert to u32 and push
    fn push(&mut self, val: u16) {
    }

    // TODO: Pop u32 and return as u16
    fn pop(&mut self) -> u16 {
        0
    }

    fn peek(&self) -> u16 {
        0
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
    }

    fn pop(&mut self) -> i32 {
        0
    }

    fn peek(&self) -> i32 {
        0
    }

    fn read(&self, offset: u32) -> i32 {
        0
    }

    fn write(&mut self, offset: u32, val: i32) {
    }
}


impl Stack<u32> for GlulxStack {
    fn push(&mut self, val: u32) {
    }

    fn pop(&mut self) -> u32 {
        0
    }

    fn peek(&self) -> u32 {
        0
    }

    fn read(&self, offset: u32) -> u32 {
        0
    }

    fn write(&mut self, offset: u32, val: u32) {
    }
}


impl Stack<f32> for GlulxStack {
    fn push(&mut self, val: f32) {
    }

    fn pop(&mut self) -> f32 {
        0.
    }

    fn peek(&self) -> f32 {
        0.
    }

    fn read(&self, offset: u32) -> f32 {
        0.
    }

    fn write(&mut self, offset: u32, val: f32) {
    }
}
