use super::Load;
use super::newtypes::*;
use std::io::{Error, Read, Result};

use byteorder::{ByteOrder, ReadBytesExt, LE};
use enumflags::BitFlags;
use num_traits::{Bounded, FromPrimitive};
use pod_io::{Decode, Nil};

fn err<T: AsRef<str>>(s: T) -> Error {
    use std::io::ErrorKind;
    Error::new(ErrorKind::Other, s.as_ref())
}

fn create_vec<T>(n: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(n);
    unsafe {
        v.set_len(n);
    }
    v
}

struct PmxHelper<R> {
    read_string: fn(rdr: &mut R) -> Result<String>,
    additional: usize,
    read_vertex_index: fn(rdr: &mut R) -> Result<i32>,
    read_texture_index: fn(rdr: &mut R) -> Result<i32>,
    read_material_index: fn(rdr: &mut R) -> Result<i32>,
    read_bone_index: fn(rdr: &mut R) -> Result<i32>,
    read_morph_index: fn(rdr: &mut R) -> Result<i32>,
    read_rigidbody_index: fn(rdr: &mut R) -> Result<i32>,
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
    fn read_index<T: Decode<R, Nil>>(r: &mut R) -> Result<i32>
    where
        i32: ::std::convert::From<T>,
        T: Bounded + ::std::cmp::PartialEq,
    {
        let raw = T::decode::<LE>(r, Nil)?;
        if raw == T::max_value() {
            Ok(-1)
        } else {
            Ok(i32::from(raw))
        }
    }
    fn from_header(h: &Header) -> Result<Self> {
        let read_string = match h.encode {
            0 => Self::read_utf16_string,
            1 => Self::read_utf8_string,
            _ => return Err(err("unknown encoding")),
        };
        macro_rules! fn_index {
            ($size:expr) => {
                match $size {
                    1 => Self::read_index::<u8>,
                    2 => Self::read_index::<u16>,
                    4 => Self::read_index::<i32>,
                    _ => return Err(err("unknown index size")),
                }
            };
        }
        let read_vertex_index = fn_index!(h.vertex_index_size);
        let read_texture_index = fn_index!(h.texture_index_size);
        let read_material_index = fn_index!(h.material_index_size);
        let read_bone_index = fn_index!(h.bone_index_size);
        let read_morph_index = fn_index!(h.morph_index_size);
        let read_rigidbody_index = fn_index!(h.rigidbody_index_size);
        Ok(PmxHelper::<R> {
            read_string,
            additional: h.additional as usize,
            read_vertex_index,
            read_texture_index,
            read_material_index,
            read_bone_index,
            read_morph_index,
            read_rigidbody_index,
        })
    }
}

#[derive(Debug)]
pub struct PmxFile {
    magic: [u8; 4],
    header: Header,
    pub model_name: Name,
    pub comment: Name,
    pub model: Model,
}

impl Load for PmxFile {
    fn load<R: Read>(rdr: &mut R) -> Result<PmxFile> {
        let magic = <[u8; 4]>::decode::<LE>(rdr, Nil)?;
        if &magic != b"PMX " {
            return Err(err("Unknown Format"));
        }
        let header = Header::decode::<LE>(rdr, Nil)?;
        println!("{:?}", header);
        let helper = PmxHelper::from_header(&header)?;
        let model_name = Name::decode::<LE>(rdr, &helper)?;
        println!("{:?}", model_name);
        let comment = Name::decode::<LE>(rdr, &helper)?;
        println!("{:?}", comment);
        let model = Model::decode::<LE>(rdr, &helper)?;
        Ok(PmxFile { magic, header, model_name, comment, model })
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
pub struct PmxString(pub String);

impl<'a, R: Read> Decode<R, &'a fn(r: &mut R) -> Result<String>> for PmxString {
    fn decode<B: ByteOrder>(r: &mut R, p: &fn(r: &mut R) -> Result<String>) -> Result<PmxString> {
        Ok(PmxString(p(r)?))
    }
}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
pub struct Name {
    #[Arg = "&p.read_string"]
    pub jp: PmxString,
    #[Arg = "&p.read_string"]
    pub en: PmxString,
}

#[derive(Debug)]
pub struct Index(pub i32);

impl<'a, R: Read> Decode<R, &'a fn(r: &mut R) -> Result<i32>> for Index {
    fn decode<B: ByteOrder>(r: &mut R, p: &fn(r: &mut R) -> Result<i32>) -> Result<Index> {
        Ok(Index(p(r)?))
    }
}

impl BigStruct for Vertex {}
impl BigStruct for Index {}
impl BigStruct for Texture {}
impl BigStruct for Material {}
impl BigStruct for Bone {}
impl BigStruct for IKLink {}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
pub struct Model {
    #[Arg = "p"]
    pub vertices: Array<Vertex>,
    #[Arg = "&p.read_vertex_index"]
    pub face_indices: Array<Index>,
    #[Arg = "p"]
    pub textures: Array<Texture>,
    #[Arg = "p"]
    pub materials: Array<Material>,
    #[Arg = "p"]
    pub bones: Array<Bone>,
}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
pub struct Vertex {
    pub position: Vec3,
    normal: Vec3,
    pub uv: Vec2,
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
    BDEF4 { indices: [i32; 4], weights: [f32; 4] },
    SDEF { indices: [i32; 2], weight: f32, c: Vec3, r0: Vec3, r1: Vec3 },
    QDEF { indices: [i32; 4], weights: [f32; 4] },
}

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for BoneWeight {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<BoneWeight> {
        use self::BoneWeight::{BDEF1, BDEF2, BDEF4, QDEF, SDEF};
        let fi = p.read_bone_index;
        let fw = |r: &mut R| f32::decode::<LE>(r, Nil);
        let fv = |r: &mut R| Vec3::decode::<LE>(r, Nil);
        let ty = u8::decode::<LE>(r, Nil)?;
        let bone_weight = match ty {
            0 => BDEF1 { index: fi(r)? },
            1 => BDEF2 { indices: [fi(r)?, fi(r)?], weight: fw(r)? },
            2 => BDEF4 {
                indices: [fi(r)?, fi(r)?, fi(r)?, fi(r)?],
                weights: [fw(r)?, fw(r)?, fw(r)?, fw(r)?],
            },
            3 => SDEF {
                indices: [fi(r)?, fi(r)?],
                weight: fw(r)?,
                c: fv(r)?,
                r0: fv(r)?,
                r1: fv(r)?,
            },
            4 => QDEF {
                indices: [fi(r)?, fi(r)?, fi(r)?, fi(r)?],
                weights: [fw(r)?, fw(r)?, fw(r)?, fw(r)?],
            },
            _ => return Err(err(format!("Invalid BoneWeight Type {}", ty))),
        };
        Ok(bone_weight)
    }
}

#[derive(Debug)]
pub struct Texture(pub PmxString);

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for Texture {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<Texture> {
        let s = (p.read_string)(r)?;
        Ok(Texture(PmxString(s)))
    }
}

#[derive(EnumFlags, Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawModeFlags {
    TwoSided = 0x01,
    GroundShadow = 0x02,
    CastSelfShadow = 0x04,
    RecieveSelfShadow = 0x08,
    DrawEdge = 0x10,
    VertexColor = 0x20,
    DrawPoint = 0x40,
    DrawLine = 0x80,
}

impl_decode_modeset!(DrawModeFlags, u8);

#[derive(Primitive, Debug, Clone, Copy)]
#[repr(u8)]
enum SphereMode {
    NONE = 0,
    MUL = 1,
    ADD = 2,
    SUB = 3,
}

impl_decode_mode!(SphereMode, u8);

#[derive(Primitive, Debug)]
#[repr(u8)]
enum ToonMode {
    Separate = 0,
    Common = 1,
}

impl_decode_mode!(ToonMode, u8);

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
pub struct Material {
    #[Arg = "p"]
    pub name: Name,
    diffuse: Vec4,
    specular: Vec3,
    intensity: f32,
    ambient: Vec3,
    draw_mode: ModeSet<DrawModeFlags>,
    edge_color: Vec4,
    edge_size: f32,
    #[Arg = "&p.read_texture_index"]
    pub texture_id: Index,
    #[Arg = "&p.read_texture_index"]
    pub sphere_texture_id: Index,
    sphere_mode: SphereMode,
    toon_mode: ToonMode,
    #[Arg = "&p.read_texture_index"]
    pub toon_texture_id: Index,
    #[Arg = "&p.read_string"]
    memo: PmxString,
    pub num_vertex_indices: i32,
}

#[derive(EnumFlags, Debug, Clone, Copy)]
#[repr(u16)]
pub enum BoneFlags {
    /// 0: position, 1: bone ID
    TargetMode = 0x0001,
    CanRotate = 0x0002,
    CanTranslate = 0x0004,
    Visible = 0x0008,
    CanControl = 0x0010,
    IK = 0x0020,
    AppendLocal = 0x0080,
    AppendRotate = 0x0100,
    AppendTranslate = 0x0200,
    AxesFixed = 0x0400,
    LocalAxes = 0x0800,
    DeformAfterPhysics = 0x1000,
    DeformOuterParent = 0x2000,
}

impl_decode_modeset!(BoneFlags, u16);

#[derive(Debug)]
struct IKLink {
    bone_id: Index,
    /// Ret: Some(min, max)
    limits: Option<(Vec3, Vec3)>,
}

impl<'a, R: Read> Decode<R, &'a PmxHelper<R>> for IKLink {
    fn decode<B: ByteOrder>(r: &mut R, p: &PmxHelper<R>) -> Result<IKLink> {
        let bone_id = Index::decode::<LE>(r, &p.read_bone_index)?;
        let limits = match r.read_u8()? {
            0 => None,
            1 => Some((Vec3::decode::<LE>(r, Nil)?, Vec3::decode::<LE>(r, Nil)?)),
            _ => return Err(err("Invalid bool value")),
        };
        Ok(IKLink { bone_id, limits })
    }
}

#[derive(Debug, Decode)]
#[Parameter = "&'a PmxHelper<R>"]
pub struct Bone {
    #[Arg = "&p.read_string"]
    name: PmxString,
    #[Arg = "&p.read_string"]
    name_en: PmxString,
    position: Vec3,
    #[Arg = "&p.read_bone_index"]
    parent_id: Index,
    deform_depth: i32,
    flags: ModeSet<BoneFlags>,
    #[Arg = "(p, &flags)"]
    extra: BoneExtraInfo,
}

#[derive(Debug)]
struct BoneExtraInfo {
    position_offset: Option<Vec3>,
    link_id: Option<Index>,
    /// Ret: Some(bone_id, weight)
    append: Option<(Index, f32)>,
    fixed_axes: Option<Vec3>,
    /// Ret: Some(rotX, rotZ)
    local_rot: Option<(Vec3, Vec3)>,
    key_value: Option<i32>,
    /// Ret: Some(bone_id, num_iterations, limit, vec![IKLink, n])
    ik: Option<(Index, i32, f32, Array<IKLink>)>,
}

impl<'a, 'b, R: Read> Decode<R, (&'a PmxHelper<R>, &'b ModeSet<BoneFlags>)> for BoneExtraInfo {
    fn decode<B: ByteOrder>(r: &mut R, p: (&PmxHelper<R>, &ModeSet<BoneFlags>)) -> Result<BoneExtraInfo> {
        use self::BoneFlags::*;
        let (helper, flags) = p;
        let (position_offset, link_id) = if flags.contains(TargetMode) {
            (None, Some(Index::decode::<LE>(r, &helper.read_bone_index)?))
        } else {
            (Some(Vec3::decode::<LE>(r, Nil)?), None)
        };
        let append = if flags.contains(AppendRotate) || flags.contains(AppendTranslate) {
            Some((Index::decode::<LE>(r, &helper.read_bone_index)?, f32::decode::<LE>(r, Nil)?))
        } else {
            None
        };
        let fixed_axes = if flags.contains(AxesFixed) { Some(Vec3::decode::<LE>(r, Nil)?) } else { None };
        let local_rot = if flags.contains(LocalAxes) { Some((Vec3::decode::<LE>(r, Nil)?, Vec3::decode::<LE>(r, Nil)?)) } else { None };
        let key_value = if flags.contains(DeformOuterParent) { Some(i32::decode::<LE>(r, Nil)?) } else { None };
        let ik = if flags.contains(IK) {
            Some((Index::decode::<LE>(r, &helper.read_bone_index)?, i32::decode::<LE>(r, Nil)?, f32::decode::<LE>(r, Nil)?, Array::<IKLink>::decode::<LE>(r, helper)?))
        } else {
            None
        };

        Ok(BoneExtraInfo {
            position_offset,
            link_id,
            append,
            fixed_axes,
            local_rot,
            key_value,
            ik,
        })
    }
}

#[derive(Primitive, Debug, Clone, Copy)]
#[repr(u8)]
enum MorphType {
    Group = 0,
    Position = 1,
    Bone = 2,
    UV = 3,
    AddUV1 = 4,
    AddUV2 = 5,
    AddUV3 = 6,
    AddUV4 = 7,
    Material = 8,
    Flip = 9,
    Impulse = 10,
}

impl_decode_mode!(MorphType, u8);
