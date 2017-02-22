use nom::{le_f32, le_u8, le_i32, rest};
use nom::IResult::*;
use byteorder::{ByteOrder, LittleEndian};
use encoding::{Encoding, DecoderTrap};
use encoding::all::{UTF_8, UTF_16LE};

#[derive(Debug)]
pub struct Pmx {
    header: Header,
    //vertices: Vec<Vertex>,
}

#[derive(Debug, PartialEq)]
struct Header {
    version: f32,
    globals: Globals, //&'a [u8],
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

impl Globals {
    fn from_slice(input: &[u8]) -> Globals {
        Globals {
            encoding: input[0],
            additional: input[1],
            vertex_index_size: input[2],
            texture_index_size: input[3],
            material_index_size: input[4],
            bone_index_size: input[5],
            morph_index_size: input[6],
            rigidbody_index_size: input[7],
        }
    }
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

fn decode_text(x: &[u8], encoding: u8) -> String {
    match encoding {
        0u8 => UTF_16LE.decode(x, DecoderTrap::Strict).unwrap(),
        1u8 => String::from_utf8(x.to_vec()).unwrap(),
        _ => "Unknown encoding".to_string(),
    }
}

named!(parse_header<&[u8], Header>, do_parse!(
    tag!("PMX ") >>
    version: le_f32 >>
    globals: map!(length_bytes!(le_u8), Globals::from_slice) >>
    text: count!(map!(length_value!(le_i32, rest), |x| decode_text(x, globals.encoding)), 4) >>

    (Header {
        version: version,
        globals: globals,
        model_name: text[0].clone(),
        model_name_en: text[1].clone(),
        comments: text[2].clone(),
        comments_en: text[3].clone(),
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
                       globals: Globals::from_slice(&[0x00u8, 0x00u8, 0x02u8, 0x01u8, 0x01u8, 0x02u8, 0x02u8, 0x02u8]),
                       model_name: String::from("江風"),
                       model_name_en: String::from("Model Name"),
                       comments: String::from("江風\r\n\r\nモデル制作：cham"),
                       comments_en: String::from("Comment"),
                   }); //);
    }
}