pub trait SubsystemManager: Subsystem {
    /// sets the active subsystem to the given mode and rock.
    fn set_io_subsystem(&mut self, mode: u32, rock: u32);

    /// Returns the currently used mode and rock.
    fn get_io_subsystem(&self) -> (u32, u32);

    /// Returns a value indicating whether the specified mode
    /// is supported by the SubsytemManager.
    ///
    /// The checked codes are as follows:
    ///
    /// * `0x0`: `Null` `Subsystem`
    /// * `0x1`: `Filter` `Subsystem`
    /// * `0x2`: Glk `Subsystem`
    /// * `0x3`: FyreVM `Subsystem`
    fn gestalt_io_subsystem(&self, mode: u16) -> u32;
}


pub trait Subsystem {
    fn tick(&mut self);
    fn quit(&mut self);
}


#[derive(Default)]
pub struct Filter {
    pub rock: u32,
}


impl Filter {
    pub fn new(rock: u32) -> Filter {
        Filter{ rock: rock }
    }
}


impl Subsystem for Filter {
    fn tick(&mut self){ /* NOP */ }
    fn quit(&mut self){ /* NOP */ }
}


#[derive(Default)]
pub struct Null;


impl Subsystem for Null {
    fn tick(&mut self){ /* NOP */ }
    fn quit(&mut self){ /* NOP */ }
}


pub struct DefaultManager {
    mode: u32,
    rock: u32,
    subsystem: Box<Subsystem>,
}


impl Default for DefaultManager {
    fn default() -> DefaultManager {
        DefaultManager{
            mode: 0x0,
            rock: 0x0,
            subsystem: Box::new(Null::default()),
        }
    }
}


impl SubsystemManager for DefaultManager {
    fn set_io_subsystem(&mut self, mode: u32, rock: u32) {
        self.subsystem = match mode {
            0x0 => Box::new(Null::default()),
            0x1 => Box::new(Filter::new(rock)),
            _ => panic!("unsupported io subsystem requested"),
        };
        self.mode = mode;
        self.rock = rock;
    }

    fn get_io_subsystem(&self) -> (u32, u32) {
        (self.mode, self.rock)
    }

    fn gestalt_io_subsystem(&self, mode: u16) -> u32 {
        if let 0x0...0x1 = mode { 0x1 } else { 0x0 }
    }

}

impl Subsystem for DefaultManager {
    fn tick(&mut self){ /* NOP */ }
    fn quit(&mut self){ /* NOP */ }
}
