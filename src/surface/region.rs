use todex::wayland::interface::wl_surface::Damage;

pub struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Region {
    pub(super) fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub(super) fn damage(&mut self, damage: Damage) {
        self.x = damage.x;
        self.y = damage.y;
        self.width = damage.width;
        self.height = damage.height;
    }
}
