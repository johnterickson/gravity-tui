use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use std::time::Duration;


use std::io::stdin;

use ratatui::crossterm::event;

pub enum Event {
    Console(ratatui::crossterm::event::Event),
    DrawInterrupt,
}

pub fn spawn_event_threads() -> (Sender<Event>, Receiver<Event>) {
    let (tx, rx) = channel();

    let tx_key = Sender::clone(&tx);
    let tx_ticker = Sender::clone(&tx);

    thread::spawn(move || keyboard_thread(tx_key));
    thread::spawn(move || redraw_interrupt_thread(tx_ticker));

    (tx, rx)
}

fn keyboard_thread(tx: Sender<Event>) {
    loop {
        let e = event::read();
        if let Ok(e) = e {
            if tx.send(Event::Console(e)).is_err() {
                return;
            }
        } else {
            return;
        }
    }
}

fn redraw_interrupt_thread(tx: Sender<Event>) {
    loop {
        if tx.send(Event::DrawInterrupt).is_err() {
            break;
        };

        thread::sleep(Duration::from_millis(50));
    }
}