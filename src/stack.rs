use byteorder::{ByteOrder, NativeEndian};


pub enum Dest {
    Null,
    Memory(u32),
    Local(u32),
    Push,
}


pub struct Stack {
    frame_ptr: u32,
    local_pos: u32,
    frame_len: u32,
    stack: Vec<u8>,
}


impl Stack {
    pub fn push_frame(mut self, nargs: u32, save: Dest) -> Stack {
        self
    }

    pub fn pop_frame(mut self) -> Stack {
        self
    }
}



impl Stack {
    pub fn read_frame(ptr: u32) -> i32 {
        0
    }


    fn push_call_stub(&mut self,
        program_counter: u32,
        dest_type: u32,
        dest_addr: u32,
    ) {
        self.push_u32(program_counter);
        self.push_u32(dest_type);
        self.push_u32(dest_addr);
        let frame_ptr = self.frame_ptr;
        self.push_u32(frame_ptr);
    }

    fn new_stack_frame(&mut self) {
        self.frame_ptr = self.stack_ptr();
    }

    fn pop_stack_frame(&mut self) {
    }

    fn stack_ptr(&self) -> u32 {
        self.stack.len() as u32
    }

    fn push_u32(&mut self, val: u32){
        let mut buf = vec![0x0; 0x4];
        NativeEndian::write_u32(&mut buf, val);
        self.stack.append(&mut buf);
    }
}
