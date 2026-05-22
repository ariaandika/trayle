use crate::conn::Connection;
use crate::epoll::Epoll;
use crate::objects::Objects;
use crate::ptr::Ptr;

// ===== ClientId =====

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ClientId(u64);

impl ClientId {
    /// Note that this should only be used to restore id from raw integer.
    ///
    /// To create new id, use `Clients` methods.
    pub fn from_u64(int: u64) -> Self {
        Self(int)
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 as u32).fmt(f)
    }
}

// ===== ClientState =====

pub struct ClientState {
    conn: Connection,
    objects: Objects,
}

// ===== ClientMut =====

pub struct ClientMut<'a> {
    state: &'a mut ClientState,
}

impl<'a> ClientMut<'a> {
    pub fn conn(&self) -> &Connection {
        &self.state.conn
    }

    pub fn objects_mut(&mut self) -> &mut Objects {
        &mut self.state.objects
    }
}

// ===== Clients =====

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
    Some(ClientState),
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

    /// (idx, id)
    const fn construct_id(&self) -> u64 {
        (self.last_delete as u64) << 4 | (self.id as u64)
    }

    /// (idx, id)
    const fn destruct_id(id: u64) -> (u32, u32) {
        debug_assert!(id & i64::MIN as u64 == 0);
        ((id >> 4) as u32, id as u32)
    }
}

impl Clients {
    pub fn add(
        &mut self,
        conn: Connection,
        epoll: &Epoll,
    ) -> ClientId {
        let key = self.construct_id();

        epoll.add_read(key, &conn);

        let id = ClientId(key);
        let objects = Objects::new();
        self.insert_inner(ClientState { conn, objects });
        id
    }

    fn insert_inner(&mut self, client: ClientState) {
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
                unreachable!("invalid entry deletion");
            };
            self.last_delete = next_delete;
        }
        self.id = self.id.wrapping_add(1);
    }
}

impl Clients {
    pub fn get_mut(&mut self, id: ClientId) -> Option<ClientMut<'_>> {
        let (idx, _) = Self::destruct_id(id.0);
        if idx < self.len {
            match self.ptr.add(idx).as_mut() {
                Entry::Some(state) => Some(ClientMut { state }),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ClientId, epoll: &Epoll) -> Option<()> {
        let state = self.remove_inner(id)?;
        epoll.remove(&state.conn);
        Some(())
    }

    fn remove_inner(&mut self, id: ClientId) -> Option<ClientState> {
        if self.len == 0 {
            return None;
        }
        let (idx, _) = Self::destruct_id(id.0);
        if idx >= self.len {
            return None;
        }
        // let Entry::Some(client) = self.ptr.add(idx).as_ref() else {
        //     unreachable!("invalid id, referencing None entry")
        // };
        // if client.id() != id {
        //     dbg!((client.id(), id));
        //     return None;
        // }
        let Entry::Some(client) = self.ptr.add(idx).replace(Entry::None(self.last_delete)) else {
            unsafe { std::hint::unreachable_unchecked() };
        };
        self.last_delete = idx;
        Some(client)
    }
}
