use std::{
    default,
    io::{self, const_error},
    u16,
};

use async_std::task::JoinHandle;
use crossbeam_channel::{TryRecvError, unbounded};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph, Widget, WidgetRef},
};

const CHANNEL_READ_ERROR: io::Error =
    const_error!(io::ErrorKind::BrokenPipe, "fail to read from channel");

use super::Device;
use crate::machine::{Instruction, Register};

struct InstructionsPane {
    code: Vec<u16>,
    current_instruction: usize,
}

impl InstructionsPane {
    fn new(insts: Vec<u16>, curr_inst: usize) -> Self {
        Self {
            code: insts,
            current_instruction: curr_inst,
        }
    }
}

impl WidgetRef for InstructionsPane {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Previous ".into(),
            "<Left>".blue().bold(),
            " Next ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            .title("Current Instruction")
            .title_bottom(instructions);

        let mut window_of_instructions = vec![];
        if self.current_instruction < 4 {
            for (idx, inst) in self.code[0..10].iter().enumerate() {
                window_of_instructions.push((inst.clone(), idx == self.current_instruction));
            }
        } else {
            let prev = self.current_instruction - 2;
            let next = std::cmp::min(self.code.len(), self.current_instruction + 8);

            for (idx, inst) in self.code[prev..next].iter().enumerate() {
                window_of_instructions.push((inst.clone(), idx == 2));
            }
        }

        let items = window_of_instructions
            .iter()
            .map(|(inst, current)| (Instruction::try_from(*inst).unwrap(), current))
            .map(|(inst, current)| {
                let inst = if *current {
                    Span::styled(inst.to_string(), Style::default().fg(Color::Yellow))
                } else {
                    Span::styled(inst.to_string(), Style::default().fg(Color::White))
                };

                Line::from(vec![inst])
            })
            .map(ListItem::new)
            .collect::<Vec<ListItem>>();

        List::new(items).block(block).render(area, buf);
    }
}

struct Screen {
    rx: crossbeam_channel::Receiver<ScreenExchange>,
    exit: bool,
    instructions_pane: Option<InstructionsPane>,
}

impl Screen {
    fn new(rx: crossbeam_channel::Receiver<ScreenExchange>) -> Self {
        Self {
            rx,
            exit: false,
            instructions_pane: None,
        }
    }

    async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            match self.rx.try_recv() {
                Ok(msg) => {
                    match msg {
                        _ => {}
                    };
                }
                Err(TryRecvError::Empty) => {}
                Err(err) => {
                    eprintln!("screen device error: {}", err);
                    self.exit = true;
                    return Err(CHANNEL_READ_ERROR);
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_evt) if key_evt.kind == KeyEventKind::Press => {
                self.handle_key_press(key_evt.code)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_press(&mut self, kc: KeyCode) {
        match kc {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left => {
                if let Some(ref mut pane) = self.instructions_pane {
                    if pane.current_instruction > 0 {
                        pane.current_instruction -= 1;
                    }
                }
            }
            KeyCode::Right => {
                if let Some(ref mut pane) = self.instructions_pane {
                    if pane.current_instruction < (pane.code.len() - 1) {
                        pane.current_instruction += 1;
                    }
                }
            }
            _ => {}
        };
    }
}

impl Widget for &Screen {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [left, right] = Layout::horizontal([Constraint::Fill(1); 2]).areas(area);
        let [top_right, center_right, bottom_right] =
            Layout::vertical([Constraint::Fill(1); 3]).areas(right);

        Block::bordered().title("Terminal").render(left, buf);

        Block::bordered().title("Registers").render(top_right, buf);
        Block::bordered().title("Logs").render(center_right, buf);

        if let Some(ref instructions_pane) = self.instructions_pane {
            instructions_pane.render(bottom_right, buf);
        }
    }
}

#[derive(Default)]
pub struct ScreenOptions {
    show_instructions: Option<(Vec<u16>, usize)>,
    show_registers: Option<Vec<Register>>,
}

impl ScreenOptions {
    pub fn debug_instructions(&mut self, inst: Vec<u16>, idx: usize) -> &mut Self {
        self.show_instructions = Some((inst, idx));
        self
    }
}

async fn screen_main(
    opts: ScreenOptions,
    rx: crossbeam_channel::Receiver<ScreenExchange>,
) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut screen = Screen::new(rx);

    if let Some((instructions, current_inst)) = opts.show_instructions {
        screen.instructions_pane = Some(InstructionsPane::new(instructions, current_inst));
    }

    let res = screen.run(&mut terminal).await;
    ratatui::restore();
    res
}

enum ScreenExchange {
    Read(usize, crossbeam_channel::Sender<u8>),
    Write((usize, u8)),
}

pub struct ScreenDevice {
    tx: crossbeam_channel::Sender<ScreenExchange>,
    device_handle: JoinHandle<io::Result<()>>,
}

impl ScreenDevice {
    pub fn start() -> Self {
        let (tx, rx) = unbounded::<ScreenExchange>();
        let handle = async_std::task::spawn(screen_main(Default::default(), rx));

        ScreenDevice {
            tx,
            device_handle: handle,
        }
    }

    pub fn start_and_debug(opts: ScreenOptions) -> Self {
        let (tx, rx) = unbounded::<ScreenExchange>();
        let handle = async_std::task::spawn(screen_main(opts, rx));

        ScreenDevice {
            tx,
            device_handle: handle,
        }
    }
}

impl Device for ScreenDevice {
    fn read(&self, offset: usize) -> u8 {
        todo!()
    }

    fn write(&mut self, offset: usize, value: u8) {
        todo!()
    }
}
