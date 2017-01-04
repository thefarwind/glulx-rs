pub struct GlulxMemory {
    memory: Vec<u8>,
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
