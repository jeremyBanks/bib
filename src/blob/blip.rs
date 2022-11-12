use std::convert::Infallible;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::rc::Rc;

use bstr::BStr;
use serde::Deserialize;
use serde::Serialize;
use static_assertions::assert_impl_all;
use thiserror::Error;

use crate::blob::Blob;
use crate::generic::default;
use crate::generic::Type;
use crate::inline::InlineVec;
use crate::never;

/// A [`Blip`] represents a [`Blob`], stored inline if it's under 32 bytes,
/// otherwise represented by its 32-byte BLAKE3 hash digest.
/// This type is `Copy`.
pub struct Blip<T>
where T: ?Sized
{
    bytes: InlineVec<u8, 32>,
    t: Type<T>,
}

assert_impl_all!(Blip<never>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<Infallible>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<Rc<u8>>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<[u8]>: Sized, Copy, Serialize, Sync, Send);
assert_impl_all!(Blip<dyn Debug>: Sized, Copy, Serialize, Sync, Send);

impl<T> Blip<T>
where T: ?Sized
{
    /// Returns the Blip representing a given Blob.
    pub fn for_blob(blob: &Blob<T>) -> Self {
        Blip::for_bytes(blob.as_ref()).retype()
    }

    /// Creates a blip from the corresponding raw bytes (either a hash or an
    /// inline value, depending on length).
    pub fn try_from_raw_bytes(blip_bytes: &[u8]) -> Result<Self, TooLongForBlipError> {
        Ok(Self {
            bytes: InlineVec::try_from_slice(&blip_bytes)
                .map_err(|_| TooLongForBlipError(blip_bytes.len()))?,
            t: default(),
        })
    }

    fn retype<R: ?Sized>(self) -> Blip<R> {
        Blip {
            bytes: self.bytes,
            t: default(),
        }
    }

    /// Whether this Blip contains an inline value.
    pub const fn is_inline(&self) -> bool {
        self.bytes.len() < 32
    }

    /// Whether this Blip contains a hash identifying external-stored content.
    pub const fn is_hash(&self) -> bool {
        self.bytes.len() == 32
    }

    /// Returns the inline value as a Blob, if present.
    pub fn inline_blob(&self) -> Option<Blob<T>> {
        if self.is_inline() {
            Some(Blob::for_bytes(&self.bytes))
        } else {
            None
        }
    }
}

impl Blip<[u8]> {
    pub fn for_bytes(mut bytes: &[u8]) -> Self {
        if bytes.len() >= 32 {
            bytes = blake3::hash(bytes).as_bytes()
        }
        Self {
            bytes: InlineVec::try_from_slice(bytes).unwrap(),
            t: default(),
        }
    }

    pub fn for_value(bytes: &[u8]) -> Self {
        Blip::for_bytes(bytes).retype()
    }

    pub fn inline_value(&self) -> Option<&[u8]> {
        if self.is_inline() {
            Some(self.bytes.as_ref())
        } else {
            None
        }
    }
}

impl Blip<str> {
    pub fn for_str(bytes: &str) -> Self {
        Blip::for_bytes(bytes.as_bytes()).retype()
    }

    pub fn for_value(bytes: &str) -> Self {
        Blip::for_bytes(bytes.as_bytes()).retype()
    }

    pub fn inline_value(&self) -> Option<&str> {
        if self.is_inline() {
            Some(std::str::from_utf8(self.bytes.as_ref()).unwrap())
        } else {
            None
        }
    }
}

impl<T> Blip<T>
where T: Serialize + Deserialize<'static>
{
    pub fn for_value(value: &T) -> Self {
        Blob::for_value(value).blip()
    }

    pub fn inline_value(&self) -> Option<T> {
        self.inline_blob().map(|b| b.as_ref().value().unwrap())
    }
}

impl<T> Copy for Blip<T> where T: ?Sized {}

impl<T> Blob<T>
where T: ?Sized
{
    pub fn blip(&self) -> Blip<T> {
        self.into()
    }
}

#[derive(Debug, Error, Copy, Clone)]
#[error("Blips must be between 0 and 32 bytes, but the input was {0} bytes.")]
pub struct TooLongForBlipError(usize);

impl<T> From<Blob<T>> for Blip<T>
where T: ?Sized
{
    fn from(value: Blob<T>) -> Self {
        Blip::for_bytes(value.as_ref()).retype()
    }
}

impl<T> From<&Blob<T>> for Blip<T>
where T: ?Sized
{
    fn from(value: &Blob<T>) -> Self {
        Blip::for_bytes(value.as_ref()).retype()
    }
}

#[derive(Debug, Error, Copy, Clone)]
#[error("This Blip represents a value that's too long to store inline ({0} bytes > 31 bytes).")]
pub struct TooLongForInlineBlipError(usize);

impl<T> TryFrom<Blip<T>> for Blob<T>
where T: ?Sized
{
    type Error = TooLongForInlineBlipError;

    fn try_from(value: Blip<T>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<T> Hash for Blip<T>
where T: ?Sized
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<T> PartialEq for Blip<T>
where T: ?Sized
{
    fn eq(&self, other: &Self) -> bool {
        self.bytes.eq(&other.bytes)
    }
}

impl<T> Eq for Blip<T> where T: ?Sized {}

impl<T> PartialOrd for Blip<T>
where T: ?Sized
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl<T> Ord for Blip<T>
where T: ?Sized
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<T> Debug for Blip<T>
where T: ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T> Display for Blip<T>
where T: ?Sized
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(BStr::new(&self.bytes), f)
    }
}

impl<T> Serialize for Blip<T>
where T: ?Sized
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serde_bytes::Bytes::new(&self.bytes.as_ref()).serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Blip<T>
where T: ?Sized
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?.into_vec();
        Self::try_from_raw_bytes(&bytes).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl<T> Default for Blip<T>
where T: ?Sized
{
    fn default() -> Self {
        Self {
            bytes: default(),
            t: default(),
        }
    }
}

impl<T> Clone for Blip<T>
where T: ?Sized
{
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            t: default(),
        }
    }
}

impl<T> AsRef<[u8]> for Blip<T>
where T: ?Sized
{
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl<T> PartialEq<Blob<T>> for Blip<T>
where T: ?Sized
{
    fn eq(&self, other: &Blob<T>) -> bool {
        self == &other.blip()
    }
}

impl<T> PartialEq<Blip<T>> for Blob<T>
where T: ?Sized
{
    fn eq(&self, other: &Blip<T>) -> bool {
        &self.blip() == other
    }
}
