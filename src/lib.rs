extern crate byteorder;

mod interpreter;
mod instructions;
mod memory;
mod opcodes;
mod stack;

pub use interpreter::Glulx;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
