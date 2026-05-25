use std::ptr::NonNull;
use std::{mem, slice};

use crate::alloc;
use crate::small_buf::SmallBuf;
use crate::conn::Connection;
use crate::epoll::Epoll;
use crate::objects::Objects;

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

    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 as u32).fmt(f)
    }
}

// ===== Client =====

pub struct Client {
    conn: Connection,
    objects: Objects,
    buffer: SmallBuf,
}

impl Client {
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    pub fn buffer_mut(&mut self) -> &mut SmallBuf {
        &mut self.buffer
    }
}

// ===== Clients =====

pub struct Clients {
    ptr: NonNull<Entry>,
    id: u32,
    len: u32,
    cap: u32,
    /// represent a linked list of deleted entry that ends in one after the last entry
    last_delete: u32,
}

impl Drop for Clients {
    fn drop(&mut self) {
        for entry in self.as_mut_slice() {
            drop(std::mem::replace(entry, Entry::None(0)));
        }
        alloc::deallocate(self.ptr, self.cap);
    }
}

enum Entry {
    Some(Client),
    None(u32)
}

impl Clients {
    pub fn with_capacity(cap: u32) -> Self {
        Self {
            ptr: alloc::allocate(cap),
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

    fn as_mut_slice(&mut self) -> &mut [Entry] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len as usize) }
    }
}

impl Clients {
    pub fn insert(&mut self, conn: Connection, epoll: &Epoll) -> ClientId {
        let id = self.construct_id();
        epoll.add(id, &conn);
        if self.len == self.cap {
            self.cap = alloc::grow(&mut self.ptr, self.cap, 0);
        }
        let new_entry = Entry::Some(Client {
            conn,
            objects: Objects::new(),
            buffer: SmallBuf::new(),
        });
        let entry = unsafe { self.ptr.add(self.last_delete as usize).as_mut() };
        let Entry::None(next_delete) = mem::replace(entry, new_entry) else {
            unreachable!("corrupted clients list");
        };
        if self.last_delete == self.len {
            self.len += 1;
            self.last_delete += 1;
        } else {
            self.last_delete = next_delete;
        }
        self.id = self.id.wrapping_add(1);
        ClientId(id)
    }
}

impl Clients {
    pub fn get_mut(&mut self, id: ClientId) -> Option<&mut Client> {
        let (idx, _) = Self::destruct_id(id.0);
        if idx < self.len {
            match unsafe { self.ptr.add(idx as usize).as_mut() } {
                Entry::Some(state) => Some(state),
                Entry::None(_) => None,
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ClientId, epoll: &Epoll) -> Option<()> {
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
        let entry = unsafe { self.ptr.add(idx as usize).as_mut() };
        let Entry::Some(client) = mem::replace(entry, Entry::None(self.last_delete)) else {
            unsafe { std::hint::unreachable_unchecked() };
        };
        self.last_delete = idx;
        epoll.delete(&client.conn);
        Some(())
    }
}
