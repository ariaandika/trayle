use std::{process::Command, time::Duration};

use smithay::{
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{Rectangle, Transform},
};

use smithay::backend::{
    renderer::{
        damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
        gles::GlesRenderer,
    },
    winit::{self, WinitEvent},
};

use crate::state::{BackendState, State};

pub struct CalloopData {
    pub state: State,
}

impl BackendState for CalloopData {
    fn state(&self) -> &State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }
}

const REFRESH_RATE: i32 = 60_000;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::<CalloopData>::try_new()?;
    let display = Display::<State>::new()?;

    let state = State::new(&mut event_loop, display);
    let mut data = CalloopData { state };

    //
    // NOTE: final output of the compositor
    //
    let output = Output::new("winit".to_string(), PhysicalProperties {
        size: (0, 0).into(),
        subpixel: Subpixel::Unknown,
        make: "Trayle".to_string(),
        model: "Winit".to_string(),
    });

    let _global = output.create_global::<State>(&mut data.state_mut().display_handle);

    //
    // NOTE: winit backend
    //
    let (mut winit_backend, winit_event_source) = winit::init()?;

    // NOTE: set `WAYLAND_DISPLAY` AFTER `winit::init()`
    // so that `winit` uses **parent** wayland display
    // while spawned processes uses **current** wayland display
    std::env::set_var("WAYLAND_DISPLAY", &data.state.socket_name);

    // NOTE: refresh the output to match winit backend
    let mode = Mode {
        size: winit_backend.window_size(),
        refresh: REFRESH_RATE,
    };
    output.change_current_state(Some(mode), Some(Transform::Flipped180), None, Some((0, 0).into()));
    output.set_preferred(mode);

    // TODO: learn this
    data.state.space.map_output(&output, (0, 0));

    // TODO: learn this
    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    // NOTE: `insert_resource` insert new **EventSource**
    // in this case, is the winit backend
    event_loop.handle().insert_source(winit_event_source, move |event, _, data|{
        let state = data.state_mut();

        match event {
            WinitEvent::Input(event) => state.process_input_event(event),
            WinitEvent::CloseRequested => state.loop_signal.stop(),
            WinitEvent::Focus(_is_focus) => { }
            WinitEvent::Resized { size, .. } => {
                output.change_current_state(
                    Some(Mode { size, refresh: REFRESH_RATE }),
                    None,
                    None,
                    None,
                );
            }

            // TODO: learn this
            WinitEvent::Redraw => {
                let size = winit_backend.window_size();
                let damage = Rectangle::from_size(size);

                winit_backend.bind().unwrap();
                smithay::desktop::space::render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
                    &output,
                    winit_backend.renderer(),
                    1.0,
                    0,
                    [&state.space],
                    &[],
                    &mut damage_tracker,
                    [0.1, 0.1, 0.1, 1.0],
                )
                    .unwrap();

                winit_backend.submit(Some(&[damage])).unwrap();

                state.space.elements().for_each(|window|{
                    window.send_frame(
                        &output,
                        state.start_time.elapsed(),
                        Some(Duration::ZERO),
                        |_, _| Some(output.clone()),
                    );
                });

                state.space.refresh();
                state.popups.cleanup();
                let _ = state.display_handle.flush_clients();

                winit_backend.window().request_redraw();
            }
        }
    })?;

    event_loop.run(None, &mut data, |_|{})?;

    //
    // post setup
    //

    match Command::new("alacritty").spawn() {
        Ok(_child) => {},
        Err(err) => {
            eprintln!("Alacritty spawn error: {err:?}");
        },
    }

    Ok(())
}

