/// The byte order trait.
pub use byteorder::ByteOrder;
/// Little Endian.
pub type LE = byteorder::LittleEndian;
/// Big Endian.
pub type BE = byteorder::BigEndian;
/// Native Endian.
pub type NE = byteorder::NativeEndian;
/// Opposing Endian.
#[cfg(target_endian = "little")]
pub type OP = byteorder::BigEndian;
#[cfg(target_endian = "big")]
pub type OP = byteorder::LittleEndian;

// These are runtime endian detection items since byteorder
// has had an outstanding ticket to add such things for several
// years and as such seems they will not be added.

// It is needed here because we 'try' to work with local endian
// but have to know how to flip things at runtime when dealing
// with non-native endian.

/// Runtime endianess values.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Endian {
    /// Little endian.
    Little,
    /// Big endian.
    Big,
}

// Runtime native endian.

/// The runtime native endian.
#[cfg(target_endian = "little")]
pub const NATIVE_ENDIAN: Endian = Endian::Little;
/// The runtime native endian.
#[cfg(target_endian = "big")]
pub const NATIVE_ENDIAN: Endian = Endian::Big;

// Runtime opposiing endian.

/// The runtime opposing endian.
#[cfg(target_endian = "little")]
pub const OPPOSING_ENDIAN: Endian = Endian::Big;
/// The runtime opposing endian.
#[cfg(target_endian = "big")]
pub const OPPOSING_ENDIAN: Endian = Endian::Little;
