use rdev::{listen, simulate, EventType, Key};
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        listen(move |event| {
            tx.send(match event.event_type {
                EventType::KeyPress(Key::KeyA) => 0,
                EventType::KeyRelease(Key::KeyA) => 1,
                _ => return,
            })
            .unwrap();
        })
        .unwrap();
    });
    for event in rx {
        simulate(&match event {
            1 => EventType::KeyPress(Key::KeyX),
            0 => EventType::KeyRelease(Key::KeyX),
            _ => continue,
        })
        .unwrap();
    }
}
