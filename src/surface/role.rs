use todex::wayland::object::Object;
use todex::wayland::interface::XdgToplevel;

/// Surface role.
#[derive(Debug, Clone, Copy)]
pub enum Role {
    None,
    XdgToplevel(Object<XdgToplevel>),
}

impl Role {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

// ===== Error =====

/// An error that occur in role related operation.
#[derive(Debug)]
pub enum RoleError {
    /// Role is unset.
    Unset,
    /// Role is overwritten.
    Overwrite,
}

impl std::fmt::Display for RoleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unset => write!(f, "role is unset"),
            Self::Overwrite => write!(f, "role is overwritten"),
        }
    }
}
