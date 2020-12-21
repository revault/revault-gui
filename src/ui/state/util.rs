use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Watch<T> {
    None,
    Some { value: T, updated_at: Instant },
}

impl<T> Watch<T> {
    pub fn new(value: T) -> Self {
        Watch::Some {
            value,
            updated_at: Instant::now(),
        }
    }

    pub fn is_recent(&self, d: std::time::Duration) -> bool {
        match self {
            Watch::None => false,
            Watch::Some { updated_at, .. } => updated_at > &(Instant::now() - d),
        }
    }

    pub const fn as_ref(&self) -> Watch<&T> {
        match *self {
            Watch::Some {
                ref value,
                updated_at,
            } => Watch::Some { value, updated_at },
            Watch::None => Watch::None,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::None;
    }
}

impl<T> From<T> for Watch<T> {
    fn from(v: T) -> Self {
        Watch::new(v)
    }
}

impl<T> From<Watch<T>> for Option<T> {
    fn from(w: Watch<T>) -> Self {
        match w {
            Watch::None => None,
            Watch::Some { value, .. } => Some(value),
        }
    }
}
