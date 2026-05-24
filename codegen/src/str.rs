use std::rc::Rc;

#[derive(Clone, Eq)]
pub struct Str {
    value: &'static str,
    shared: Option<Rc<Box<str>>>,
}

impl Str {
    pub fn new(string: String) -> Self {
        let value = Box::leak(string.into_boxed_str());
        // SAFETY: `value` just created right above, but `value` is also copied to other place, this
        // is fine because its immutable reference and the lifetime is managed by `Rc`, it will
        // still valid as long as there is `Str` alive
        let boxed = unsafe { Box::from_raw(value) };
        Self {
            shared: Some(Rc::new(boxed)),
            value,
        }
    }

    pub const fn from_static(string: &'static str) -> Self {
        Self {
            shared: None,
            value: string,
        }
    }

    pub fn advance(&mut self, cnt: usize) {
        self.value = &self.value[cnt..];
    }

    pub fn split_to(&mut self, len: usize) -> Self {
        let (prefix, suffix) = self.value.split_at(len);
        self.value = suffix;
        Self {
            value: prefix,
            shared: self.shared.clone(),
        }
    }

    pub fn slice<R: std::ops::RangeBounds<usize>>(&self, range: R) -> Self {
        let start = match range.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(&n) => n + 1,
            std::ops::Bound::Excluded(&n) => n,
            std::ops::Bound::Unbounded => self.value.len(),
        };
        Self {
            value: &self.value[start..end],
            shared: self.shared.clone(),
        }
    }

    pub fn trim_start(self) -> Self {
        Self {
            value: self.value.trim_start(),
            shared: self.shared,
        }
    }
}

impl std::ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl From<String> for Str {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&'static str> for Str {
    fn from(value: &'static str) -> Self {
        Self::from_static(value)
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl std::hash::Hash for Str {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq<str> for Str {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}
