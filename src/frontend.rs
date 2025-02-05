use anyhow::{Context, Result};
use smithay::{
    backend::renderer::element::RenderElementStates,
    desktop::{self, utils::OutputPresentationFeedback, Space, Window},
    input::{pointer::CursorImageStatus, SeatState},
    output::Output,
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    utils::{Logical, Point},
    wayland::{
        compositor::CompositorState,
        dmabuf::{DmabufFeedback, DmabufState},
        shell::xdg::XdgShellState,
        shm::ShmState,
        socket::ListeningSocketSource,
    },
};

use crate::Trayle;

pub struct Frontend {
    pub wlsocket: String,
    pub space: Space<Window>,
    // Globals
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub seat_state: SeatState<Trayle>,
    pub shm_state: ShmState,
    pub dmabuf_state: DmabufState,
}

impl Frontend {
    pub fn setup(dh: &DisplayHandle) -> Result<(Frontend, FrontendSources)> {
        let space = Space::default();

        let socket = ListeningSocketSource::new_auto().context("failed to setup wayland socket")?;
        let wlsocket = socket.socket_name().to_string_lossy().into_owned();
        tracing::info!(name = wlsocket, "listening on wayland socket");

        //
        // setup globals, look for corresponding smithay module for documentation
        //

        let compositor_state = CompositorState::new::<Trayle>(dh);
        let xdg_shell_state = XdgShellState::new::<Trayle>(dh);
        let seat_state = SeatState::new();
        let shm_state = ShmState::new::<Trayle>(dh, vec![]);
        let dmabuf_state = DmabufState::new();

        let sources = FrontendSources {
            socket,
        };

        let frontend = Self {
            wlsocket,
            space,

            // Globals
            compositor_state,
            xdg_shell_state,
            seat_state,
            shm_state,
            dmabuf_state,
        };

        Ok((frontend,sources))
    }

    // pub fn pre_repaint(&mut self, output: &Output, frame_target: impl Into<Time<Monotonic>>) {
    //     let frame_target = Into::<Time<Monotonic>>::into(frame_target);
    //
    //     fn processor(
    //         clients: &mut HashMap<ClientId, Client>,
    //         frame_target: Time<Monotonic>,
    //         surface: &WlSurface,
    //         states: &SurfaceData,
    //     ) {
    //         let Some(mut commit_timer_state) = states
    //             .data_map
    //             .get::<CommitTimerBarrierStateUserData>()
    //             .map(|commit_timer| commit_timer.lock().unwrap())
    //         else {
    //             return;
    //         };
    //
    //         commit_timer_state.signal_until(frame_target);
    //         let client = surface.client().unwrap();
    //         clients.insert(client.id(), client);
    //     }
    //
    //     let mut clients = HashMap::<ClientId, Client>::new();
    //     self.space.elements().for_each(|window| {
    //         window.with_surfaces(|surface, states| {
    //             processor(&mut clients, frame_target, surface, states);
    //         });
    //     });
    //
    //     let map = desktop::layer_map_for_output(output);
    //     for layer_surface in map.layers() {
    //         layer_surface.with_surfaces(|surface,states|{
    //             processor(&mut clients, frame_target, surface, states);
    //         });
    //     }
    //
    //     // map is a mutex lock
    //     drop(map);
    //
    //     // if let CursorImageStatus::Surface(ref surface) = self.cursor_status {
    //     //     desktop::utils::with_surfaces_surface_tree(surface, |surface,states|{
    //     //         processor(&mut clients, frame_target, surface, states);
    //     //     });
    //     // }
    //
    //     // if let Some(surface) = self.dnd_icon.as_ref().map(|icon|&icon.surface) {
    //     //     desktop::utils::with_surfaces_surface_tree(surface, |surface,states|{
    //     //         processor(&mut clients, frame_target, surface, states);
    //     //     });
    //     // }
    //
    //     let dh = self.dh.clone();
    //     for client in clients.into_values() {
    //         self.client_compositor_state(&client).blocker_cleared(self, &dh);
    //     }
    // }

    // pub fn post_repaint(
    //     &mut self,
    //     output: &Output,
    //     time: impl Into<Duration>,
    //     dmabuf_feedback: Option<SurfaceDmabufFeedback>,
    //     render_element_states: &RenderElementStates,
    // ) {
    //     let time = time.into();
    //     let throttle = Some(Duration::from_secs(1));
    //
    //     let mut clients = HashMap::new();
    //
    //     for window in self.space.elements() {
    //         window.with_surfaces(|surface, states|{
    //             let primary_scanout_output = desktop::utils::surface_primary_scanout_output(surface, states);
    //
    //             if let Some(output) = primary_scanout_output.as_ref() {
    //                 smithay::wayland::fractional_scale::with_fractional_scale(states, |fraction_scale|{
    //                     fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
    //                 })
    //             }
    //
    //             if primary_scanout_output
    //                 .as_ref()
    //                 .map(|o|o==output)
    //                 .unwrap_or(true)
    //             {
    //                 let fifo_barrier = states
    //                     .cached_state
    //                     .get::<FifoBarrierCachedState>()
    //                     .current()
    //                     .barrier
    //                     .take();
    //
    //                 if let Some(fifo_barrier) = fifo_barrier {
    //                     fifo_barrier.signal();
    //                     let client = surface.client().unwrap();
    //                     clients.insert(client.id(), client);
    //                 }
    //             }
    //         });
    //
    //         if self.space.outputs_for_element(window).contains(output) {
    //             window.send_frame(output, time, throttle, desktop::utils::surface_primary_scanout_output);
    //             if let Some(dmabuf_feedback) = dmabuf_feedback.as_ref() {
    //                 window.send_dmabuf_feedback(
    //                     output,
    //                     desktop::utils::surface_primary_scanout_output,
    //                     |surface, _| {
    //                         smithay::backend::renderer::element::utils::select_dmabuf_feedback(
    //                             surface,
    //                             render_element_states,
    //                             &dmabuf_feedback.render_feedback,
    //                             &dmabuf_feedback.scanout_feedback,
    //                         )
    //                     }
    //                 );
    //             }
    //         }
    //     }
    //
    //     let map = desktop::layer_map_for_output(output);
    //     for layer_surface in map.layers() {
    //         layer_surface.with_surfaces(|surface, states| {
    //             let primary_scanout_output = desktop::utils::surface_primary_scanout_output(surface, states);
    //
    //             if let Some(output) = primary_scanout_output.as_ref() {
    //                 smithay::wayland::fractional_scale::with_fractional_scale(states, |fraction_scale|{
    //                     fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
    //                 })
    //             }
    //
    //             if primary_scanout_output
    //                 .as_ref()
    //                 .map(|o|o==output)
    //                 .unwrap_or(true)
    //             {
    //                 let fifo_barrier = states
    //                     .cached_state
    //                     .get::<FifoBarrierCachedState>()
    //                     .current()
    //                     .barrier
    //                     .take();
    //
    //                 if let Some(fifo_barrier) = fifo_barrier {
    //                     fifo_barrier.signal();
    //                     let client = surface.client().unwrap();
    //                     clients.insert(client.id(), client);
    //                 }
    //             }
    //         });
    //
    //         layer_surface.send_frame(output, time, throttle, desktop::utils::surface_primary_scanout_output);
    //         if let Some(dmabuf_feedback) = dmabuf_feedback.as_ref() {
    //             layer_surface.send_dmabuf_feedback(
    //                 output,
    //                 desktop::utils::surface_primary_scanout_output,
    //                 |surface, _| {
    //                     smithay::backend::renderer::element::utils::select_dmabuf_feedback(
    //                         surface,
    //                         render_element_states,
    //                         &dmabuf_feedback.render_feedback,
    //                         &dmabuf_feedback.scanout_feedback,
    //                     )
    //                 }
    //             );
    //         }
    //     }
    //
    //     drop(map);
    //
    //     // if let CursorImageStatus::Surface(ref surface) = self.cursor_status {
    //     //     desktop::utils::with_surfaces_surface_tree(surface, |surface, states| {
    //     //         let primary_scanout_output = desktop::utils::surface_primary_scanout_output(surface, states);
    //     //
    //     //         if let Some(output) = primary_scanout_output.as_ref() {
    //     //             smithay::wayland::fractional_scale::with_fractional_scale(states, |fraction_scale|{
    //     //                 fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
    //     //             })
    //     //         }
    //     //
    //     //         if primary_scanout_output
    //     //             .as_ref()
    //     //             .map(|o|o==output)
    //     //             .unwrap_or(true)
    //     //         {
    //     //             let fifo_barrier = states
    //     //                 .cached_state
    //     //                 .get::<FifoBarrierCachedState>()
    //     //                 .current()
    //     //                 .barrier
    //     //                 .take();
    //     //
    //     //             if let Some(fifo_barrier) = fifo_barrier {
    //     //                 fifo_barrier.signal();
    //     //                 let client = surface.client().unwrap();
    //     //                 clients.insert(client.id(), client);
    //     //             }
    //     //         }
    //     //     });
    //     // }
    //
    //     // if let Some(surface) = self.dnd_icon.as_ref().map(|icon| &icon.surface) {
    //     //     desktop::utils::with_surfaces_surface_tree(surface, |surface, states| {
    //     //         let primary_scanout_output = desktop::utils::surface_primary_scanout_output(surface, states);
    //     //
    //     //         if let Some(output) = primary_scanout_output.as_ref() {
    //     //             smithay::wayland::fractional_scale::with_fractional_scale(states, |fraction_scale|{
    //     //                 fraction_scale.set_preferred_scale(output.current_scale().fractional_scale());
    //     //             })
    //     //         }
    //     //
    //     //         if primary_scanout_output
    //     //             .as_ref()
    //     //             .map(|o|o==output)
    //     //             .unwrap_or(true)
    //     //         {
    //     //             let fifo_barrier = states
    //     //                 .cached_state
    //     //                 .get::<FifoBarrierCachedState>()
    //     //                 .current()
    //     //                 .barrier
    //     //                 .take();
    //     //
    //     //             if let Some(fifo_barrier) = fifo_barrier {
    //     //                 fifo_barrier.signal();
    //     //                 let client = surface.client().unwrap();
    //     //                 clients.insert(client.id(), client);
    //     //             }
    //     //         }
    //     //     });
    //     // }
    //
    //     let dh = self.dh.clone();
    //     for client in clients.into_values() {
    //         self.client_compositor_state(&client).blocker_cleared(self, &dh);
    //     }
    // }
}

pub struct FrontendSources {
    pub socket: ListeningSocketSource,
}


pub struct DndIcon {
    pub surface: WlSurface,
    pub offset: Point<i32, Logical>,
}

#[derive(Clone)]
pub struct SurfaceDmabufFeedback {
    pub render_feedback: DmabufFeedback,
    pub scanout_feedback: DmabufFeedback,
}

pub mod setup {
    use super::*;

    pub fn socket() -> Result<(String, ListeningSocketSource)> {
        let socket_source = ListeningSocketSource::new_auto().context("failed to setup wayland socket")?;
        let socket_name = socket_source.socket_name().to_string_lossy().into_owned();
        tracing::info!(name = socket_name, "listening on wayland socket");
        Ok((socket_name,socket_source))
    }
}

pub mod utils {
    use super::*;

    pub fn update_primary_scanout_output(
        space: &Space<Window>,
        output: &Output,
        dnd_icon: Option<&DndIcon>,
        cursor_status: &mut CursorImageStatus,
        render_element_states: &RenderElementStates,
    ) {
        for window in space.elements() {
            window.with_surfaces(|surface,states|{
                desktop::utils::update_surface_primary_scanout_output(
                    surface,
                    output,
                    states,
                    render_element_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare,
                );
            });
        };

        let map = smithay::desktop::layer_map_for_output(output);
        for layer_surface in map.layers() {
            layer_surface.with_surfaces(|surface,states|{
                desktop::utils::update_surface_primary_scanout_output(
                    surface,
                    output,
                    states,
                    render_element_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare,
                );
            });
        }

        if let CursorImageStatus::Surface(ref surface) = cursor_status {
            desktop::utils::with_surfaces_surface_tree(surface, |surface,states|{
                desktop::utils::update_surface_primary_scanout_output(
                    surface,
                    output,
                    states,
                    render_element_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare,
                );
            });
        }

        if let Some(surface) = dnd_icon.as_ref().map(|icon|&icon.surface) {
            desktop::utils::with_surfaces_surface_tree(surface, |surface,states|{
                desktop::utils::update_surface_primary_scanout_output(
                    surface,
                    output,
                    states,
                    render_element_states,
                    smithay::backend::renderer::element::default_primary_scanout_output_compare,
                );
            });
        }
    }

    pub fn take_presentation_feedback(
        output: &Output,
        space: &Space<Window>,
        render_element_states: &RenderElementStates,
    ) -> OutputPresentationFeedback {
        let mut output_presentation_feedback = OutputPresentationFeedback::new(output);

        for window in space.elements() {
            if space.outputs_for_element(window).contains(output) {
                window.take_presentation_feedback(
                    &mut output_presentation_feedback,
                    desktop::utils::surface_primary_scanout_output,
                    |surface,_|
                    desktop::utils::surface_presentation_feedback_flags_from_states(surface,render_element_states)
                );
            }
        }

        output_presentation_feedback
    }
}

pub mod xwayland {
    use super::*;

    pub fn start_xwayland(_trayle: &mut Trayle) -> Result<()> {
        anyhow::bail!("xwayland is not yet implemented")
        // use std::process::Stdio;
        // use smithay::wayland::compositor::CompositorHandler;
        //
        // let (xwayland,client) = XWayland::spawn(
        //     self.dh.clone(),
        //     None,
        //     std::iter::empty::<(String,String)>(),
        //     true,
        //     Stdio::null(),
        //     Stdio::null(),
        //     |_|{},
        // ).context("failed to spawn xwayland server")?;
        //
        // self.backend.loop_handle().insert_source(xwayland, move |event,_,data| match event {
        //     XWaylandEvent::Ready { x11_socket, display_number } => {
        //         let xwayland_scale = std::env::var("TRAYLE_XWAYLAND_SCALE")
        //             .ok()
        //             .and_then(|s|s.parse::<u32>().ok())
        //             .unwrap_or(1);
        //
        //         data.client_compositor_state(&client)
        //             .set_client_scale(xwayland_scale);
        //
        //         let mut wm = match X11Wm::start_wm(data.backend.loop_handle().clone(), x11_socket, client.clone()) {
        //             Ok(ok) => ok,
        //             Err(err) => {
        //                 tracing::error!("failed to attach x11 window manager: {err}");
        //                 return;
        //             },
        //         };
        //
        //         let cursor = Cursor::load();
        //         let image = cursor.get_image(1, Duration::ZERO);
        //         let set_cursor_result = wm.set_cursor(
        //             &image.pixels_rgba,
        //             Size::from((image.width as u16, image.height as u16)),
        //             Point::from((image.xhot as u16, image.yhot as u16)),
        //         );
        //         if let Err(err) = set_cursor_result {
        //             tracing::warn!("failed to set cursor to x11 wm: {err}");
        //             return;
        //         }
        //
        //         data.xwm = Some(wm);
        //         data.xdisplay = Some(display_number);
        //     }
        //     XWaylandEvent::Error => {
        //         tracing::warn!("xwayland server exited unexpectedly during startup")
        //     }
        // }).context("failed to setup xwayland event source: {err}")?;
    }
}



