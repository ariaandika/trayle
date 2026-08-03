use std::os::fd::{AsFd, BorrowedFd};

use todex::sys::libinput::{EventKind, Keyboard, Libinput};
use todex::sys::udev::Udev;
use todex::sys::xkb::{KeyCode, KeyDirection, KeyboardState, Keymap, KeymapFormat, Xkb};

use crate::error::FatalError;
use crate::log;

// pub use todex::sys::libinput::Event;

pub use self::event::InputEvent;

// ===== mods =====

mod event;

// ===== Input =====

#[expect(dead_code)]
pub struct Input {
    udev: Udev,
    input: Libinput,
    xkb: Xkb,
    keymap: Keymap,
    keyboard: KeyboardState,
}

impl AsFd for Input {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.input.as_fd()
    }
}

impl Input {
    pub fn new() -> Result<Self, FatalError> {
        let udev = Udev::new()?;
        let mut input = Libinput::new_libc(&udev)?;
        input.assign_seat(c"seat0")?;

        let xkb = Xkb::new(<_>::default())?;
        let keymap = Keymap::new_from_names(&xkb, None, KeymapFormat::TextV1, <_>::default())?;
        let keyboard = KeyboardState::new(&keymap)?;

        Ok(Self {
            udev,
            input,
            xkb,
            keymap,
            keyboard,
        })
    }
}

impl Input {
    pub fn dispatch(&mut self) -> InputEvents<'_> {
        if let Err(err) = self.input.dispatch() {
            log::error!("{err}")
        }
        InputEvents::new(self)
    }
}

// ===== InputEvents =====

pub struct InputEvents<'a>(&'a mut Input);

impl<'a> InputEvents<'a> {
    fn new(input: &'a mut Input) -> Self {
        Self(input)
    }
}

impl<'a> Iterator for InputEvents<'a> {
    type Item = InputEvent;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let event = self.0.input.pop_event()?;
            match event.event_kind() {
                EventKind::DeviceAdded => break Some(InputEvent::DeviceAdded(event)),
                EventKind::DeviceRemoved => break Some(InputEvent::DeviceRemoved(event)),
                EventKind::KeyboardKey => {
                    let Ok(kb) = event.try_into_type::<Keyboard>() else {
                        continue;
                    };
                    let dir = match kb.key_state() {
                        0 => KeyDirection::Up,
                        1 => KeyDirection::Down,
                        _ => unreachable!(),
                    };
                    let code = KeyCode::from_linux_keycode(kb.key());
                    self.0.keyboard.update_key(code, dir);
                    break Some(InputEvent::KeyboardKey(kb));
                }
                _ => continue,
            }
        }
    }
}
