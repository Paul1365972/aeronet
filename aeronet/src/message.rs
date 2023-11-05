use std::error::Error;

/// Data that can be sent to and received by a transport.
///
/// This is a marker trait that ensures that data sent between transports is:
/// * [`Send`]
/// * [`Sync`]
/// * `'static`
///
/// The user defines which types of messages their transport uses, and this
/// trait acts as a minimum bound for all message types. However, for different
/// transport implementations, there may be additional bounds placed on message
/// types, such as for transports using a network, in which data is sent as a
/// byte sequence:
/// * [`TryIntoBytes`] if the message should be able to be converted into a byte
///   sequence
/// * [`TryFromBytes`] if the message should be able to be constructed from a
///   byte sequence
///
/// This trait is automatically implemented for all types matching these
/// criteria.
pub trait Message: Send + Sync + 'static {}

impl<T> Message for T where T: Send + Sync + 'static {}

/// Data that can potentially be converted into a sequence of bytes.
///
/// The transport implementation may wish to handle messages as a byte sequence,
/// for example if it communicates over a network.
/// This trait can be used as a bound to ensure that messages can be converted
/// into a byte form.
///
/// See [`TryFromBytes`] for the receiving counterpart.
#[cfg_attr(
    feature = "bincode",
    doc = r##"

# [`bincode`] + [`serde`] support

With the `bincode` feature enabled, this trait will automatically be implemented for types which
implement [`serde::Serialize`].
"##
)]
pub trait TryIntoBytes {
    /// Error type for [`TryIntoBytes::try_into_bytes`].
    type Error: Error + Send + Sync + 'static;

    /// Performs the conversion.
    /// 
    /// # Errors
    /// 
    /// Errors if the conversion could not be performed.
    fn try_into_bytes(self) -> Result<Vec<u8>, Self::Error>;
}

/// Data that can potentially be converted from a sequence of bytes into this
/// type.
///
/// The transport implementation may wish to handle messages as a byte sequence,
/// for example if it communicates over a network.
/// This trait can be used as a bound to ensure that messages can be converted
/// from a byte form.
///
/// See [`TryIntoBytes`] for the sending counterpart.
#[cfg_attr(
    feature = "bincode",
    doc = r##"

# [`bincode`] + [`serde`] support

With the `bincode` feature enabled, this trait will automatically be implemented for types which
implement [`serde::de::DeserializeOwned`].
"##
)]
pub trait TryFromBytes: Sized {
    /// Error type for [`TryFromBytes::try_from_bytes`].
    type Error: Error + Send + Sync + 'static;

    /// Performs the conversion.
    /// 
    /// # Errors
    /// 
    /// Errors if the conversion could not be performed.
    fn try_from_bytes(buf: &[u8]) -> Result<Self, Self::Error>;
}

#[cfg(feature = "bincode")]
impl<T> TryIntoBytes for T
where
    T: serde::Serialize,
{
    type Error = bincode::Error;

    fn try_into_bytes(self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(&self)
    }
}

#[cfg(feature = "bincode")]
impl<T> TryFromBytes for T
where
    T: serde::de::DeserializeOwned,
{
    type Error = bincode::Error;

    fn try_from_bytes(buf: &[u8]) -> Result<Self, Self::Error> {
        bincode::deserialize(buf)
    }
}
