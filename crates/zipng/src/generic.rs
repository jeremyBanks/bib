use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitCode;
use std::process::Termination;

use static_assertions::assert_impl_all;

pub fn default<T>() -> T
where T: Default {
    T::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
/// An uninhabited [`!`]-like "never" type, with trait implementations as needed
/// for convenience within this crate's types.
pub enum never {}

assert_impl_all!(never: Send, Sync);
assert_impl_all!(panic: Send, Sync);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
/// An uninhabited [`!`]-like "never" type that provides a panicking
/// implementation of `From` for any `Display + Debug` error type,
/// with trait implementations as needed for convenience within this crate's.
pub enum panic {}

impl<Err> From<Err> for panic
where Err: Display + Debug
{
    #[track_caller]
    fn from(error: Err) -> Self {
        panic!("{error}")
    }
}

impl Default for never {
    fn default() -> Self {
        unreachable!()
    }
}

impl From<panic> for never {
    fn from(_: panic) -> never {
        unreachable!()
    }
}

impl From<never> for panic {
    fn from(_: never) -> panic {
        unreachable!()
    }
}

impl Termination for panic {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}

impl Termination for never {
    fn report(self) -> ExitCode {
        unreachable!()
    }
}
