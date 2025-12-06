use std::sync::atomic::{AtomicU32, Ordering};

use crate::objects;
use crate::objects::{Message, ObjectKind};

// defined by protocol
const MAX_CLIENT_ID: u32 = 0xfeffffff;

// ===== GlobalId =====

#[derive(Debug)]
pub struct GlobalId {
    id: AtomicU32,
}

static GLOBAL_ID: GlobalId = GlobalId {
    id: AtomicU32::new(2),
};

impl GlobalId {
    pub fn next() -> u32 {
        GLOBAL_ID.id.fetch_add(1, Ordering::Relaxed)
    }
}

// ===== Manager =====

#[derive(Debug)]
pub struct ObjectManager {
    /// represent available id to be used
    id: u32,
    objects: Box<[Option<ObjectKind>]>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            id: 2,
            objects: Box::new([None; 16]),
        }
    }

    pub fn next_id(&mut self, kind: ObjectKind) -> u32 {
        let id = self.id;
        let index = id.strict_sub(2) as usize;

        if id >= MAX_CLIENT_ID {
            todo!("id overflow, id reuse is not yet implemented")
        }
        if index == self.objects.len() {
            todo!("reallocate")
        }

        assert!(self.objects[index].replace(kind).is_none());
        self.id += 1;
        id
    }

    pub fn event_kind(&self, message: &Message) -> Option<ObjectKind> {
        match message.object_id() {
            0 => None,
            objects::wl_display::OBJECT_ID => Some(ObjectKind::Display),
            index => match self.objects.get(index.wrapping_sub(2) as usize) {
                Some(Some(kind)) => Some(*kind),
                Some(None) | None => None,
            },
        }
    }
}

impl Default for ObjectManager {
    fn default() -> Self {
        Self::new()
    }
}

