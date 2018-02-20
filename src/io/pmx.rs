use super::Load;
use super::newtypes::*;
use std::io::{Read, Result, Error};

use byteorder::{ByteOrder, LE, ReadBytesExt};
use pod_io::{Decode, Nil};

fn err(s: &str) -> Error {
    use std::io::ErrorKind;
    Error::new(ErrorKind::Other, s)
}

fn create_vec<T>(n: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(n);
    unsafe { v.set_len(n); }
    v
}

struct PmxHelper<R> {
    read_string: fn(rdr: &mut R) -> Result<String>,
    additional: usize,
    read_vertex_index: fn(rdr: &mut R) -> Result<i32>,
    /*
    read_texture_index: fn(rdr: &mut R) -> i32,
    read_material_index: fn(rdr: &mut R) -> i32,
    read_bone_index: fn(rdr: &mut R) -> i32,
    read_morph_index: fn(rdr: &mut R) -> i32,
    read_rigidbody_index: fn(rdr: &mut R) -> i32,
    */
}

impl<R: Read> PmxHelper<R> {
    fn read_utf16_string(r: &mut R) -> Result<String> {
        let n = u32::decode::<LE>(r, Nil)? as usize / 2;
        let mut buf = create_vec(n);
        r.read_u16_into::<LE>(&mut buf)?;
        Ok(String::from_utf16(&buf).unwrap())
    }
    fn read_utf8_string(r: &mut R) -> Result<String> {
        let n = u32::decode::<LE>(r, Nil)? as usize;
        let mut buf = create_vec(n);
        r.read_exact(&mut buf)?;
        Ok(String::from_utf8(buf).unwrap())
    }
    fn read_index<T: Decode<R, Nil>>(r: &mut R) -> Result<i32> where i32: ::std::convert::From<T> {
        let raw = T::decode::<LE>(r, Nil)?;
        Ok(i32::from(raw))
    }
    fn from_header(h: &Header) -> Result<Self> {
        let read_string = match h.encode {
            0 => Self::read_utf16_string,
            1 => Self::read_utf8_string,
            _ => return Err(err("unknown encoding")),
        };
        let read_vertex_index = match h.vertex_index_size {
            1 => Self::read_index::<i8>,
            2 => Self::read_index::<i16>,
            4 => Self::read_index::<i32>,
            _ => return Err(err("unknown index size")),
        };
        Ok(PmxHelper::<R> {
            read_string,
            additional: h.additional as usize,
            read_vertex_index,
        })
    }
}

#[derive(Debug)]
pub struct PmxFile {
    magic: [u8; 4],
    header: Header,
    model_name: Name,
    comment: Name,
    model: Model,
}

impl Load for PmxFile {
    fn load<R: Read>(rdr: &mut R) -> Result<PmxFile> {
        let magic = <[u8; 4]>::decode::<LE>(rdr, Nil)?;
        if &magic != b"PMX " {
            return Err(err("Unknown Format"));
        }
        let header = Header::decode::<LE>(rdr, Nil)?;
        let helper = PmxHelper::from_header(&header)?;
        let model_name = Name::decode::<LE>(rdr, &helper)?;
        let comment = Name::decode::<LE>(rdr, &helper)?;
        let model = Model::decode::<LE>(rdr, &helper)?;
        Ok(PmxFile{magic, header, model_name, comment, model})
    }
}

#[derive(Debug, Decode)]
struct Header {
    version: f32,
    dummy: u8,
    encode: u8,
    additional: u8,
    vertex_index_size: u8,
    texture_index_size: u8,
    material_index_size: u8,
    bone_index_size: u8,
    morph_index_size: u8,
    rigidbody_index_size: u8,
}

#[derive(Debug)]
struct Name {
    jp: String,
    en: String,
}

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for Name {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<Name> {
        let jp = (p.read_string)(r)?;
        let en = (p.read_string)(r)?;
        Ok(Name { jp, en })
    }
}

#[derive(Debug)]
struct Index(i32);

impl<'a, R: Read> Decode<R, &'a fn(r: &mut R) -> Result<i32>> for Index {
    fn decode<B: ByteOrder>(r: &mut R, p: &fn(r: &mut R) -> Result<i32>) -> Result<Index> {
        Ok(Index(p(r)?))
    }
}

impl BigStruct for Vertex {}
impl BigStruct for Index {}
impl BigStruct for Texture {}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
struct Model {
    #[Arg = "p"]
    vertices: Array<Vertex>,
    #[Arg = "&p.read_vertex_index"]
    face_indices: Array<Index>,
    #[Arg = "p"]
    textures: Array<Texture>,
}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
struct Vertex{
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    #[Arg = "p.additional"]
    additional: Array<Vec4>,
    #[Arg = "p"]
    bone_weight: BoneWeight,
    edge_scale: f32,
}

#[derive(Debug)]
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

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for BoneWeight {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<BoneWeight> {
        use self::BoneWeight::{BDEF1, BDEF2, BDEF4, SDEF, QDEF};
        let fi = p.read_vertex_index;
        let fw = |r: &mut R| f32::decode::<LE>(r, Nil);
        let fv = |r: &mut R| Vec3::decode::<LE>(r, Nil);
        let ty = u8::decode::<LE>(r, Nil)?;
        let bone_weight = match ty {
            0 => BDEF1 { index: fi(r)? },
            1 => BDEF2 { indices: [fi(r)?, fi(r)?], weight: fw(r)? },
            2 => BDEF4 {
                indices: [fi(r)?, fi(r)?, fi(r)?, fi(r)?],
                weights: [fw(r)?, fw(r)?, fw(r)?, fw(r)? ]
            },
            3 => SDEF {
                indices: [fi(r)?, fi(r)?], weight: fw(r)?,
                c: fv(r)?, r0: fv(r)?, r1: fv(r)?
            },
            4 => QDEF {
                indices: [fi(r)?, fi(r)?, fi(r)?, fi(r)?],
                weights: [fw(r)?, fw(r)?, fw(r)?, fw(r)? ]
            },
            _ => return Err(err("Invalid BoneWeigth Type")),
        };
        Ok(bone_weight)
    }
}

#[derive(Debug)]
struct Texture(String);

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for Texture {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<Texture> {
        let s = (p.read_string)(r)?;
        Ok(Texture(s))
    }
}
