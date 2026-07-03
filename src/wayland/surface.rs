use todex::wayland::interface::wl_surface::Damage;
use todex::wayland::object::Handle;

pub struct Surface {
    states: [State; 2],
    current: bool,
    #[expect(dead_code)]
    kind: SurfaceKind,
}

impl Surface {
    pub fn new() -> Self {
        Self {
            states: [State::new(), State::new()],
            current: false,
            kind: SurfaceKind::None,
        }
    }

    fn current_mut(&mut self) -> &mut State {
        &mut self.states[self.current as usize]
    }

    pub fn attach(&mut self, buffer: Option<Handle>) {
        self.current_mut().buffer = buffer;
    }

    pub fn damage(&mut self, damage: Damage) {
        self.current_mut().damage.damage(damage);
    }

    pub fn request_frame(&mut self) {
        let state = self.current_mut();
        state.request_frames = state.request_frames.saturating_add(1);
    }

    pub fn commit(&mut self) {
        self.current = !self.current;
    }
}

// ===== State =====

struct State {
    request_frames: u8,
    buffer: Option<Handle>,
    damage: Region,
    // opaque region,
    // input region,
    // buffer transform,
    // buffer scale,
    // damage buffer,
    // offset
}

impl State {
    pub fn new() -> Self {
        Self {
            request_frames: 0,
            buffer: None,
            damage: Region::new(),
        }
    }
}

// ===== SurfaceKind =====

pub enum SurfaceKind {
    None,
    #[expect(dead_code)]
    XdgToplevel(XdgToplevel),
}

#[expect(dead_code)]
#[derive(Debug)]
pub struct XdgToplevel {
    pub title: Box<str>,
    pub app_id: Box<str>,
}

// ===== Region =====

struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Region {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    fn damage(&mut self, damage: Damage) {
        self.x = damage.x;
        self.y = damage.y;
        self.width = damage.width;
        self.height = damage.height;
    }
}
