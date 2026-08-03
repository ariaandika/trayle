use todex::sys::libinput::{Event, Keyboard};

#[expect(dead_code)]
pub enum InputEvent {
    DeviceAdded(Event),
    DeviceRemoved(Event),
    KeyboardKey(Keyboard)
}
