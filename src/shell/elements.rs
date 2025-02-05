#![allow(unused_variables)]
use std::cell::{RefCell, RefMut};

use smithay::{
    backend::renderer::{
        element::{
            solid::SolidColorRenderElement, surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        ImportAll, ImportMem, Renderer, Texture,
    },
    desktop::{space::SpaceElement, utils::OutputPresentationFeedback, Window},
    output::Output,
    reexports::{
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale},
    wayland::{compositor::SurfaceData as WlSurfaceData, dmabuf::DmabufFeedback, shell::xdg::ToplevelSurface},
};

use crate::shell::ssd::HEADER_BAR_HEIGHT;

use super::ssd::{HeaderBar, WindowState};

#[derive(Debug, Clone, PartialEq)]
pub struct WindowElement(pub Window);

impl WindowElement {
    pub fn with_surfaces<F>(&self, processor: F) where F: FnMut(&WlSurface, &WlSurfaceData) {
        self.0.with_surfaces(processor);
    }

    pub fn decoration_state(&self) -> RefMut<'_, WindowState> {
        self.0.user_data().insert_if_missing(||{
            RefCell::new(WindowState {
                is_ssd: false,
                header_bar: HeaderBar {
                    pointer_loc: None,
                    width: 0,
                    close_button_hover: false,
                    maximize_button_hover: false,
                    background: Default::default(),
                    close_button: Default::default(),
                    maximize_button: Default::default(),
                }
            })
        });

        self.0.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow_mut()
    }

    pub fn take_presentation_feedback<F1, F2>(
            &self,
            output_feedback: &mut OutputPresentationFeedback,
            primary_scan_out_output: F1,
            presentation_feedback_flags: F2,
        ) where
            F1: FnMut(&WlSurface, &WlSurfaceData) -> Option<Output> + Copy,
            F2: FnMut(&WlSurface, &WlSurfaceData) -> wp_presentation_feedback::Kind + Copy, {
        self.0.take_presentation_feedback(
            output_feedback,
            primary_scan_out_output,
            presentation_feedback_flags
        )
    }

    pub fn send_frame<T, F>(
            &self,
            output: &Output,
            time: T,
            throttle: Option<std::time::Duration>,
            primary_scan_out_output: F,
        ) where
            T: Into<std::time::Duration>,
            F: FnMut(&WlSurface, &WlSurfaceData) -> Option<Output> + Copy, {
        self.0.send_frame(output, time, throttle, primary_scan_out_output)
    }

    pub fn send_dmabuf_feedback<'a, P, F>(
            &self,
            output: &Output,
            primary_scan_out_output: P,
            select_dmabuf_feedback: F,
        ) where
            P: FnMut(&WlSurface, &WlSurfaceData) -> Option<Output> + Copy,
            F: Fn(&WlSurface, &WlSurfaceData) -> &'a DmabufFeedback + Copy, {
        self.0.send_dmabuf_feedback(output, primary_scan_out_output, select_dmabuf_feedback)
    }

    pub fn toplevel(&self) -> Option<&ToplevelSurface> {
        self.0.toplevel()
    }

    pub fn on_commit(&self) {
        self.0.on_commit()
    }
}

impl IsAlive for WindowElement {
    #[inline]
    fn alive(&self) -> bool {
        self.0.alive()
    }
}

impl SpaceElement for WindowElement {
    fn bbox(&self) -> Rectangle<i32, Logical> {
        let mut bbox = self.0.bbox();
        if self.decoration_state().is_ssd {
            bbox.size.h += HEADER_BAR_HEIGHT;
        }
        bbox
    }

    fn is_in_input_region(&self, point: &Point<f64, Logical>) -> bool {
        if self.decoration_state().is_ssd {
            point.y < HEADER_BAR_HEIGHT as f64 ||
            self.0.is_in_input_region(&(*point - Point::from((0., HEADER_BAR_HEIGHT as f64))))
        } else {
            self.0.is_in_input_region(point)
        }
    }

    fn set_activate(&self, activated: bool) {
        self.0.set_activate(activated);
    }

    fn output_enter(&self, output: &Output, overlap: Rectangle<i32, Logical>) {
        self.0.output_enter(output, overlap);
    }

    fn output_leave(&self, output: &Output) {
        self.0.output_leave(output);
    }

    fn z_index(&self) -> u8 {
        self.0.z_index()
    }

    fn refresh(&self) {
        self.0.refresh();
    }

    fn geometry(&self) -> Rectangle<i32, Logical> {
        let mut geo = self.0.geometry();
        if self.decoration_state().is_ssd {
            geo.size.h += HEADER_BAR_HEIGHT;
        }
        geo
    }
}

impl<R> AsRenderElements<R> for WindowElement
where
    R: Renderer + ImportAll + ImportMem,
    <R as Renderer>::TextureId: Clone + Texture + 'static,
{
    type RenderElement = WindowRenderElement<R>;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        mut location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        let window_bbox = SpaceElement::bbox(&self.0);

        if self.decoration_state().is_ssd && window_bbox.is_empty() {
            let window_geo = SpaceElement::geometry(&self.0);

            let mut state = self.decoration_state();
            let width = window_geo.size.w;
            state.header_bar.redraw(width as u32);
            let mut vec = AsRenderElements::<R>::render_elements::<WindowRenderElement<R>>(
                &state.header_bar,
                renderer,
                location,
                scale,
                alpha
            );

            location.y += (scale.y * HEADER_BAR_HEIGHT as f64) as i32;

            let window_elements = AsRenderElements::<R>::render_elements(
                &self.0,
                renderer,
                location,
                scale,
                alpha
            );
            vec.extend(window_elements);
            vec.into_iter().map(C::from).collect()
        } else {
            AsRenderElements::<R>::render_elements(
                &self.0,
                renderer,
                location,
                scale,
                alpha
            )
                .into_iter()
                .map(C::from)
                .collect()
        }
    }
}

smithay::render_elements! {
    pub WindowRenderElement<R> where R: ImportAll + ImportMem;
    Window=WaylandSurfaceRenderElement<R>,
    Decoration=SolidColorRenderElement,
}

