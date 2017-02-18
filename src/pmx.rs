use nom::{le_f32, le_u8, le_i32, rest};
use nom::IResult::*;
use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Pmx<'a> {
    header: Header<'a>,
    //vertices: Vec<Vertex>,
}

#[derive(Debug, PartialEq)]
struct Header<'a> {
    version: f32,
    globals: &'a [u8], // usually &[u8; 8]
    model_name: String,
    model_name_en: String,
    comments: String,
    comments_en: String,
}

#[derive(Debug, PartialEq, Eq)]
struct Globals {
    encoding: u8,
    additional: u8,
    vertex_index_size: u8,
    texture_index_size: u8,
    material_index_size: u8,
    bone_index_size: u8,
    morph_index_size: u8,
    rigidbody_index_size: u8,
}

#[derive(Debug)]
enum Weight_deform {
    BDEF1(i32),
    BDEF2(i32),
    BDEF4(i32),
    SDEF(i32),
    QDEF(i32),
}

#[derive(Debug)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    additional: Vec<[f32; 4]>,
    weight_deform: Weight_deform,
    edge_scale: f32,
}

fn decode_string(x: &[u8], encode: u8) -> String {
    match encode {
        0u8 => bytes_to_u16v(x),
        1u8 => String::from_utf8(x.to_vec()).unwrap(),
        _ => "Unknown encoding".to_string(),
    }
}

fn bytes_to_u16v(input: &[u8]) -> String {
    let mut u16_vec = Vec::new();
    let iter = input.chunks(2);
    for x in iter {
        u16_vec.push(LittleEndian::read_u16(x));
    }
    return String::from_utf16(&u16_vec).unwrap();
}

named!(parse_header<&[u8], Header>, do_parse!(
    tag!("PMX ") >>
    version: le_f32 >>
    globals: length_bytes!(le_u8) >>
    text: count!(length_value!(le_i32, rest), 4) >>

    (Header {
        version: version,
        globals: globals,
        model_name: decode_string(text[0], globals[0]),
        model_name_en: decode_string(text[1], globals[0]),
        comments: decode_string(text[2], globals[0]),
        comments_en: decode_string(text[3], globals[0]),
    })
));

named!(parse_pmx<&[u8], Pmx>, do_parse!(
    header: parse_header >>
    //vertices: length_count!(le_i32, parse_vertex) >>
    rest >>

    (Pmx {
        header: header,
        //vertices: vertices,
    })
));

#[test]
fn mytest() {
    /*
    let head_pattern = [0x50u8, 0x4Du8, 0x58u8, 0x20u8, // b"PMX "
                        0x00u8, 0x00u8, 0x00u8, 0x40u8, // 2.0f32 8u8
                        0x08u8, 0x00u8, 0x00u8, 0x02u8,
                        0x01u8, 0x01u8, 0x02u8, 0x02u8,
                        0x02u8];
    let r = parse_header(&head_pattern);
    */
    let head_pattern = include_bytes!("../asset/江風ver1.05.pmx");
    let r = parse_pmx(head_pattern); //parse_header(head_pattern);
    if let Done(_, pmx) = r {
        assert_eq!(pmx.header,
                   //Done(&b""[..],
                   Header {
                       version: 2.0,
                       globals: &[0x00u8, 0x00u8, 0x02u8, 0x01u8, 0x01u8, 0x02u8, 0x02u8, 0x02u8],
                       model_name: String::from("江風"),
                       model_name_en: String::from("Model Name"),
                       comments: String::from("江風\r\n\r\nモデル制作：cham"),
                       comments_en: String::from("Comment"),
                   }); //);
    }
}