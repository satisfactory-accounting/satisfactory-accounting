//! Helper for dealing with trailing whitespace in input elements.

use std::borrow::Cow;

/// Replaces ascii space characters with &nbsp;
///
/// We need this when using a div to control the size of an input element, because input elements
/// preserve spaces while divs collaps spaces and don't render trailing spaces.
///
/// This doesn't cover every possible type of space but is "good enough". Probably.
pub fn space_to_nbsp(src: &str) -> Cow<str> {
    if src.contains(' ') {
        src.replace(' ', "\u{00a0}").into()
    } else {
        src.into()
    }
}
