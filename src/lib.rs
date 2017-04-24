extern crate byteorder;

mod interpreter;
mod memory;
mod stack;
pub mod io;

pub use interpreter::Glulx;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
