//! # Glulx memory mapping
//!
//! The Glulx machine memory consists of a ROM and RAM, which themselves
//! are separated into several different groups.
//!
//!
//! ## Program Provided Values
//!
//! The following values are defined by the given program:
//!
//! * RAMSTART -- Specifies the starting point of the RAM
//! * EXTSTART -- Specifies start of zeroed section of RAM
//! * ENDMEM -- Specifies the ending point of the RAM
//!
//!
//! ## ROM
//!
//! The ROM is defined by the following:
//!
//! * The ROM ranges from 0x00000000 to RAMSTART.
//! * The Header ranges from 0x00000000 to 0x00000024.
//! * The ROM is read-only.
//! * The ROM must be at least 0x100 bytes long.
//! * The ROM usually (but not always) contains all the executable code
//! and constants for the loaded program.
//!
//!
//! ## RAM
//!
//! The RAM is defined by the following:
//!
//! * The RAM ranges from RAMSTART to ENDMEM
//! * The RAM from EXTSTART to ENDMEM is initialized to all zeros.
//! * The RAM can be 0x0 bytes long.
//!
//! ## Misc
//!
//! * RAMSTART, EXTSTART, and ENDMEM must be aligned on 0x100 byte
//! boundries
//! * A Glulx gamefile only stores data from 0x0 to EXTSTART.

use byteorder::{BigEndian, ByteOrder};


/// Glulx magic number
const MAGIC_NUMBER: u32 = 0x476C756C;


/// Minimum supported glulx target
const MIN_VERSION: u32 = 0x0020000;


/// Maximum supported glulx target
const MAX_VERSION: u32 = 0x00301FF;


/// Struct representing a glulx memory object.
pub struct GlulxMemory {
    heap_mode: bool,
    memory: Vec<u8>,
}


impl GlulxMemory {

    /// This takes a rom, validates it to verify that it is a valid
    /// Glulx rom, and retuns either an error string or a `GlulxMemory`
    /// TODO: Replace error string with custom glulx error type.
    pub fn from_rom(rom: Vec<u8>) -> Result<GlulxMemory, String> {
        // Validate magic number.
        let magic_number = BigEndian::read_u32(&rom[0x0..0x4]);
        if magic_number != MAGIC_NUMBER {
            return Err("executable code starts with invalid magic number"
                .to_string());
        }

        // Validate glulx version.
        let glulx_version = BigEndian::read_u32(&rom[0x4..0x8]);
        if glulx_version < MIN_VERSION {
            return Err("executable code glulx version is less than 2.0.0"
                .to_string());
        } else if glulx_version > MAX_VERSION {
            return Err("executable code glulx version is greater than 3.1.x"
                .to_string());
        }

        // Validate ramstart value.
        let ramstart = BigEndian::read_u32(&rom[0x8..0xC]);
        if ramstart < 0x100 {
            return Err("ramstart is less than 0x100 bytes".to_string());
        } else if ramstart % 0x100 != 0 {
            return Err("ramstart is misaligned".to_string());
        }

        // Validate extstart value.
        let extstart = BigEndian::read_u32(&rom[0xC..0x10]);
        if extstart < ramstart {
            return Err("extstart is less than ramstart".to_string());
        } else if extstart % 0x100 != 0 {
            return Err("extstart is misaligned".to_string());
        }

        // Validate endmem value.
        let endmem = BigEndian::read_u32(&rom[0x10..0x14]);
        if endmem < extstart {
            return Err("endmem is less than extstart".to_string());
        } else if endmem % 0x100 != 0 {
            return Err("endmem is misaligned".to_string());
        }

        // Validate stack size.
        let stack_size = BigEndian::read_u32(&rom[0x14..0x18]);
        if stack_size % 0x100 != 0 {
            return Err("stack size is misaligned".to_string());
        }

        if (endmem as usize) < rom.len() {
            return Err("rom size is not correct".to_string());
        }

        let checksum = BigEndian::read_u32(&rom[0x20..0x24]);
        let sum = {
            let mut sum = 0u32;
            for i in 0..rom.len()/4 {
                sum = sum.wrapping_add(BigEndian::read_u32(&rom[i*4..]));
            }
            sum.wrapping_sub(checksum)
        };
        if checksum != sum {
            return Err("rom checksum is not correct".to_string());
        }


        let mut rom = rom;
        let ext_size = (endmem - extstart) as usize;
        rom.reserve_exact(ext_size);
        rom.resize(endmem as usize, 0x0);

        Ok(GlulxMemory { heap_mode: false, memory: rom })
    }

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

    // Header value functions.

    /// The glulx magic number, stored from `0x0..0x4` in the header.
    /// This number is unique to all glulx games. This value should be
    /// `0x476C756C`, which is equivalent to `b"Glul"`.
    fn magic_number(&self) -> u32 {
        BigEndian::read_u32(&self.memory[..0x4])
    }

    /// The glulx version number, stored from `0x4..0x8` in the header.
    /// First two bytes store the major version number. The next byte
    /// stores the minor version number. The final byte stores the patch
    /// version number.
    pub fn glulx_version(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x4..0x8])
    }

    /// The address indicating the start of the RAM, stored from
    /// `0x4..0x8` in the header. TODO: MOAR DATA
    fn ramstart(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x8..0xC])
    }

    fn extstart(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0xC..0x10])
    }

    fn endmem(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x10..0x14])
    }

    pub fn stack_size(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x14..0x18])
    }

    pub fn start_func(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x18..0x1C])
    }

    fn decoding_tbl(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x1C..0x20])
    }

    /// The sum of the intial contents of memory, considered as an array
    /// of `u32`s. When calculated, the checksum value is considered to
    /// be `0`.
    fn checksum(&self) -> u32 {
        BigEndian::read_u32(&self.memory[0x20..0x24])
    }


    pub fn get_mem_size(&self) -> u32 {
        self.memory.len() as u32
    }

    /// Sets the memory size to the given value. This call only works if
    /// there is no active heap, the given value is a multiple of 0x100,
    /// and the value is greater than the ENDMEM value identified in the
    /// header.
    pub fn set_mem_size(&mut self, value: u32) -> u32 {
        if self.heap_mode
                && value % 0x100 == 0
                && value >= self.endmem() {
            self.memory.resize(value as usize, 0x0);
            0
        } else {
            1
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
        self.memory[ptr as usize]
    }

    fn write(&mut self, ptr: u32, value: u8) {
        self.memory[ptr as usize] = value
    }

    fn ram_read(&self, ptr: u32) -> u8 {
        let ptr = (ptr + self.ramstart()) as usize;
        self.memory[ptr]
    }

    fn ram_write(&mut self, ptr: u32, value: u8) {
        let ptr = (ptr + self.ramstart()) as usize;
        self.memory[ptr] = value
    }
}


impl Memory<i8> for GlulxMemory {
    fn read(&self, ptr: u32) -> i8 {
        self.memory[ptr as usize] as i8
    }

    fn write(&mut self, ptr: u32, value: i8) {
        self.memory[ptr as usize] = value as u8
    }

    fn ram_read(&self, ptr: u32) -> i8 {
        let ptr = (ptr + self.ramstart()) as usize;
        self.memory[ptr] as i8
    }

    fn ram_write(&mut self, ptr: u32, value: i8) {
        let ptr = (ptr + self.ramstart()) as usize;
        self.memory[ptr] = value as u8
    }
}


impl Memory<u16> for GlulxMemory {
    fn read(&self, ptr: u32) -> u16 {
        BigEndian::read_u16(&self.memory[ptr as usize..])
    }

    fn write(&mut self, ptr: u32, value: u16) {
        BigEndian::write_u16(&mut self.memory[ptr as usize..], value)
    }

    fn ram_read(&self, ptr: u32) -> u16 {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::read_u16(&self.memory[ptr..])
    }

    fn ram_write(&mut self, ptr: u32, value: u16) {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::write_u16(&mut self.memory[ptr..], value)
    }
}


impl Memory<i16> for GlulxMemory {
    fn read(&self, ptr: u32) -> i16 {
        BigEndian::read_i16(&self.memory[ptr as usize..])
    }

    fn write(&mut self, ptr: u32, value: i16) {
        BigEndian::write_i16(&mut self.memory[ptr as usize..], value)
    }

    fn ram_read(&self, ptr: u32) -> i16 {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::read_i16(&self.memory[ptr..])
    }

    fn ram_write(&mut self, ptr: u32, value: i16) {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::write_i16(&mut self.memory[ptr..], value)
    }
}


impl Memory<u32> for GlulxMemory {
    fn read(&self, ptr: u32) -> u32 {
        BigEndian::read_u32(&self.memory[ptr as usize..])
    }

    fn write(&mut self, ptr: u32, value: u32) {
        BigEndian::write_u32(&mut self.memory[ptr as usize..], value)
    }

    fn ram_read(&self, ptr: u32) -> u32 {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::read_u32(&self.memory[ptr..])
    }

    fn ram_write(&mut self, ptr: u32, value: u32) {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::write_u32(&mut self.memory[ptr..], value)
    }
}


impl Memory<i32> for GlulxMemory {
    fn read(&self, ptr: u32) -> i32 {
        BigEndian::read_i32(&self.memory[ptr as usize..])
    }

    fn write(&mut self, ptr: u32, value: i32) {
        BigEndian::write_i32(&mut self.memory[ptr as usize..], value)
    }

    fn ram_read(&self, ptr: u32) -> i32 {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::read_i32(&self.memory[ptr..])
    }

    fn ram_write(&mut self, ptr: u32, value: i32) {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::write_i32(&mut self.memory[ptr..], value)
    }
}


impl Memory<f32> for GlulxMemory {
    fn read(&self, ptr: u32) -> f32 {
        BigEndian::read_f32(&self.memory[ptr as usize..])
    }

    fn write(&mut self, ptr: u32, value: f32) {
        BigEndian::write_f32(&mut self.memory[ptr as usize..], value)
    }

    fn ram_read(&self, ptr: u32) -> f32 {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::read_f32(&self.memory[ptr..])
    }

    fn ram_write(&mut self, ptr: u32, value: f32) {
        let ptr = (ptr + self.ramstart()) as usize;
        BigEndian::write_f32(&mut self.memory[ptr..], value)
    }
}
