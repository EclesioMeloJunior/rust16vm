use std::{borrow::Cow, collections::VecDeque, sync::{Arc, Mutex}, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode};
use super::Device;

const BUFFER_EMPTY: u8 = 0;
const BUFFER_NOT_EMPTY: u8 = 1;

#[derive(Clone)]
pub struct InnerKeyboard {
    buffer: VecDeque<u8>,
    flags: u8,
}

pub struct Keyboard(Arc<Mutex<InnerKeyboard>>);

impl Keyboard {
    pub fn new() -> Self {
        let inner = Arc::new(Mutex::new(InnerKeyboard {
            buffer: VecDeque::new(),
            flags: BUFFER_EMPTY,
        }));

        async_std::task::spawn(non_blocking_keyboard(inner.clone()));
        Self(inner)
    }
}

async fn non_blocking_keyboard(keyboard: Arc<Mutex<InnerKeyboard>>) {
    loop {
        if poll(Duration::from_millis(0)).unwrap() {
            let evt = read().unwrap();
            match evt {
                Event::Key(key_evt) => match key_evt.code {
                    KeyCode::Char(c) => {
                        {
                            let mut kb = keyboard.lock().unwrap();
                            kb.buffer.push_back(c as u8);
                            kb.flags = BUFFER_NOT_EMPTY;
                        }
                        keyboard.lock().unwrap().buffer.push_back(c as u8);
                    }
                    KeyCode::Esc => return,
                    _ => {}
                } 
                _ => {},
            } 
        }
    } 
}

impl Device for Keyboard {
    fn read(&self, offset: usize) -> u8 {
        println!("keyboard read @ {}", offset);

        match offset {
            0 => self.0.lock().unwrap().flags,
            1 => {
                let mut kb = self.0.lock().unwrap();
                
                if let Some(c) = kb.buffer.pop_front() {
                    if kb.buffer.len() == 0 {
                        kb.flags = BUFFER_EMPTY;
                    }

                    return c;
                }
                
                return 0;
            },
            _ => 0,
        }
    }

    fn write(&mut self, offset: usize, _value: u8) {
        // should do nothing!
        match offset {
            _ => {}
        }
    }
}
