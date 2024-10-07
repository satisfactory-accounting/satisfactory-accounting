use std::ops::Deref;
use std::rc::Rc;

/// Rc<T>, but with reference equality semantics. Supports PartialEq and Eq even if T does not.
#[derive(Debug, Eq)]
#[repr(transparent)]
pub struct RefEqRc<T> {
    rc: Rc<T>,
}

impl<T> RefEqRc<T> {
    /// Create a new RefEqRc containing the given value.
    #[inline]
    pub fn new(val: T) -> Self {
        Self { rc: Rc::new(val) }
    }
}

impl<T> Clone for RefEqRc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            rc: <Rc<T> as Clone>::clone(&self.rc),
        }
    }
}

impl<T> PartialEq for RefEqRc<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.rc, &other.rc)
    }
}

impl<T> Deref for RefEqRc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}
