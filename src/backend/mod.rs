use todex::sys::epoll::Epoll;
use todex::sys::udev::Udev;

use crate::error::FatalError;
use crate::log;
use crate::poller::Poller;

use input::Input;

mod input;

// ===== Backend =====

pub struct Backend {}

impl Backend {
    pub fn setup() -> Result<Self, FatalError> {
        let epoll = Epoll::new()?;
        let udev = Udev::new()?;

        let mut input = Input::setup(&udev)?;

        const INPUT_KEY: u64 = 1;
        epoll.add(INPUT_KEY, &input);

        let mut poll = Poller::new(64, &epoll);

        loop {
            let Some(event) = poll.next_event() else {
                log::flush();
                poll.wait(None);
                continue;
            };
            match event.key {
                INPUT_KEY => input.dispatch()?,
                key => log::error!("unknown key from epoll: {key}"),
            }
        }
    }
}
