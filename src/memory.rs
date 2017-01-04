pub struct GlulxMemory {
    memory: Vec<u8>,
}


impl GlulxMemory {
    pub fn zero_range(&mut self, size: u32, ptr: u32){
        for i in ptr as usize..(ptr + size) as usize {
            self.memory[i] = 0;
        }
    }

    pub fn copy_range(&mut self, size: u32, from_ptr: u32, to_ptr: u32){
        let to_ptr = to_ptr as usize;
        let from_ptr = from_ptr as usize;

        for i in 0x0..size as usize {
            self.memory[to_ptr + i] = self.memory[from_ptr + i];
        }
    }
}


pub trait Memory<T> {
    fn read(&self, ptr: u32) -> T;
    fn write(&mut self, ptr: u32, value: T);
    fn ram_read(&self, ptr: u32) -> T;
    fn ram_write(&mut self, ptr: u32, value: T);
}


impl Memory<u8> for GlulxMemory {
    fn read(&self, ptr: u32) -> u8 {
        0
    }

    fn write(&mut self, ptr: u32, value: u8) {
    }

    fn ram_read(&self, ptr: u32) -> u8 {
        0
    }

    fn ram_write(&mut self, ptr: u32, value: u8) {
    }
}

impl Memory<u16> for GlulxMemory {
    fn read(&self, ptr: u32) -> u16 {
        0
    }

    fn write(&mut self, ptr: u32, value: u16) {
    }

    fn ram_read(&self, ptr: u32) -> u16 {
        0
    }

    fn ram_write(&mut self, ptr: u32, value: u16) {
    }
}

impl Memory<u32> for GlulxMemory {
    fn read(&self, ptr: u32) -> u32 {
        0
    }

    fn write(&mut self, ptr: u32, value: u32) {
    }

    fn ram_read(&self, ptr: u32) -> u32 {
        0
    }

    fn ram_write(&mut self, ptr: u32, value: u32) {
    }
}

impl Memory<i32> for GlulxMemory {
    fn read(&self, ptr: u32) -> i32 {
        0
    }

    fn write(&mut self, ptr: u32, value: i32) {
    }

    fn ram_read(&self, ptr: u32) -> i32 {
        0
    }

    fn ram_write(&mut self, ptr: u32, value: i32) {
    }
}

impl Memory<f32> for GlulxMemory {
    fn read(&self, ptr: u32) -> f32 {
        0.
    }

    fn write(&mut self, ptr: u32, value: f32) {
    }

    fn ram_read(&self, ptr: u32) -> f32 {
        0.
    }

    fn ram_write(&mut self, ptr: u32, value: f32) {
    }
}
