
#[doc(hidden)]
#[macro_export]
macro_rules! cartesian__ {
    (@product::next([$($token:tt)+] $($rest:tt)*) -> $cb:tt)
    => { $crate::cartesian__!{ @product::unpack([$($token)+] $($rest)*) -> $cb } };

    // base case; direct product of no arguments
    (@product::next() -> ($mac:ident!($($args:tt)*)))
    => {$mac!{$($args)*}};

    // Each direct product in the invocation incurs a fixed number of recursions
    //  as we replicate the macro.  First, we must smash anything we want to replicate
    //  into a single tt that can be matched without repetitions.  Do this to `rest`.
    (@product::unpack([$($token:tt)*] $($rest:tt)*) -> $cb:tt)
    => {$crate::cartesian__!{ @product::unpack_2([$($token)*] [$($rest)*]) -> $cb }};

    // Replicate macro for each token.
    (@product::unpack_2([$($token:tt)*] $rest:tt) -> $cb:tt)
    => { $( $crate::cartesian__!{ @product::unpack_3($token $rest) -> $cb } )* };

    // Expand the unparsed arguments back to normal;
    // add the token into the macro call
    (@product::unpack_3($token:tt [$($rest:tt)*]) -> ($mac:ident!($($args:tt)*)))
    => {$crate::cartesian__!{ @product::next($($rest)*) -> ($mac!($($args)*$token)) }};
}
/// Higher-order macro that iterates over a cartesian product.
///
/// Useful for generating impls involving opaque traits of the sort
/// sometimes seen used as bounds in public APIs.
///
/// It takes a number of groups of token trees and a suitable definition
/// for a callback macro, and it calls the macro with one token tree from
/// each group in order.
#[macro_export]
macro_rules! cartesian {
    (
        $([$($groups:tt),*]),*
        ($($mac_match:tt)*)
        => {$($mac_body:tt)*}$(;)*
    )
    => {
        // (this fixed name gets overwritten on each use.  Technically, using a fixed name
        //  makes this macro not "re-entrant", in the sense that you can't call it in a nested
        //  fashion... but there is no reason to do so, because the macro has a monolithic design)
        macro_rules! __cartesian__user_macro {
            ($($mac_match)*) => {$($mac_body)*};
        }
        $crate::cartesian__!{ @product::next($([$($groups)*])*) -> (__cartesian__user_macro!()) }
    };
}
