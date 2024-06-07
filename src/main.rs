use enigo::{self, Direction, Keyboard};
use gilrs::{self, Event, EventType};

fn main() {
    let mut gilrs = gilrs::Gilrs::new().unwrap();
    let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();

    let mut hold = gilrs.gamepads().count() != 0;
    while let Some(Event { event, .. }) = gilrs.next_event_blocking(None) {
        hold = match event {
            EventType::Disconnected => false,
            EventType::Connected => true,
            EventType::ButtonPressed(gilrs::Button::LeftTrigger2, _)
            | EventType::ButtonPressed(gilrs::Button::RightTrigger2, _)
            | EventType::ButtonReleased(gilrs::Button::RightTrigger2, _) => !hold,
            _ => continue,
        };
        let dir = if hold {
            Direction::Press
        } else {
            Direction::Release
        };
        enigo.key(enigo::Key::X, dir).unwrap();
    }
}
