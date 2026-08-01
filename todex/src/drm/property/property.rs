/// A pair of property id and value.
#[derive(Debug, Clone, Copy)]
pub struct Property<T> {
    pub id: u32,
    pub value: T,
}

impl<T> Property<T> {
    /// Returns property with the same id but with given value.
    #[inline]
    pub fn with_value<U>(self, value: U) -> Property<U> {
        Property { id: self.id, value }
    }

    /// Map the property value with the same property id.
    #[inline]
    pub fn map_value<F: FnOnce(T) -> U, U>(self, map: F) -> Property<U> {
        Property {
            id: self.id,
            value: map(self.value),
        }
    }
}

impl<T> std::ops::Deref for Property<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Property<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
