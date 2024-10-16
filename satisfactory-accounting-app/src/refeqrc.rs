use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;

/// [`Rc<T>`][Rc], but with reference equality semantics. Supports [`PartialEq`] and [`Eq`] even if T does
/// not. Note that this make equality checks more pessimistic -- if [`RefEqRc`] compares equal, the
/// contents usually will, except in case of things like NaN which never compare equals, however if
/// two [`RefEqRc`] compare unequal, that doesn't tell you much about whether the contents will
/// compare equal.
#[derive(Debug)]
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

    /// Get a pointer to the underlying T. Same as Rc::as_ptr.
    #[inline]
    pub fn as_ptr(rerc: &RefEqRc<T>) -> *const T {
        Rc::as_ptr(&rerc.rc)
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

impl<T> Eq for RefEqRc<T> {}

/// Because RefEqRc uses reference equality for [`PartialEq`], it also implements [`Hash`] using its
/// pointer address.
impl<T> Hash for RefEqRc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Self::as_ptr(self) as usize);
    }
}

impl<T> Deref for RefEqRc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}
