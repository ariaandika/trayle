use smithay::{
    backend::renderer::{
        element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            surface::WaylandSurfaceRenderElement,
            AsRenderElements, Kind,
        }, Color32F, ImportAll, ImportMem, Renderer, Texture
    },
    input::pointer::CursorImageStatus,
    utils::{Physical, Point, Scale},
};

pub static CLEAR_COLOR: Color32F = Color32F::new(0.8, 0.2, 0.1, 1.0);
pub static CLEAR_COLOR_FULLSCREEN: Color32F = Color32F::new(0., 0., 0., 0.);

pub struct PointerElement {
    buffer: Option<MemoryRenderBuffer>,
    status: CursorImageStatus,
}

impl PointerElement {
    pub fn set_status(&mut self, status: CursorImageStatus) {
        self.status = status;
    }
    pub fn set_buffer(&mut self, buffer: MemoryRenderBuffer) {
        self.buffer = Some(buffer);
    }
}

impl Default for PointerElement {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            status: CursorImageStatus::default_named()
        }
    }
}

impl<T: Texture + Clone + Send + 'static, R> AsRenderElements<R> for PointerElement
where
    R: Renderer<TextureId = T> + ImportAll + ImportMem,
{
    type RenderElement = PointerRenderElement<R>;
    fn render_elements<E>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<E>
    where
        E: From<PointerRenderElement<R>>
    {
        match &self.status {
            CursorImageStatus::Hidden => vec![],
            CursorImageStatus::Named(_) => {
                match self.buffer.as_ref() {
                    Some(buffer) => vec![
                        PointerRenderElement::<R>::from(
                            MemoryRenderBufferRenderElement::from_buffer(
                                renderer,
                                location.to_f64(),
                                buffer,
                                None,
                                None,
                                None,
                                Kind::Cursor
                            )
                            .expect("lost system pointer buffer")
                        )
                        .into()
                    ],
                    None => vec![],
                }
            }
            CursorImageStatus::Surface(surface) => {
                let elements = smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                    renderer,
                    surface,
                    location,
                    scale,
                    alpha,
                    Kind::Cursor
                );
                elements.into_iter().map(E::from).collect()
            }
        }
    }
}

smithay::render_elements! {
    pub PointerRenderElement<R> where R: ImportAll + ImportMem;
    Surface=WaylandSurfaceRenderElement<R>,
    Memory=MemoryRenderBufferRenderElement<R>,
}

