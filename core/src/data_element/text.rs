//! This module contains reusable components for encoding and decoding text in DICOM
//! data structures, including support for character repertoires.
//! 
//! The Character Repertoires supported by DICOM are:
//! - ISO 8859
//! - JIS X 0201-1976 Code for Information Interchange
//! - JIS X 0208-1990 Code for the Japanese Graphic Character set for information interchange
//! - JIS X 0212-1990 Code of the supplementary Japanese Graphic Character set for information interchange
//! - KS X 1001 (registered as ISO-IR 149) for Korean Language
//! - TIS 620-2533 (1990) Thai Characters Code for Information Interchange
//! - ISO 10646-1, 10646-2, and their associated supplements and extensions for Unicode character set
//! - GB 18030
//! - GB2312
//!
//! At the moment, this library supports only the first repertoire.

use error::{Result, TextEncodingError};
use std::io::Write;
use std::fmt::Debug;
/* use encoding; */

/// An enum type for the the supported character sets.
#[derive(Debug, Clone, Copy)]
pub enum SpecificCharacterSet {
    /// The default character set.
    Default

    // TODO needs more
}

impl SpecificCharacterSet {

    /// Retrieve the codec.
    pub fn get_codec(&self) -> Option<Box<TextCodec>> {
        match *self {
            SpecificCharacterSet::Default => Some(Box::new(DefaultCharacterSetCodec))
        }
    }
}

impl Default for SpecificCharacterSet {
    fn default() -> SpecificCharacterSet {
        SpecificCharacterSet::Default
    }
}

/// A holder of encoding and decoding mechanisms for text in DICOM content,
/// which according to the standard, depends on the specific character set.
pub trait TextCodec: Debug {
    /// Decode the given byte buffer as a single string. The resulting string
    /// _will_ contain backslash character ('\') to delimit individual values,
    /// and should be split later on if required.
    fn decode(&self, text: &[u8]) -> Result<String>;

    /// Encode a text value into the given destination. The input string can
    /// feature multiple text values by using the backslash character ('\')
    /// as the value delimiter.
    fn encode<W: Write>(&self, text: &str, out: &mut W) -> Result<()> where Self: Sized;
}

/// Data type representing the default character set.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct DefaultCharacterSetCodec;

impl TextCodec for DefaultCharacterSetCodec {
    fn decode(&self, text: &[u8]) -> Result<String> {
        // TODO this is NOT DICOM compliant,
        // although it will decode 7-bit ASCII text just fine
        let r = try!(String::from_utf8(Vec::from(text.as_ref()))
                            .map_err(TextEncodingError::from));
        Ok(r)
    }

    fn encode<W: Write>(&self, text: &str, out: &mut W) -> Result<()> {
        // TODO this is NOT DICOM compliant,
        // although it will encode 7-bit ASCII text just fine
        try!(out.write(text.as_bytes()));
        Ok(())
    }
    
}