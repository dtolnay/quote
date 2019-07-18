/// Formatting macro for constructing [`Ident`]s.
///
/// [`Ident`]: `proc_macro2::Ident`
///
/// # Syntax
///
/// Syntax is copied from the [`format!`] macro, supporting both positional and
/// named arguments.
///
/// Only a limited set of formatting traits are supported. The current mapping
/// of format types to traits is:
///
/// * *nothing* ⇒ [`IdentFragment`]
/// * `o` ⇒ [`Octal`](`std::fmt::Octal`)
/// * `x` ⇒ [`LowerHex`](`std::fmt::LowerHex`)
/// * `X` ⇒ [`UpperHex`](`std::fmt::UpperHex`)
/// * `b` ⇒ [`Binary`](`std::fmt::Binary`)
///
/// See [`std::fmt`] for more information.
///
/// # IdentFragment
///
/// Unlike [`format!`], this macro uses the [`IdentFragment`] formatting trait
/// by default. This trait is like `Display`, with a few differences:
///
/// * `IdentFragment` is only implemented for a limited set of types, such as
///    unsigned integers and strings.
/// * [`Ident`] arguments will have their `r#` prefixes stripped, if present.
///
/// # Hygiene
///
/// The [`Span`] of the first [`Ident`] argument is used as the span of the
/// final identifier, falling back to [`Span::call_site`] when no identifiers
/// are provided.
///
/// ```edition2018
/// # use quote::format_ident;
/// # let ident = format_ident!("Ident");
/// // If `ident` is an Ident, the span of `my_ident` will be inherited from it.
/// let my_ident = format_ident!("My{}{}", ident, "IsCool");
/// assert_eq!(my_ident, "MyIdentIsCool");
/// ```
///
/// Alternatively, the span can be overridden by passing the `span` named
/// argument.
///
/// ```edition2018
/// # use quote::format_ident;
/// # const IGNORE_TOKENS: &'static str = stringify! {
/// let my_span = /* ... */;
/// # };
/// # let my_span = proc_macro2::Span::call_site();
/// format_ident!("MyIdent", span = my_span);
/// ```
///
/// [`Span`]: `proc_macro2::Span`
/// [`Span::call_site`]: `proc_macro2::Span::call_site`
///
/// # Panics
///
/// This method will panic if the formatted string is not a valid identifier, or
/// a formatting trait implementation returned an error.
///
/// # Examples
///
/// Composing raw and non-raw identifiers:
/// ```edition2018
/// # use quote::format_ident;
/// let my_ident = format_ident!("My{}", "Ident");
/// assert_eq!(my_ident, "MyIdent");
///
/// let raw = format_ident!("r#Raw");
/// assert_eq!(raw, "r#Raw");
///
/// let my_ident_raw = format_ident!("{}Is{}", my_ident, raw);
/// assert_eq!(my_ident_raw, "MyIdentIsRaw");
/// ```
///
/// Integer formatting options:
/// ```edition2018
/// # use quote::format_ident;
/// let num: u32 = 10;
///
/// let decimal = format_ident!("Id_{}", num);
/// assert_eq!(decimal, "Id_10");
///
/// let octal = format_ident!("Id_{:o}", num);
/// assert_eq!(octal, "Id_12");
///
/// let binary = format_ident!("Id_{:b}", num);
/// assert_eq!(binary, "Id_1010");
///
/// let lower_hex = format_ident!("Id_{:x}", num);
/// assert_eq!(lower_hex, "Id_a");
///
/// let upper_hex = format_ident!("Id_{:X}", num);
/// assert_eq!(upper_hex, "Id_A");
/// ```
#[macro_export]
macro_rules! format_ident {
    // Final State
    (@@ [$span:expr, $($o:tt)*] $(,)*) => {
        $crate::__rt::mk_ident(&format!($($o)*), $span)
    };

    // Span Argument
    (@@ [$_sp:expr, $($o:tt)*] span = $span:expr, $($t:tt)*) => {
        format_ident!(@@ [
            ::std::option::Option::Some::<$crate::__rt::Span>($span),
            $($o)*
        ] $($t)*)
    };

    // Named Arguments
    (@@ [$span:expr, $($o:tt)*] $i:ident = $e:expr, $($t:tt)*) => {
        match $crate::__rt::IdentFragmentAdapter(&$e) {
            arg => format_ident!(@@ [$span.or(arg.span()), $($o)*, $i = arg] $($t)*),
        }
    };

    // Positional Arguments
    (@@ [$span:expr, $($o:tt)*] $e:expr, $($t:tt)*) => {
        match $crate::__rt::IdentFragmentAdapter(&$e) {
            arg => format_ident!(@@ [$span.or(arg.span()), $($o)*, arg] $($t)*),
        }
    };

    // Argument Options
    ($f:expr) => {
        format_ident!(@@ [
            ::std::option::Option::None,
            $f
        ])
    };
    ($f:expr, $($t:tt)*) => {
        format_ident!(@@ [
            ::std::option::Option::None,
            $f
        ] $($t)*,)
    };
}
