// Reference:
// https://gist.github.com/ulrikdamm/8274171
// https://gist.github.com/felixjones/f8a06bd48f9da9a4539f
// https://github.com/benikabocha/saba
use nom::{le_f32, le_u8, le_i8, le_i16, le_i32};
use nom::IResult;
use nom::IResult::Done;
use encoding::{Encoding, DecoderTrap};
use encoding::all::UTF_16LE;
use types::{Vec2, Vec3, Vec4};
use traits::Parse;

/*
#[derive(Debug)]
pub struct Pmx {
    header: Header, 
    vertices: Vec<Vertex>,
}
*/

#[derive(Debug, PartialEq)]
struct Header {
    version: f32,
    globals: Globals,
    model_name: String,
    model_name_en: String,
    comments: String,
    comments_en: String,
}

impl Header {
    named!(parse<&[u8], Header>, do_parse!(
        tag!("PMX ") >>
        version: le_f32 >>
        globals: map!(length_bytes!(le_u8), Globals::from_slice) >>
        //text: count!(map!(length_data!(le_i32), |x| decode_text(x, globals.encoding)), 4) >>
        text1: apply!(length_string, globals.encoding) >>
        text2: apply!(length_string, globals.encoding) >>
        text3: apply!(length_string, globals.encoding) >>
        text4: apply!(length_string, globals.encoding) >>

        (Header {
            version: version,
            globals: globals,
            model_name: text1,
            model_name_en: text2,
            comments: text3,
            comments_en: text4,
            /*
            model_name: text[0].clone(),
            model_name_en: text[1].clone(),
            comments: text[2].clone(),
            comments_en: text[3].clone(),
            */
        })
    ));
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
struct BoneIndexWeight<T> {
    index: T,
    weight: f32,
}

/*
#[derive(Debug)]
enum WeightDeform<T> {
    BDEF1 { bones: [BoneIndexWeight<T>; 1] },
    BDEF2 { bones: [BoneIndexWeight<T>; 2] },
    BDEF4 { bones: [BoneIndexWeight<T>; 4] },
    SDEF  { bones: [BoneIndexWeight<T>; 2], c: Vec3, r0: Vec3, r1: Vec3 },
    QDEF  { bones: [BoneIndexWeight<T>; 4] },
}

impl WeightDeform<T> {
    fn parse(input: &[u8], bone_idx_size: usize, weightDeformType: u8) -> IResult<&[u8], WeightDeform<u32> > {
        let n = match weightDeformType {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 2,
            4 => 4,
        };
        let getBones = match bone_idx_size {
            1 => count!(input, i8, le_i8, n),
            2 => count!(input, i16, le_i16, n),
            4 => count!(input, i32, le_i32, n),
            _ => unreachable!(),
        };
        do_parse!(input,
           bones: getBones >>

           (WeightDeform::BDEF1 {
               bones: bones,
           })
        )
    }
}

#[derive(Debug)]
struct Vertex {
    pos: Vec3,
    normal: Vec3,
    uv: Vec2,
    additional: Vec<Vec4>,
    /// We extend all index size to 4
    weight_deform: WeightDeform<i32>,
    edge_scale: f32,
}

impl Vertex {
    fn parse(input: &[u8], additional_n: usize, bone_idx_size: usize) -> IResult<&[u8], Vertex> {
        do_parse!(input,
            pos: call!(Vec3::parse) >>
            normal: call!(Vec3::parse) >>
            uv: call!(Vec2::parse) >>
            additional: count!(call!(Vec4::parse), additional_n) >>
            weight_deform_type: take!(1) >>
            weight_deform: apply!(WeightDeform::parse, bone_idx_size, *weight_deform_type) >>

            (Vertex {
                pos: pos,
                normal: normal,
                uv: uv,
                additional: additional,
                weight_deform: weight_deform,
                edge_scale: 0.0,
            })
        )
    }
}
*/

fn decode_text(x: &[u8], encoding: u8) -> String {
    match encoding {
        0u8 => UTF_16LE.decode(x, DecoderTrap::Strict).unwrap(),
        1u8 => String::from_utf8(x.to_vec()).unwrap(),
        _ => panic!("Unknown encoding"),
    }
}

named_args!(length_string(encode: u8)<String>, map!(length_data!(le_i32), |x| decode_text(x, encode)));

/*
named!(pub parse_pmx<&[u8], Pmx>, do_parse!(
    header: parse_header >>
    vertices: length_count!(le_i32, apply!(
        parse_vertex, header.globals.additional as usize, header.globals.bone_index_size)) >>

    (Pmx {
        header: header,
        vertices: vertices,
    })
));
*/

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_bytes() -> Vec<u8> {
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;

        let f = File::open("asset/江風ver1.05.pmx").unwrap();
        let mut buf_reader = BufReader::new(f);
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents);
        contents
    }

    #[test]
    fn test_header() {
    /*
        let head_pattern = [0x50u8, 0x4Du8, 0x58u8, 0x20u8, // b"PMX "
                            0x00u8, 0x00u8, 0x00u8, 0x40u8, // 2.0f32 8u8
                            0x08u8, 0x00u8, 0x00u8, 0x02u8,
                            0x01u8, 0x01u8, 0x02u8, 0x02u8,
                            0x02u8];
        let r = parse_header(&head_pattern);
        let (_, header) = r.unwrap();
                            */
        use std::mem;
        let data = get_test_bytes();
        let (_, header) = Header::parse(&data[..mem::size_of::<Header>()]).unwrap();
        assert_eq!(header,
           Header {
               version: 2.0,
               globals: Globals::from_slice(&[0x00u8, 0x00u8, 0x02u8, 0x01u8, 0x01u8,
                                              0x02u8, 0x02u8, 0x02u8]),
               model_name: String::from("江風"),
               model_name_en: String::from("Model Name"),
               comments: String::from("江風\r\n\r\nモデル制作：cham"),
               comments_en: String::from("Comment"),
           });
    }
}
