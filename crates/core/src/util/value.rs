#[derive(Clone)]
pub enum Value<'a, T> {
    Owned(T),
    Ref(&'a T),
    None,
}

impl<'a, T> Value<'a, T> {
    pub fn has(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Owned(t) => Some(t),
            Self::Ref(t) => Some(t),
            Self::None => None,
        }
    }

    pub fn set(&mut self, t: T) {
        *self = Value::Owned(t)
    }

    pub fn set_ref(&mut self, t: &'a T) {
        *self = Value::Ref(t)
    }

    pub fn clear(&mut self) {
        *self = Value::None
    }
}
