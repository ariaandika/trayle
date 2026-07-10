pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Region {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn add(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.width += other.width;
        self.height += other.height;
    }

    pub fn subtract(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.width -= other.width;
        self.height -= other.height;
    }

    pub fn union(&mut self, other: Self) {
        self.x = other.x.max(self.x);
        self.y = other.y.max(self.y);
        self.width = other.width.max(self.width);
        self.height = other.height.max(self.height);
    }
}
