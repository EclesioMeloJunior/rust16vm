use std::{borrow::Cow, collections::VecDeque, sync::{Arc, Mutex}};

use super::Device;

const BUFFER_EMPTY: u8 = 0;
const BUFFER_NOT_EMPTY: u8 = 1;

pub struct Keyboard {
    buffer: VecDeque<u8>,
    flags: u8,
}

impl Keyboard {
    pub fn new() -> Self {
        let mut keyboard = Self {
            buffer: VecDeque::new(),
            flags: BUFFER_EMPTY,
        };

        async_std::task::spawn(non_blocking_keyboard(Mutex::new(&mut keyboard)));
        keyboard
    }
}

async fn non_blocking_keyboard(keyboard: Mutex<&mut Keyboard>) {
    
}

impl Device for Keyboard {
    fn read(&self, offset: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, offset: usize, value: u8) {
        todo!()
    }
}
