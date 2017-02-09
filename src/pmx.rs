use nom::{IResult, le_f32, le_u8, le_u32, rest};
use nom::IResult::*;

#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    version: f32,
    globals: &'a [u8], // usually &[u8; 8]
    model_name: String,
    model_name_en: String,
    comments: String,
    comments_en: String,
}

named!(parse_header<&[u8], Header>, do_parse!(
    tag!("PMX ") >>
    version: le_f32 >>
    globals: length_bytes!(le_u8) >>
    text: count!(length_value!(le_u32, rest), 4) >>
            rest >>

    (Header {
        version: version,
        globals: globals,
        model_name: String::from_utf16(Vec::from(text[0])).unwrap(),
        model_name_en: String::from_utf16(Vec::from(text[1])).unwrap(),
        comments: String::from_utf16(Vec::from(text[2])).unwrap(),
        comments_en: String::from_utf16(Vec::from(text[3])).unwrap(),
    })
));

#[test]
fn mytest() {
    /*
    let head_pattern = [0x50u8, 0x4Du8, 0x58u8, 0x20u8, // b"PMD "
                        0x00u8, 0x00u8, 0x00u8, 0x40u8, // 2.0f32 8u8
                        0x08u8, 0x00u8, 0x00u8, 0x02u8,
                        0x01u8, 0x01u8, 0x02u8, 0x02u8,
                        0x02u8];
    let r = parse_header(&head_pattern);
    */
    let head_pattern = include_bytes!("../asset/江風ver1.05.pmx");
    let r = parse_header(head_pattern);
    assert_eq!(r,
               Done(&b""[..],
                    Header {
                        version: 2.0,
                        globals: &[0x00u8, 0x00u8, 0x02u8, 0x01u8, 0x01u8, 0x02u8, 0x02u8, 0x02u8],
                        model_name: String::from(""),
                        model_name_en: String::from(""),
                        comments: String::from(""),
                        comments_en: String::from(""),
                    }));
}