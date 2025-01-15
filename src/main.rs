use std::process::Command;
use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::Display
};
use state::{CalloopData, State};

mod state;
mod winit;
mod input;
mod handlers;
mod grabs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::<CalloopData>::try_new()?;

    let display = Display::<State>::new()?;
    let display_handle = display.handle();
    let state = State::new(&mut event_loop, display);

    let mut data = CalloopData::new(state, display_handle);

    crate::winit::init(&mut event_loop, &mut data)?;


    let res = Command::new("alacritty").spawn();
    match res {
        Ok(_child) => {},
        Err(err) => {
            eprintln!("Alacritty spawn error: {err:?}");
        },
    }

    event_loop.run(None, &mut data, on_loop)?;

    Ok(())
}

fn on_loop(_state: &mut CalloopData) {
    
}

