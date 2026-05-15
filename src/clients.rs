use crate::client::Client;
use crate::ptr::Ptr;

pub struct Clients {
    ptr: Ptr<Entry>,
    id: u32,
    len: u32,
    cap: u32,
    /// represent a linked list of deleted entry that ends in one after the last entry
    last_delete: u32,
}

impl Drop for Clients {
    fn drop(&mut self) {
        for entry in self.ptr.as_mut_slice(self.len) {
            drop(std::mem::replace(entry, Entry::None(0)));
        }
        self.ptr.deallocate(self.cap);
    }
}

enum Entry {
    Some(Client),
    None(u32)
}

impl Clients {
    pub fn with_capacity(cap: u32) -> Self {
        Self {
            ptr: Ptr::allocate(cap),
            id: 0,
            len: 0,
            cap,
            last_delete: 0,
        }
    }

    /// Returns id that will be used for the next `insert`.
    pub const fn peek_id(&self) -> u64 {
        (self.last_delete as u64) << 4 | (self.id as u64)
    }

    /// (idx, id)
    pub const fn destruct_id(id: u64) -> (u32, u32) {
        debug_assert!(id & !(u64::MAX >> 1) == 0);
        ((id >> 4) as u32, id as u32)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Client> {
        let (idx, _) = Self::destruct_id(id);
        if idx < self.len {
            match self.ptr.add(idx).as_mut() {
                Entry::Some(client) => Some(client),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, client: Client) {
        if self.len == self.cap {
            self.cap = self.ptr.grow(self.cap, 0);
        }
        if self.last_delete == self.len {
            self.ptr.add(self.last_delete).write(Entry::Some(client));
            self.len += 1;
            self.last_delete += 1;
        } else {
            let Entry::None(next_delete) =
                self.ptr.add(self.last_delete).replace(Entry::Some(client))
            else {
                unreachable!("invalid replacement of deleted entry");
            };
            self.last_delete = next_delete;
        }
        self.id += 1;
    }

    pub fn remove(&mut self, id: u64) -> Option<Client> {
        if self.len == 0 {
            return None;
        }
        let (idx, _) = Self::destruct_id(id);
        if idx >= self.len {
            dbg!((idx, self.len));
            return None;
        }
        let Entry::Some(client) = self.ptr.add(idx).as_ref() else {
            unreachable!("invalid id, referencing None entry")
        };
        if client.id() != id {
            dbg!((client.id(), id));
            return None;
        }
        let Entry::Some(client) = self.ptr.add(idx).replace(Entry::None(self.last_delete)) else {
            unreachable!();
        };
        self.last_delete = idx;
        Some(client)
    }
}
