//! Module containing data structures and readers of DICOM file meta information tables.
use std::io::{Read, Seek, SeekFrom};
use error::{Result, Error};
use byteorder::{ByteOrder, LittleEndian};
use attribute::tag::Tag;
use data_element::Header;
use data_element::decode;
use data_element::text;
use data_element::text::TextCodec;
use data_element::decode::Decode;

#[cfg(test)]
mod tests {
    use super::DicomMetaTable;
    use std::io::Cursor;

    const TEST_PREAMBLE: &'static [u8] = &[
        // preamble
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        // magic code
        0x44, 0x49, 0x43, 0x4D,
        // File Meta Information Group Length: (0000,0002) ; UL ; 4 ; 200
        0x02, 0x00, 0x00, 0x00, 0x55, 0x4c, 0x04, 0x00, 0xc8, 0x00, 0x00, 0x00,
        // File Meta Information Version: (0002, 0001) ; OB ; 2 ; [0x00, 0x01]
        0x02, 0x00, 0x01, 0x00, 0x4f, 0x42, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01,
        // Media Storage SOP Class UID (0002, 0002) ; UI ; 26 ; "1.2.840.10008.5.1.4.1.1.1\0" (ComputedRadiographyImageStorage)
        0x02, 0x00, 0x02, 0x00, 0x55, 0x49, 0x1a, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34,
        0x30, 0x2e, 0x31, 0x30, 0x30, 0x30, 0x38, 0x2e, 0x35, 0x2e, 0x31, 0x2e, 0x34, 0x2e,
        0x31, 0x2e, 0x31, 0x2e, 0x31, 0x00,
        // Media Storage SOP Instance UID (0002, 0003) ; UI ; 56 ; "1.2.3.4.5.12345678.1234567890.1234567.123456789.1234567\0"
        0x02, 0x00, 0x03, 0x00, 0x55, 0x49, 0x38, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x33, 0x2e,
        0x34, 0x2e, 0x35, 0x2e, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x2e, 0x31,
        0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x2e, 0x31, 0x32, 0x33, 0x34,
        0x35, 0x36, 0x37, 0x2e, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x2e,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x00,
        // Transfer Syntax UID (0002, 0010) ; UI ; 20 ; "1.2.840.10008.1.2.1\0" (LittleEndianExplicit)
        0x02, 0x00, 0x10, 0x00, 0x55, 0x49, 0x14, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x38, 0x34,
        0x30, 0x2e, 0x31, 0x30, 0x30, 0x30, 0x38, 0x2e, 0x31, 0x2e, 0x32, 0x2e, 0x31, 0x00,
        // Implementation Class UID (0002, 0012) ; UI ; 20 ; "1.2.345.6.7890.1.234"
        0x02, 0x00, 0x12, 0x00, 0x55, 0x49, 0x14, 0x00, 0x31, 0x2e, 0x32, 0x2e, 0x33, 0x34,
        0x35, 0x2e, 0x36, 0x2e, 0x37, 0x38, 0x39, 0x30, 0x2e, 0x31, 0x2e, 0x32, 0x33, 0x34,

        // optional elements:
        
        // Implementation Version Name (0002,0013) ; SH ; "RUSTY_DICOM_269"
        0x02, 0x00, 0x13, 0x00, 0x53, 0x48, 0x10, 0x00, 0x52, 0x55, 0x53, 0x54, 0x59, 0x5f,
        0x44, 0x49, 0x43, 0x4f, 0x4d, 0x5f, 0x32, 0x36, 0x39, 0x20,
        // Source Application Entity Title (0002, 0016) ; AE ; 0 (no data)
        0x02, 0x00, 0x16, 0x00, 0x41, 0x45, 0x00, 0x00

    ];

    #[test]
    fn meta_table_ok() {
        let mut source = Cursor::new(TEST_PREAMBLE);

        let table = DicomMetaTable::read_meta_table(&mut source)
                .expect("Should not yield any errors dangit!");

        assert_eq!(table.information_group_length, 200);
        assert_eq!(table.information_version, [0u8, 1u8]);
        assert_eq!(table.media_storage_sop_class_uid, "1.2.840.10008.5.1.4.1.1.1\0");
        assert_eq!(table.media_storage_sop_instance_uid, "1.2.3.4.5.12345678.1234567890.1234567.123456789.1234567\0");
        assert_eq!(table.transfer_syntax, "1.2.840.10008.1.2.1\0");
        assert_eq!(table.implementation_class_uid, "1.2.345.6.7890.1.234");
        assert_eq!(table.implementation_version_name, Some(String::from("RUSTY_DICOM_269 ")));
        assert_eq!(table.source_application_entity_title, None);
        assert_eq!(table.sending_application_entity_title, None);
        assert_eq!(table.receiving_application_entity_title, None);
        assert_eq!(table.private_information_creator_uid, None);
        assert_eq!(table.private_information, None);
    }

}

const DICM_MAGIC_CODE:[u8; 4] = [0x44, 0x49, 0x43, 0x4D];

/// DICOM Meta Information Table.
///
/// This data type contains the relevant parts of the file meta information table, as
/// specified in [1].
///
/// [1]: http://dicom.nema.org/medical/dicom/current/output/chtml/part06/chapter_7.html
#[derive(Debug, PartialEq)]
pub struct DicomMetaTable {
    /// File Meta Information Group Length
    information_group_length: u32,         
    /// File Meta Information Version
    information_version: [u8; 2],
    /// Media Storage SOP Class UID
    media_storage_sop_class_uid: String,
    /// Media Storage SOP Instance UID
    media_storage_sop_instance_uid: String,
    /// Transfer Syntax UID
    transfer_syntax: String,
    /// Implementation Class UID
    implementation_class_uid: String,

    /// Implementation Version Name
    implementation_version_name: Option<String>,
    /// Source Application Entity Title
    source_application_entity_title: Option<String>,
    /// Sending Application Entity Title
    sending_application_entity_title: Option<String>,
    /// Receiving Application Entity Title
    receiving_application_entity_title: Option<String>,
    /// Private Information Creator UID
    private_information_creator_uid: Option<String>,
    /// Private Information
    private_information: Option<Vec<u8>>,
}

/// Utility function for reading the whole DICOM element as a string with the given tag.
fn read_str_as_tag<S: Read + Seek, D: Decode<Source=S>, T: TextCodec>(
            source: &mut S, decoder: &D, text: &T, group_length_remaining: &mut u32, tag: Tag) -> Result<String> {
    let elem = try!(decoder.decode_header(source));
    if elem.tag() != tag {
        return Err(Error::UnexpectedElement);
    }
    read_str_body(source, text, group_length_remaining, elem.len())
}

/// Utility function for reading the body of the DICOM element as a UID.
fn read_str_body<S: Read + Seek, T: TextCodec>(
            source: &mut S, text: &T, group_length_remaining: &mut u32, len: u32) -> Result<String> {
    let mut v = Vec::<u8>::with_capacity(len as usize);
    v.resize(len as usize, 0);
    try!(source.read_exact(&mut v));
    *group_length_remaining -= 8 + len;
    text.decode(&v)
}

impl DicomMetaTable {

    /// read the full meta information from the given file source
    pub fn read_meta_table<F: Read + Seek>(file: &mut F) -> Result<DicomMetaTable> {
        // Check the preamble and magic code (128 bytes of preamble followed by 'DICM')

        let mut buff: [u8; 4] = [0; 4];

        // skip the preamble
        try!(file.seek(SeekFrom::Current(128)));
        {
            // check magic code
            try!(file.read_exact(&mut buff));

            if buff != DICM_MAGIC_CODE {
                return Err(Error::InvalidFormat);
            }
        }

        let decoder = decode::get_file_header_decoder();
        let text = text::DefaultCharacterSetCodec;

        let builder = DicomMetaTableBuilder::new();

        let group_length: u32 = {
            let elem = try!(decoder.decode_header(file));
            if elem.tag() != (0x0002, 0x0000) {
                return Err(Error::UnexpectedElement);
            }
            if elem.len() != 4 {
                return Err(Error::UnexpectedDataValueLength);
            }
            let mut buff: [u8; 4] = [0; 4];
            try!(file.read_exact(&mut buff));
            LittleEndian::read_u32(&buff)
        };

        let mut group_length_remaining = group_length;

        let mut builder = builder
            .group_length(group_length)
            .information_version({
                let elem = try!(decoder.decode_header(file));
                if elem.tag() != (0x0002, 0x0001) {
                    return Err(Error::UnexpectedElement);
                }
                if elem.len() != 2 {
                    return Err(Error::UnexpectedDataValueLength);
                }
                let mut hbuf = [0u8; 2];
                try!(file.read_exact(&mut hbuf[..]));
                group_length_remaining -= 8 + elem.len();
                hbuf
            })
            .media_storage_sop_class_uid(try!(read_str_as_tag(file, &decoder, &text, &mut group_length_remaining, Tag(0x0002, 0x0002))))
            .media_storage_sop_instance_uid(try!(read_str_as_tag(file, &decoder, &text, &mut group_length_remaining, Tag(0x0002, 0x0003))))
            .transfer_syntax(try!(read_str_as_tag(file, &decoder, &text, &mut group_length_remaining, Tag(0x0002, 0x0010))))
            .implementation_class_uid(try!(read_str_as_tag(file, &decoder, &text, &mut group_length_remaining, Tag(0x0002, 0x0012))));

        // Fetch optional data elements
        while group_length_remaining > 0 {
            let elem = try!(decoder.decode_header(file));
            group_length_remaining -= elem.len();
            builder = match elem.tag() {
                Tag(0x0002,0x0013) => { // Implementation Version Name
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    group_length_remaining -= 4 + elem.len();
                    builder.implementation_version_name(try!(text.decode(&v)))
                }
                Tag(0x0002,0x0016) => { // Source Application Entity Title
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    group_length_remaining -= 4 + elem.len();
                    builder.source_application_entity_title(try!(text.decode(&v)))
                },
                Tag(0x0002,0x0017) => { // Sending Application Entity Title
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    group_length_remaining -= 4 + elem.len();
                    builder.sending_application_entity_title(try!(text.decode(&v)))
                },
                Tag(0x0002,0x0018) => { // Receiving Application Entity Title
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    group_length_remaining -= 4 + elem.len();
                    builder.receiving_application_entity_title(try!(text.decode(&v)))
                },
                Tag(0x0002,0x0100) => { // Private Information Creator UID
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    group_length_remaining -= 4 + elem.len();
                    builder.private_information_creator_uid(try!(text.decode(&v)))
                },
                Tag(0x0002,0x0102) => { // Private Information
                    let mut v = Vec::<u8>::with_capacity(elem.len() as usize);
                    v.resize(elem.len() as usize, 0);
                    try!(file.read_exact(&mut v));
                    builder.private_information(v)
                },
                Tag(0x0002, _) => { // unknown tag, do nothing
                    builder
                },
                _ => { // unexpected tag! do nothing for now, although this could represent a serious bug
                    builder
                }
            }
        }

        builder.build()
    }
   
}

/// A builder for DICOM meta information tables.
#[derive(Debug, Clone)]
pub struct DicomMetaTableBuilder {
    /// File Meta Information Group Length (UL)
    information_group_length: Option<u32>,         
    /// File Meta Information Version (OB)
    information_version: Option<[u8; 2]>,
    /// Media Storage SOP Class UID (UI)
    media_storage_sop_class_uid: Option<String>,
    /// Media Storage SOP Instance UID (UI)
    media_storage_sop_instance_uid: Option<String>,
    /// Transfer Syntax UID (UI)
    transfer_syntax: Option<String>,
    /// Implementation Class UID (UI)
    implementation_class_uid: Option<String>,

    /// Implementation Version Name (SH)
    implementation_version_name: Option<String>,
    /// Source Application Entity Title (AE)
    source_application_entity_title: Option<String>,
    /// Sending Application Entity Title (AE)
    sending_application_entity_title: Option<String>,
    /// Receiving Application Entity Title (AE)
    receiving_application_entity_title: Option<String>,
    /// Private Information Creator UID (UI)
    private_information_creator_uid: Option<String>,
    /// Private Information (OB)
    private_information: Option<Vec<u8>>,
    
}

impl Default for DicomMetaTableBuilder {
    fn default() -> DicomMetaTableBuilder {
        DicomMetaTableBuilder {
            information_group_length: None,         
            information_version: None,
            media_storage_sop_class_uid: None,
            media_storage_sop_instance_uid: None,
            transfer_syntax: None,
            implementation_class_uid: None,
            implementation_version_name: None,
            source_application_entity_title: None,
            sending_application_entity_title: None,
            receiving_application_entity_title: None,
            private_information_creator_uid: None,
            private_information: None,
        }
    }
}

impl DicomMetaTableBuilder {
    /// Create a new, empty builder.
    pub fn new() ->  DicomMetaTableBuilder {
        DicomMetaTableBuilder::default()
    }

    /// Define the meta information group length.
    pub fn group_length(mut self, value: u32) -> DicomMetaTableBuilder {
        self.information_group_length = Some(value);
        self
    }

    /// Define the meta information version.
    pub fn information_version(mut self, value: [u8; 2]) -> DicomMetaTableBuilder {
        self.information_version = Some(value);
        self
    }

    /// Define the media storage SOP class UID.
    pub fn media_storage_sop_class_uid(mut self, value: String) -> DicomMetaTableBuilder {
        self.media_storage_sop_class_uid = Some(value);
        self
    }

    /// Define the media storage SOP instance UID.
    pub fn media_storage_sop_instance_uid(mut self, value: String) -> DicomMetaTableBuilder {
        self.media_storage_sop_instance_uid = Some(value);
        self
    }

    /// Define the transfer syntax.
    pub fn transfer_syntax(mut self, value: String) -> DicomMetaTableBuilder {
        self.transfer_syntax = Some(value);
        self
    }

    /// Define the implementation class UID.
    pub fn implementation_class_uid(mut self, value: String) -> DicomMetaTableBuilder {
        self.implementation_class_uid = Some(value);
        self
    }

    /// Define the implementation version name.
    pub fn implementation_version_name(mut self, value: String) -> DicomMetaTableBuilder {
        self.implementation_version_name = Some(value);
        self
    }

    /// Define the source application entity title.
    pub fn source_application_entity_title(mut self, value: String) -> DicomMetaTableBuilder {
        self.source_application_entity_title = Some(value);
        self
    }

    /// Define the sending application entity title.
    pub fn sending_application_entity_title(mut self, value: String) -> DicomMetaTableBuilder {
        self.sending_application_entity_title = Some(value);
        self
    }

    /// Define the receiving application entity title.
    pub fn receiving_application_entity_title(mut self, value: String) -> DicomMetaTableBuilder {
        self.receiving_application_entity_title = Some(value);
        self
    }

    /// Define the private information creator UID.
    pub fn private_information_creator_uid(mut self, value: String) -> DicomMetaTableBuilder {
        self.private_information_creator_uid = Some(value);
        self
    }

    /// Define the private information as a vector of bytes.
    pub fn private_information(mut self, value: Vec<u8>) -> DicomMetaTableBuilder {
        self.private_information = Some(value);
        self
    }
    
    /// Build the table.
    pub fn build(self) -> Result<DicomMetaTable> {
        let group_length = try!(self.information_group_length.ok_or_else(|| Error::InvalidFormat));
        let information_version = try!(self.information_version.ok_or_else(|| Error::InvalidFormat));
        let media_storage_sop_class_uid = try!(self.media_storage_sop_class_uid.ok_or_else(|| Error::InvalidFormat));
        let media_storage_sop_instance_uid = try!(self.media_storage_sop_instance_uid.ok_or_else(|| Error::InvalidFormat));
        let transfer_syntax = try!(self.transfer_syntax.ok_or_else(|| Error::InvalidFormat));
        let implementation_class_uid = try!(self.implementation_class_uid.ok_or_else(|| Error::InvalidFormat));
        Ok(DicomMetaTable {
            information_group_length: group_length,         
            information_version: information_version,
            media_storage_sop_class_uid: media_storage_sop_class_uid,
            media_storage_sop_instance_uid: media_storage_sop_instance_uid,
            transfer_syntax: transfer_syntax,
            implementation_class_uid: implementation_class_uid,
            implementation_version_name: self.implementation_version_name,
            source_application_entity_title: self.source_application_entity_title,
            sending_application_entity_title: self.sending_application_entity_title,
            receiving_application_entity_title: self.receiving_application_entity_title,
            private_information_creator_uid: self.private_information_creator_uid,
            private_information: self.private_information,
        })
    }
}