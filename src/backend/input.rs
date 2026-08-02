use std::os::fd::{AsFd, BorrowedFd};

use todex::sys::libinput::{Capability, Event, EventKind, Keyboard, Libinput};
use todex::sys::udev::Udev;
use todex::sys::xkb::{KeyCode, KeyDirection, KeySym, KeyboardState, Keymap, KeymapFormat, Xkb};

use crate::error::FatalError;
use crate::log;

// ===== Input =====

pub struct Input {
    input: Libinput,
    #[expect(dead_code)]
    xkb: Xkb,
    #[expect(dead_code)]
    keymap: Keymap,
    keyboard: KeyboardState,
}

impl AsFd for Input {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.input.as_fd()
    }
}

impl Input {
    pub fn setup(udev: &Udev) -> Result<Self, FatalError> {
        let mut input = Libinput::new_libc(&udev)?;
        input.assign_seat(c"seat0")?;

        let xkb = Xkb::new(<_>::default())?;
        let keymap = Keymap::new_from_names(&xkb, None, KeymapFormat::TextV1, <_>::default())?;
        let keyboard = KeyboardState::new(&keymap)?;

        Ok(Self {
            input,
            xkb,
            keymap,
            keyboard,
        })
    }
}

impl Input {
    pub fn dispatch(&mut self) -> Result<(), FatalError> {
        self.input.dispatch()?;
        while let Some(event) = self.input.pop_event() {
            self.handle_event(event)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<(), FatalError> {
        let device = event.device_ref();

        match event.event_kind() {
            EventKind::DeviceAdded => {
                for cap in Capability::ENTRIES {
                    if device.has_capability(cap) {
                        log::info!("{:?}: {cap:?}", device.name());
                    }
                }
            }
            EventKind::KeyboardKey => {
                let Ok(kb) = event.try_into_type::<Keyboard>() else {
                    return Ok(());
                };
                let dir = match kb.key_state() {
                    0 => KeyDirection::Up,
                    1 => KeyDirection::Down,
                    _ => unreachable!(),
                };
                let code = KeyCode::from_linux_keycode(kb.key());
                let comp = self.keyboard.update_key(code, dir);
                log::info!("{comp:b}");

                if let Some(sym) = self.keyboard.keysym(code) {
                    if matches!((sym, dir), (KeySym::ESCAPE, KeyDirection::Up)) {
                        return Err(FatalError);
                    }
                }
            }
            _ => {
                log::info!("{:?}: {:?}", device.name(), event);
            }
        }

        Ok(())
    }
}
