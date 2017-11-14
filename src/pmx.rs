// Reference:
// https://gist.github.com/ulrikdamm/8274171
// https://gist.github.com/felixjones/f8a06bd48f9da9a4539f
// https://github.com/benikabocha/saba
use nom::{le_f32, le_i16, le_i32, le_i8, le_u8};
use nom::IResult;
use encoding::{DecoderTrap, Encoding};
use encoding::all::UTF_16LE;
use types::{PmxString, Vec2, Vec3, Vec4};
use traits::Parse;

#[derive(Debug)]
pub struct Pmx {
    header: Header,
    vertices: Vec<Vertex>,
}

#[derive(Debug, PartialEq)]
struct Header {
    version: f32,
    globals: Globals,
    model_name: String,
    model_name_en: String,
    comment: String,
    comment_en: String,
}

impl Header {
    named!(parse<&[u8], Header>, do_parse!(
        tag!("PMX ") >>
        version: le_f32 >>
        globals: map!(length_bytes!(le_u8), Globals::from) >>
        text: count!(call!(length_string,globals.encoding), 4) >>

        ({
            let mut text = text;
            let comment_en = text.pop().unwrap();
            let comment = text.pop().unwrap();
            let model_name_en = text.pop().unwrap();
            let model_name = text.pop().unwrap();
            Header {
                version,
                globals,
                model_name,
                model_name_en,
                comment,
                comment_en,
            }
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
    fn from(input: &[u8]) -> Globals {
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

#[derive(Debug, PartialEq, Clone, Copy)]
struct BoneIndexWeight<T> {
    index: T,
    weight: f32,
}

impl BoneIndexWeight<i32> {
    fn parse(input: &[u8], index_num_bytes: usize, need_weight: bool) -> IResult<&[u8], Self> {
        let index_res = {
            match index_num_bytes {
                1 => le_i8(input).map(|o| o as i32),
                2 => le_i16(input).map(|o| o as i32),
                4 => le_i32(input),
                _ => unreachable!(),
            }
        };

        if !need_weight {
            return index_res.map(|o| {
                Self {
                    index: o,
                    weight: 1.0f32,
                }
            });
        }

        let (_, index) = index_res.unwrap();

        let weight_raw = &input[index_num_bytes..];

        le_f32(weight_raw).map(|weight| Self { index, weight })
    }
}

#[derive(Debug, PartialEq)]
enum WeightDeform<T> {
    BDEF1 { bones: [BoneIndexWeight<T>; 1] },
    BDEF2 { bones: [BoneIndexWeight<T>; 2] },
    BDEF4 { bones: [BoneIndexWeight<T>; 4] },
    SDEF {
        bones: [BoneIndexWeight<T>; 2],
        c: Vec3,
        r0: Vec3,
        r1: Vec3,
    },
    QDEF { bones: [BoneIndexWeight<T>; 4] },
}

impl WeightDeform<i32> {
    /// TODO: wait for const generics(RFC 2000) to revise
    fn parse(input: &[u8], bone_idx_size: usize, weight_deform_type: u8) -> IResult<&[u8], Self> {
        match weight_deform_type {
            0 => do_parse!(input,
                    bones: count_fixed!(BoneIndexWeight<i32>, call!(BoneIndexWeight::<i32>::parse, bone_idx_size, false), 1) >>
                    (WeightDeform::BDEF1 { bones })
                ),
            1 => do_parse!(input,
                    bones: count_fixed!(BoneIndexWeight<i32>, call!(BoneIndexWeight::<i32>::parse, bone_idx_size, true), 2) >>
                    (WeightDeform::BDEF2 { bones })
                ),
            2 => do_parse!(input,
                    bones: count_fixed!(BoneIndexWeight<i32>, call!(BoneIndexWeight::<i32>::parse, bone_idx_size, true), 4) >>
                    (WeightDeform::BDEF4 { bones })
                ),
            3 => do_parse!(input,
                    bones: count_fixed!(BoneIndexWeight<i32>, call!(BoneIndexWeight::<i32>::parse, bone_idx_size, true), 2) >>
                    c: call!(Vec3::parse) >>
                    r0: call!(Vec3::parse) >>
                    r1: call!(Vec3::parse) >>
                    (WeightDeform::SDEF { bones, c, r0, r1 })
                ),
            4 => do_parse!(input,
                    bones: count_fixed!(BoneIndexWeight<i32>, call!(BoneIndexWeight::<i32>::parse, bone_idx_size, true), 4) >>
                    (WeightDeform::QDEF { bones })
                ),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
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
            weight_deform: apply!(WeightDeform::parse, bone_idx_size, weight_deform_type[0]) >>
            edge_scale: le_f32 >>

            (Vertex {
                pos: pos,
                normal: normal,
                uv: uv,
                additional: additional,
                weight_deform: weight_deform,
                edge_scale: edge_scale,
            })
        )
    }
}

fn decode_text(x: &[u8], encoding: u8) -> String {
    match encoding {
        0u8 => UTF_16LE.decode(x, DecoderTrap::Strict).unwrap(),
        1u8 => String::from_utf8(x.to_vec()).unwrap(),
        _ => panic!("Unknown encoding"),
    }
}

named_args!(length_string(encode: u8)<String>, map!(length_data!(le_i32), |x| decode_text(x, encode)));

named!(pub parse_pmx<&[u8], Pmx>, do_parse!(
    header: call!(Header::parse) >>
    vertices: length_count!(le_i32, apply!(Vertex::parse, header.globals.additional as usize, header.globals.bone_index_size as usize)) >>

    (Pmx {
        header: header,
        vertices: vertices,
    })
));

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};

    fn get_test_bytes() -> Vec<u8> {
        use std::io::prelude::*;
        use std::io::BufReader;
        use std::fs::File;

        let f = File::open("asset/江風ver1.05.pmx").unwrap();
        let mut buf_reader = BufReader::new(f);
        let mut contents = Vec::new();
        buf_reader.read_to_end(&mut contents).unwrap();
        contents
    }

    #[test]
    fn test_bone_index_weight() {
        let data = [10u8, 0u8, 0u8, 0u8, 0x40u8];
        let (_, index_weight) = BoneIndexWeight::<i32>::parse(&data, 1, true).unwrap();
        assert_eq!(
            index_weight,
            BoneIndexWeight::<i32> {
                index: 10,
                weight: 2.0,
            }
        );
    }

    #[test]
    fn test_header() {
        let magic = String::from("PMX ");
        let version = 2.0f32;
        let num_globals = 8u8;
        let globals = [
            0x00u8,
            0x00u8,
            0x02u8,
            0x01u8,
            0x01u8,
            0x02u8,
            0x02u8,
            0x02u8,
        ];

        let mut head_pattern = magic.into_bytes();
        head_pattern.write_f32::<LittleEndian>(version).unwrap();
        head_pattern.push(num_globals);
        head_pattern.extend_from_slice(&globals);

        let model_name = PmxString::from("モデル名前");
        let model_name_en = PmxString::from("Model Name");
        let comment = PmxString::from("コメント");
        let comment_en = PmxString::from("Comment");

        let h = Header {
            version,
            globals: Globals::from(&globals),
            model_name: (*model_name).to_owned(),
            model_name_en: (*model_name_en).to_owned(),
            comment: (*comment).to_owned(),
            comment_en: (*comment_en).to_owned(),
        };

        head_pattern.append(&mut model_name.into());
        head_pattern.append(&mut model_name_en.into());
        head_pattern.append(&mut comment.into());
        head_pattern.append(&mut comment_en.into());
        let (_, header) = Header::parse(&head_pattern).unwrap();

        assert_eq!(header, h);
    }

    #[test]
    fn test_vertex() {
        let pos = Vec3::unit_z();
        let normal = Vec3::unit_y();
        let uv = Vec2::unit_x();
        let additional = vec![Vec4::unit_w()];
        let bones = [
            BoneIndexWeight::<i32> {
                index: 99,
                weight: 1.0,
            },
        ];
        let weight_deform = WeightDeform::BDEF1 { bones };
        let edge_scale = 9.9f32;

        let mut data = vec![];
        [&pos[..], &normal[..], &uv[..], &additional[0][..]]
            .concat()
            .into_iter()
            .for_each(|x| data.write_f32::<LittleEndian>(x).unwrap());
        data.push(0u8);
        data.write_i32::<LittleEndian>(bones[0].index).unwrap();
        data.write_f32::<LittleEndian>(edge_scale).unwrap();

        let additional_n = 1;
        let bone_idx_size = 4;
        let (_, vertex) = Vertex::parse(&data, additional_n, bone_idx_size).unwrap();
        debug_assert_eq!(
            vertex,
            Vertex {
                pos,
                normal,
                uv,
                additional,
                weight_deform,
                edge_scale,
            }
        );
    }
}
