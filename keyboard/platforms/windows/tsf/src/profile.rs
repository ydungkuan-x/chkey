//! TSF Profile definitions and constants for CHKey.

use windows::core::GUID;

/// GUID for CHKey TSF Text Service COM Class ID.
pub const CHKEY_TSF_CLSID_GUID: GUID =
    GUID::from_u128(0xa3f2b801_1234_4567_89ab_cdef01234567);

/// GUID for CHKey TSF Profile ID.
pub const CHKEY_TSF_PROFILE_GUID: GUID =
    GUID::from_u128(0xa3f2b802_1234_4567_89ab_cdef01234567);


/// Language identifier for English US (0x0409).
pub const LANGID_ENGLISH_US: u16 = 0x0409;

/// Language identifier for Vietnamese (0x042A).
pub const LANGID_VIETNAMESE: u16 = 0x042A;
