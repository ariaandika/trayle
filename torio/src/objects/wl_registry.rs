
#[derive(Debug)]
pub struct Registry {
    object_id: u32,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            object_id: super::GlobalId::next(),
        }
    }

    pub const fn object_id(&self) -> u32 {
        self.object_id
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

