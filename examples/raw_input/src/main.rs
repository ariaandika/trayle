//! note that user must in `input` group
//!
//! arch linux: `sudo gpasswd -a [user] input`
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::{
    fs::{File, OpenOptions},
    os::{fd::AsFd, unix::{fs::OpenOptionsExt, prelude::OwnedFd}},
    path::Path,
};



struct Interface {
    session: libseat::Seat,
    // devices: Vec<Device>
}

impl Interface {
    fn new() -> Self {
        let session  = libseat::Seat::open(|_,_|{}).unwrap();
        Self { session }
    }
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        let device = self.session.open_device(&path).unwrap();
        device.as_fd().try_clone_to_owned().map_err(|err|err.raw_os_error().unwrap())
        // OpenOptions::new()
        //     .custom_flags(flags)
        //     .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
        //     .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
        //     .open(path)
        //     .map(Into::into)
        //     .map_err(|err|err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

fn main() {
    let mut input = Libinput::new_with_udev(Interface::new());
    input.udev_assign_seat("seat0").unwrap();
    use std::thread;

    thread::spawn(||{
        thread::sleep(std::time::Duration::from_secs(4));
        std::process::exit(1);
    });

    loop {
        input.dispatch().unwrap();
        for event in &mut input {
            println!("Event {event:?}");
        }
    }
}

