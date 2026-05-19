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

    /// `(idx, id)`
    pub fn to_parts(self) -> (u32, u32) {
        Clients::destruct_id(self.0)
    }
}

// ===== ClientState =====

pub struct ClientState {
    conn: Connection,
    objects: Objects,
}

// ===== Client =====

#[allow(unused, reason = "TODO")]
pub struct Client {
    id: ClientId,
    state: ClientState,
}

impl Client {
    pub fn id(&self) -> ClientId {
        self.id
    }
}

// ===== ClientMut =====

pub struct ClientMut<'a> {
    id: ClientId,
    state: &'a mut ClientState,
}

impl<'a> ClientMut<'a> {
    pub fn id(&self) -> ClientId {
        self.id
    }

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
    pub fn insert<'a>(
        &'a mut self,
        conn: Connection,
        epoll: &Epoll,
    ) -> crate::errno::Result<ClientMut<'a>> {
        let key = self.construct_id();

        epoll.add_read(key, &conn)?;

        let id = ClientId(key);
        let objects = Objects::new();
        let state = self.insert_inner(ClientState { conn, objects });
        Ok(ClientMut { id, state })
    }

    fn insert_inner(&mut self, client: ClientState) -> &mut ClientState {
        if self.len == self.cap {
            self.cap = self.ptr.grow(self.cap, 0);
        }
        let ptr = self.ptr;
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
        match ptr.as_mut() {
            Entry::Some(state) => state,
            Entry::None(_) => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

impl Clients {
    pub fn get_mut(&mut self, id: ClientId) -> Option<ClientMut<'_>> {
        let (idx, _) = Self::destruct_id(id.0);
        if idx < self.len {
            match self.ptr.add(idx).as_mut() {
                Entry::Some(state) => Some(ClientMut { id, state }),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ClientId, epoll: &Epoll) -> Option<crate::errno::Result<Client>> {
        let state = self.remove_inner(id)?;
        if let Err(err) = epoll.remove(&state.conn) {
            return Some(Err(err));
        }
        Some(Ok(Client { id, state }))
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
