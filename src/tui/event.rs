use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::spawn,
    time::Duration,
};

use ratatui::crossterm::event::{self, KeyEvent};

pub struct EventHandler {
    pub tx: Sender<Events>,
    rx: Receiver<Events>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (tx, rx) = mpsc::channel();
        tx.send(Events::Init).expect("init event could not be sent");
        let sender = tx.clone();

        spawn(move || loop {
            let event: Events = event::poll(tick_rate)
                .ok()
                .and_then(|has_event| if has_event { event::read().ok() } else { None })
                .map(|event| match event {
                    event::Event::Key(k) => Events::Key(k),
                    event::Event::Resize(x, y) => Events::Resize(x, y),
                    _ => Events::Tick,
                })
                .unwrap_or(Events::Tick);
            sender.send(event).unwrap();
        });

        Self { tx, rx }
    }

    pub fn next(&mut self) -> Events {
        self.rx.recv().ok().unwrap_or(Events::Tick)
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub enum Events {
    #[default]
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
    Init,
    Quit,
}
