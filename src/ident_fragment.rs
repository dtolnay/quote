use proc_macro2::{Ident, Span};
use std::fmt;

/// Specialized formatting trait used by [`format_ident!`].
///
/// [`Ident`] arguments formatted using this trait will have their `r#` prefix
/// stripped, if present.
///
/// See [`format_ident!`] for more information.
pub trait IdentFragment {
    /// Format this value as an identifier fragment.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;

    /// Span associated with this [`IdentFragment`].
    ///
    /// If non-`None`, may be inherited by formatted identifiers.
    #[inline]
    fn span(&self) -> Option<Span> {
        None
    }
}

impl<'a, T: IdentFragment + ?Sized> IdentFragment for &'a T {
    #[inline]
    fn span(&self) -> Option<Span> {
        <T as IdentFragment>::span(*self)
    }

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        IdentFragment::fmt(*self, f)
    }
}

impl<'a, T: IdentFragment + ?Sized> IdentFragment for &'a mut T {
    #[inline]
    fn span(&self) -> Option<Span> {
        <T as IdentFragment>::span(*self)
    }

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        IdentFragment::fmt(*self, f)
    }
}

impl IdentFragment for Ident {
    #[inline]
    fn span(&self) -> Option<Span> {
        Some(self.span())
    }

    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let id = self.to_string();
        if id.starts_with("r#") {
            fmt::Display::fmt(&id[2..], f)
        } else {
            fmt::Display::fmt(&id[..], f)
        }
    }
}

// Limited set of types which this is implemented for, as we want to avoid types
// which will often include non-identifier characters in their `Display` impl.
macro_rules! ident_fragment_display {
    ($($T:ty),*) => {
        $(
            impl IdentFragment for $T {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt::Display::fmt(self, f)
                }
            }
        )*
    }
}

ident_fragment_display!(bool, str, String);
ident_fragment_display!(u8, u16, u32, u64, usize);

#[cfg(integer128)]
ident_fragment_display!(u128);

// XXX: Should we implement `IdentFragment` for signed integers? It's a touch
// inconvenient that the default inferred type for integer literals isn't a
// valid `IdentFragment`.

// XXX: Should `IdentFragment` be implemented for types like `NonZeroUsize`
// ident_fragment_display!(
//     num::NonZeroU8, num::NonZeroU16, num::NonZeroU32,
//     num::NonZeroU64, num::NonZeroU128, num::NonZeroUsize
// );
