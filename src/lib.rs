use std::env::Args;

pub struct Modes {
    no_auto: bool,
    fan_mode: u8,
    force: bool,
}

impl Modes {
    pub fn get(mut args_vec: Args) -> Self {
        for args in args_vec.next() {}
        todo!()
    }
}
