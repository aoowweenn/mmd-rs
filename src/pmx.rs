// Reference:
// https://gist.github.com/ulrikdamm/8274171
// https://gist.github.com/felixjones/f8a06bd48f9da9a4539f
// https://github.com/benikabocha/saba
use nom::{le_f32, le_i16, le_i32, le_i8, le_u16, le_u32, le_u8};
use nom::IResult;
use types::{PmxString, Vec2, Vec3, Vec4};
use traits::Parse;

/// For vertex index type
fn le_unsigned(input: &[u8], size: usize) -> IResult<&[u8], i32> {
    match size {
        1 => le_u8(input).map(i32::from),
        2 => le_u16(input).map(i32::from),
        4 => le_i32(input),
        _ => unreachable!(),
    }
}

/// For other index types (Bone, Texture, Material, Morph, Rigibody)
fn le_integer(input: &[u8], size: usize) -> IResult<&[u8], i32> {
    match size {
        1 => le_i8(input).map(i32::from),
        2 => le_i16(input).map(i32::from),
        4 => le_i32(input),
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub struct Pmx {
    header: Header,
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    textures: Vec<Texture>,
}

impl Pmx {
    named!(pub parse<&[u8], Pmx>, do_parse!(
        header: call!(Header::parse) >>
        vertices: length_count!(le_i32, apply!(Vertex::parse, header.globals.additional as usize, header.globals.bone_index_size as usize)) >>
        num_face_mul_3: le_i32 >>
        faces: count!(apply!(Face::parse, header.globals.vertex_index_size as usize), num_face_mul_3 as usize / 3) >>
        textures: length_count!(le_u32, apply!(Texture::parse, header.globals.encoding)) >>

        (Pmx {
            header,
            vertices,
            faces,
            textures,
        })
    ));
}

#[derive(Debug, PartialEq)]
struct Header {
    version: f32,
    globals: Globals,
    model_name: PmxString,
    model_name_en: PmxString,
    comment: PmxString,
    comment_en: PmxString,
}

impl Header {
    named!(parse<&[u8], Header>, do_parse!(
        tag!("PMX ") >>
        version: le_f32 >>
        globals: map!(length_bytes!(le_u8), Globals::from) >>
        text: count!(call!(PmxString::parse, globals.encoding), 4) >>

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

impl<'a> From<&'a [u8]> for Globals {
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

#[allow(dead_code)]
#[repr(u8)]
enum BoneWeightType {
    BDEF1 = 0,
    BDEF2 = 1,
    BDEF4 = 2,
    SDEF = 3,
    QDEF = 4,
}

/// when index = -1, we neglect the bone.
#[derive(Debug, PartialEq)]
enum BoneWeight {
    BDEF1 { index: i32 },
    BDEF2 { indices: [i32; 2], weight: f32 },
    BDEF4 {
        indices: [i32; 4],
        weights: [f32; 4],
    },
    SDEF {
        indices: [i32; 2],
        weight: f32,
        c: Vec3,
        r0: Vec3,
        r1: Vec3,
    },
    QDEF {
        indices: [i32; 4],
        weights: [f32; 4],
    },
}

impl BoneWeight {
    /// TODO: wait for const generics(RFC 2000) to revise
    fn parse(input: &[u8], bone_idx_size: usize, bone_weight_type_u8: u8) -> IResult<&[u8], Self> {
        let bone_weight_type = unsafe { ::std::mem::transmute(bone_weight_type_u8) };

        match bone_weight_type {
            BoneWeightType::BDEF1 => le_integer(input, bone_idx_size).map(|index| BoneWeight::BDEF1 { index }),
            BoneWeightType::BDEF2 => do_parse!(input,
                    indices: count_fixed!(i32, apply!(le_integer, bone_idx_size), 2) >>
                    weight: le_f32 >>
                    (BoneWeight::BDEF2 { indices, weight })
                ),
            BoneWeightType::BDEF4 => do_parse!(input,
                    indices: count_fixed!(i32, apply!(le_integer, bone_idx_size), 4) >>
                    weights: count_fixed!(f32, le_f32, 4) >>
                    (BoneWeight::BDEF4 { indices, weights })
                ),
            BoneWeightType::SDEF => do_parse!(input,
                    indices: count_fixed!(i32, apply!(le_integer, bone_idx_size), 2) >>
                    weight: le_f32 >>
                    c: call!(Vec3::parse) >>
                    r0: call!(Vec3::parse) >>
                    r1: call!(Vec3::parse) >>
                    (BoneWeight::SDEF { indices, weight, c, r0, r1 })
                ),
            BoneWeightType::QDEF => do_parse!(input,
                    indices: count_fixed!(i32, apply!(le_integer, bone_idx_size), 4) >>
                    weights: count_fixed!(f32, le_f32, 4) >>
                    (BoneWeight::QDEF { indices, weights })
                ),
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
    bone_weight: BoneWeight,
    edge_scale: f32,
}

impl Vertex {
    fn parse(input: &[u8], additional_n: usize, bone_idx_size: usize) -> IResult<&[u8], Vertex> {
        do_parse!(input,
            pos: call!(Vec3::parse) >>
            normal: call!(Vec3::parse) >>
            uv: call!(Vec2::parse) >>
            additional: count!(call!(Vec4::parse), additional_n) >>
            bone_weight_type_u8: take!(1) >>
            bone_weight: apply!(BoneWeight::parse, bone_idx_size, bone_weight_type_u8[0]) >>
            edge_scale: le_f32 >>

            (Vertex {
                pos,
                normal,
                uv,
                additional,
                bone_weight,
                edge_scale,
            })
        )
    }
}

#[derive(Debug)]
struct Face(i32, i32, i32);

impl Face {
    fn parse(input: &[u8], vertex_idx_size: usize) -> IResult<&[u8], Face> {
        count!(input, apply!(le_unsigned, vertex_idx_size), 3).map(|o| Face(o[0], o[1], o[2]))
    }
}

#[derive(Debug)]
struct Texture(PmxString);

impl Texture {
    fn parse(input: &[u8], encoding: u8) -> IResult<&[u8], Texture> {
        map!(
            input,
            apply!(PmxString::parse, encoding),
            |o| { Texture(o) }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::DataBlock;

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

        let model_name = PmxString::from("モデル名前");
        let model_name_en = PmxString::from("Model Name");
        let comment = PmxString::from("コメント");
        let comment_en = PmxString::from("Comment");

        let data = DataBlock::new() << magic.as_bytes() << version << num_globals << &globals[..] << &model_name << &model_name_en << &comment << &comment_en;

        let h = Header {
            version,
            globals: Globals::from(&globals[..]),
            model_name,
            model_name_en,
            comment,
            comment_en,
        };

        let (_, header) = Header::parse(&data.unwrap()).unwrap();

        assert_eq!(header, h);
    }

    #[test]
    fn test_vertex() {
        let pos = Vec3::unit_z();
        let normal = Vec3::unit_y();
        let uv = Vec2::unit_x();
        let additional = vec![Vec4::unit_w()];
        let indices = [111, 999];
        let weight = 0.4;
        let bone_type = BoneWeightType::BDEF2;
        let bone_weight = BoneWeight::BDEF2 { indices, weight };
        let edge_scale = 9.9f32;

        let data = DataBlock::new() << &pos[..] << &normal[..] << &uv[..] << &additional[0][..] << (bone_type as u8) << indices[0] << indices[1] << weight << edge_scale;

        let additional_n = 1;
        let bone_idx_size = 4;
        let (_, vertex) = Vertex::parse(&data.unwrap(), additional_n, bone_idx_size).unwrap();
        debug_assert_eq!(
            vertex,
            Vertex {
                pos,
                normal,
                uv,
                additional,
                bone_weight,
                edge_scale,
            }
        );
    }
}
