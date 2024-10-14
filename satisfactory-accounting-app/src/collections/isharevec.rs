use std::ops::Deref;
use std::rc::Rc;

use implicit_clone::ImplicitClone;

/// Contains a slice of [T] with ownership and cheap cloning.
///
/// Unlike implicit-clone's [IArray][implicit_clone::unsync::IArray], this type is always
/// [ImplicitClone] even if `T` isn't.
pub struct IShareArray<T: 'static>(ShareableArrayTypes<T>);

impl<T: 'static> IShareArray<T> {
    /// A constant empty shareable array.
    pub const EMPTY: IShareArray<T> = IShareArray(ShareableArrayTypes::StaticSlice(&[]));

    /// Get the inner slice.
    pub fn as_slice(&self) -> &[T] {
        self.0.slice()
    }
}

impl<T: 'static + PartialEq> PartialEq for IShareArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<'a, T: 'static> IntoIterator for &'a IShareArray<T> {
    type IntoIter = std::slice::Iter<'a, T>;
    type Item = <std::slice::Iter<'a, T> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T: 'static> AsRef<[T]> for IShareArray<T> {
    fn as_ref(&self) -> &[T] {
        self.0.slice()
    }
}

impl<T: 'static> Deref for IShareArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.slice()
    }
}

impl<T: 'static> Clone for IShareArray<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: 'static> From<&'static [T]> for IShareArray<T> {
    fn from(value: &'static [T]) -> Self {
        Self(ShareableArrayTypes::StaticSlice(value))
    }
}

impl<T: 'static> From<Rc<[T]>> for IShareArray<T> {
    fn from(value: Rc<[T]>) -> Self {
        Self(ShareableArrayTypes::RcSlice(value))
    }
}

impl<T: 'static> From<Rc<Vec<T>>> for IShareArray<T> {
    fn from(value: Rc<Vec<T>>) -> Self {
        Self(ShareableArrayTypes::RcVec(value))
    }
}

impl<T: 'static> ImplicitClone for IShareArray<T> {}

/// Supported shareable array implementation types.
enum ShareableArrayTypes<T: 'static> {
    /// Directly uses a slice.
    StaticSlice(&'static [T]),
    /// Directly holds an `Rc<[T]>` (wide `Rc`).
    RcSlice(Rc<[T]>),
    /// Hold an Rc wrapping a `Vec<T>`. Unlike `Rc<[T]>`, this doesn't require copying the Vec in
    /// order to create it from an existing vector.
    RcVec(Rc<Vec<T>>),
}

impl<T: 'static> ShareableArrayTypes<T> {
    /// Gets the inner slice.
    fn slice(&self) -> &[T] {
        match self {
            Self::StaticSlice(slice) => slice,
            Self::RcSlice(rc) => &*rc,
            Self::RcVec(rc) => &**rc,
        }
    }
}

impl<T: 'static> Clone for ShareableArrayTypes<T> {
    fn clone(&self) -> Self {
        match self {
            Self::StaticSlice(slice) => Self::StaticSlice(slice),
            Self::RcSlice(rc) => Self::RcSlice(Rc::clone(rc)),
            Self::RcVec(rc) => Self::RcVec(Rc::clone(rc)),
        }
    }
}
