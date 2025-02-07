use input::{event::{keyboard::KeyboardEventTrait, EventTrait}, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::{
    env::var,
    fs::File,
    os::{
        fd::AsFd,
        unix::{fs::OpenOptionsExt, prelude::OwnedFd},
    },
    path::Path,
};


struct Interface {
    seat: Option<libseat::Seat>,
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        match &mut self.seat {
            // open device manually
            None => File::options()
                .custom_flags(flags)
                .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
                .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
                .open(path)
                .map(Into::into),
            // open device with seat
            Some(seat) => seat
                .open_device(&path)
                .unwrap()
                .as_fd()
                .try_clone_to_owned(),
        }
        .map_err(|err|err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn main() {
    // opening device manually require user to be in `input` group
    //
    // using `libseat` allow user to bypass it, but it will take over
    // the seat which presumably used by compositor to render the display
    // and prevent it for doing so
    let seat = if matches!(var("USE_SEAT").as_deref(),Ok("1")) {
        let mut seat = libseat::Seat::open(|_,_|{}).unwrap();
        println!("using seat: {:?}",seat.name());
        Some(seat)
    } else {
        None
    };

    let interface = Interface { seat };
    let mut input = Libinput::new_with_udev(interface);
    input.udev_assign_seat("seat0").unwrap();

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            match event {
                input::Event::Keyboard(event) => {
                    println!(
                        "Keyboard key: {:?} {:?}",
                        event.key(),
                        event.key_state(),
                    );
                }
                event => println!("Event {event:?}")
            }
        }
    }
}

