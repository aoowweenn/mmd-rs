use num::FromPrimitive;
use enumflags::*;

use std::io;
use std::io::BufReader;
use std::path::Path;
use std::marker::Sized;

use byteorder::{ReadBytesExt, LE};

use encoding::all::UTF_16LE;
use encoding::DecoderTrap;
use encoding::Encoding;

use cgmath::{Vector2, Vector3, Vector4, Quaternion};

/*
#[derive(Debug)]
enum PmxError {
    Io(io::Error),
}
*/

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Quat = Quaternion<f32>;
pub type DrawMode = BitFlags<DrawModeFlags>;
pub type BoneMode = BitFlags<BoneFlags>;

trait FromReader<T> {
    fn from_reader<RExt: ReaderExt>(_rdr: &mut RExt) -> Result<Self, io::Error> where Self: Sized {
        Err(err_str("dummy"))
    }
    fn from_reader_arg<RExt: ReaderExt>(_rdr: &mut RExt, _t: T) -> Result<Self, io::Error> where Self: Sized {
        Err(err_str("dummy"))
    }
}

fn err_str(s: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::Other,
        s
    )
}

trait ReaderExt: ReadBytesExt {
    fn read_vec(&mut self, n: usize) -> Result<Vec<u8>, io::Error> {
        let mut vec = Vec::with_capacity(n);
        unsafe { vec.set_len(n) }
        self.read_exact(&mut vec)?;
        Ok(vec)
    }

    fn read_index(&mut self, isize: u8) -> Result<i32, io::Error> {
        let index = match isize {
            1 => {
                let i = self.read_i8()?;
                assert!(i >= -1);
                i as i32
            },
            2 => {
                let i = self.read_i16::<LE>()?;
                assert!(i >= -1);
                i as i32
            },
            4 => {
                let i = self.read_i32::<LE>()?;
                assert!(i >= -1);
                i
            }
            _ => return Err(err_str("invalid index size"))
        };
        Ok(index)
    }

    fn read_indices2(&mut self, isize: u8) -> Result<[i32; 2], io::Error> {
        Ok([self.read_index(isize)?, self.read_index(isize)?])
    }

    fn read_indices4(&mut self, isize: u8) -> Result<[i32; 4], io::Error> {
        Ok([self.read_index(isize)?, self.read_index(isize)?
        ,self.read_index(isize)?, self.read_index(isize)?])
    }

    fn read_array2(&mut self) -> Result<[f32; 2], io::Error> {
        let mut dst = [0.0; 2];
        self.read_f32_into::<LE>(&mut dst)?;
        Ok(dst)
    }

    fn read_array3(&mut self) -> Result<[f32; 3], io::Error> {
        let mut dst = [0.0; 3];
        self.read_f32_into::<LE>(&mut dst)?;
        Ok(dst)
    }

    fn read_array4(&mut self) -> Result<[f32; 4], io::Error> {
        let mut dst = [0.0; 4];
        self.read_f32_into::<LE>(&mut dst)?;
        Ok(dst)
    }

    fn read_vec2(&mut self) -> Result<Vec2, io::Error> {
        let array2 = self.read_array2()?;
        Ok(Vec2::from(array2))
    }

    fn read_vec3(&mut self) -> Result<Vec3, io::Error> {
        let array3 = self.read_array3()?;
        Ok(Vec3::from(array3))
    }

    fn read_vec4(&mut self) -> Result<Vec4, io::Error> {
        let array4 = self.read_array4()?;
        Ok(Vec4::from(array4))
    }

    /// TODO: make sure s first v last
    fn read_quat(&mut self) -> Result<Quat, io::Error> {
        let s = self.read_f32::<LE>()?;
        let v = self.read_vec3()?;
        Ok(Quat::from_sv(s, v))
    }

    /// TODO: handle String Error
    fn read_string(&mut self, n: usize) -> Result<String, io::Error> {
        Ok(String::from_utf8(self.read_vec(n)?).expect("Invalid String"))
    }

    /// TODO: handle encoding error
    fn read_nstring(&mut self, enc: StringEnc) -> Result<String, io::Error> {
        let n = self.read_u32::<LE>()? as usize;
        match enc {
            StringEnc::UTF16 => {
                let vec = self.read_vec(n)?;
                Ok(
                    UTF_16LE
                        .decode(&vec, DecoderTrap::Strict)
                        .expect("Not valid UTF16 string"),
                )
            }
            StringEnc::UTF8 => self.read_string(n),
        }
    }

    fn read_structs<'a, T: FromReader<&'a Globals>>(&mut self, n: usize, g: &'a Globals) -> Result<Vec<T>, io::Error> where Self: Sized{
        let mut v = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(T::from_reader_arg(self, g)?);
        }
        Ok(v)
    }
}

impl<T: ReadBytesExt> ReaderExt for T {}

#[derive(Debug)]
pub struct PmxFile {
    header: Header,
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
    bones: Vec<Bone>,
    morphs: Vec<Morph>,
}

impl FromReader<()> for PmxFile {
    fn from_reader<R: io::Read>(rdr: &mut R) -> Result<PmxFile, io::Error> {
        let header = Header::from_reader(rdr)?;

        let num_vertices = rdr.read_u32::<LE>()? as usize;
        let vertices = rdr.read_structs(num_vertices, &header.globals)?;

        let num_face_indices = rdr.read_u32::<LE>()? as usize;
        let num_faces = num_face_indices / 3;
        let faces = rdr.read_structs(num_faces, &header.globals)?;

        let num_textures = rdr.read_u32::<LE>()? as usize;
        let textures = rdr.read_structs(num_textures, &header.globals)?;
        
        let num_materials = rdr.read_u32::<LE>()? as usize;
        let materials = rdr.read_structs(num_materials, &header.globals)?;

        let num_bones = rdr.read_u32::<LE>()? as usize;
        let bones = rdr.read_structs(num_bones, &header.globals)?;
        
        let num_morphs = rdr.read_u32::<LE>()? as usize;
        let morphs = rdr.read_structs(num_morphs, &header.globals)?;

        Ok(PmxFile {
            header, vertices, faces,
            textures, materials, bones,
            morphs,
        })
    }
}

enum_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum StringEnc {
        UTF16,
        UTF8,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Globals {
    encoding: StringEnc,
    additional: u8,
    vertex_index_size: u8,
    texture_index_size: u8,
    material_index_size: u8,
    bone_index_size: u8,
    morph_index_size: u8,
    rigidbody_index_size: u8,
}

impl FromReader<()> for Globals {
    fn from_reader<RBExt: ReadBytesExt>(rdr: &mut RBExt) -> Result<Globals, io::Error> {
        let encoding = StringEnc::from_u8(rdr.read_u8()?).ok_or(err_str("Unknown String Encoding"))?;
        let additional = rdr.read_u8()?;
        let vertex_index_size = rdr.read_u8()?;
        let texture_index_size = rdr.read_u8()?;
        let material_index_size = rdr.read_u8()?;
        let bone_index_size = rdr.read_u8()?;
        let morph_index_size = rdr.read_u8()?;
        let rigidbody_index_size = rdr.read_u8()?;
        Ok(Globals {
            encoding,
            additional,
            vertex_index_size,
            texture_index_size,
            material_index_size,
            bone_index_size,
            morph_index_size,
            rigidbody_index_size,
        })
    }
}

#[derive(Debug)]
struct Header {
    //magic_id: String,
    magic_id: [u8; 4],
    version: f32,
    num_globals: u8,
    globals: Globals,
    model_name: String,
    model_name_en: String,
    comment: String,
    comment_en: String,
}

impl FromReader<()> for Header {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt) -> Result<Header, io::Error> {
        //let magic_id = rdr.read_string(4)?;
        let magic_id = [rdr.read_u8()?, rdr.read_u8()?, rdr.read_u8()?, rdr.read_u8()?];
        if &magic_id != b"PMX " {
            return Err(err_str("Not valid PMX file"));
        }
        let version = rdr.read_f32::<LE>()?;
        let num_globals = rdr.read_u8()?;
        if num_globals != 8 {
            return Err(err_str("num_globals != 8"));
        }
        let globals = Globals::from_reader(rdr)?;
        let model_name = rdr.read_nstring(globals.encoding)?;
        let model_name_en = rdr.read_nstring(globals.encoding)?;
        let comment = rdr.read_nstring(globals.encoding)?;
        let comment_en = rdr.read_nstring(globals.encoding)?;
        Ok(Header {
            magic_id,
            version,
            num_globals,
            globals,
            model_name,
            model_name_en,
            comment,
            comment_en,
        })
    }
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

impl FromReader<u8> for BoneWeight {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, index_size: u8) -> Result<BoneWeight, io::Error> {
        let bone_weight = match rdr.read_u8()? {
            0 => BoneWeight::BDEF1 { index: rdr.read_index(index_size)? },
            1 => BoneWeight::BDEF2 { indices: rdr.read_indices2(index_size)?, weight: rdr.read_f32::<LE>()? },
            2 => BoneWeight::BDEF4 { indices: rdr.read_indices4(index_size)?, weights: rdr.read_array4()? },
            3 => BoneWeight::SDEF { indices: rdr.read_indices2(index_size)?, weight: rdr.read_f32::<LE>()?, c: rdr.read_vec3()?, r0: rdr.read_vec3()?, r1: rdr.read_vec3()? },
            4 => BoneWeight::QDEF { indices: rdr.read_indices4(index_size)?, weights: rdr.read_array4()? },
            _ => return Err(err_str("unknown BoneWeight Type"))
        };
        Ok(bone_weight)
    }
}

#[derive(Debug, PartialEq)]
struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
    additional: Vec<Vec4>,
    /// We extend all index size to 4
    bone_weight: BoneWeight,
    edge_scale: f32,
}

impl<'a> FromReader<&'a Globals> for Vertex {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Vertex, io::Error> {
        let position = rdr.read_vec3()?;
        let normal = rdr.read_vec3()?;
        let uv = rdr.read_vec2()?;
        let mut additional = Vec::with_capacity(globals.additional as usize);
        for _ in 0..globals.additional {
            additional.push(rdr.read_vec4()?);
        }
        let bone_weight = BoneWeight::from_reader_arg(rdr, globals.bone_index_size)?;
        let edge_scale = rdr.read_f32::<LE>()?;
        
        Ok(Vertex {
            position,
            normal,
            uv,
            additional,
            bone_weight,
            edge_scale,
        })
    }
}

#[derive(Debug)]
struct Face(i32, i32, i32);

impl<'a> FromReader<&'a Globals> for Face {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Face, io::Error> {
        let face = match globals.vertex_index_size {
            1 => Face(rdr.read_u8()? as i32, rdr.read_u8()? as i32, rdr.read_u8()? as i32),
            2 => Face(rdr.read_u16::<LE>()? as i32, rdr.read_u16::<LE>()? as i32, rdr.read_u16::<LE>()? as i32),
            4 => Face(rdr.read_i32::<LE>()?, rdr.read_i32::<LE>()?, rdr.read_i32::<LE>()?),
            _ => return Err(err_str("Invalid vertex index size")),
        };
        Ok(face)
    }
}

#[derive(Debug)]
struct Texture(String);

impl<'a> FromReader<&'a Globals> for Texture {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Texture, io::Error> {
        let s = rdr.read_nstring(globals.encoding)?;
        Ok(Texture(s))
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

enum_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum SphereMode {
        NONE, MUL, ADD, SUB,
    }
}

enum_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum ToonMode {
        Separate, Common,
    }
}

#[derive(Debug)]
struct Material {
    name: String,
    name_en: String,
    diffuse: Vec4,
    specular: Vec3,
    intensity: f32,
    ambient: Vec3,
    draw_mode: DrawMode,
    edge_color: Vec4,
    edge_size: f32,
    texture_id: i32,
    sphere_texture_id: i32,
    sphere_mode: SphereMode,
    toon_mode: ToonMode,
    toon_texture_id: i32,
    memo: String,
    num_vertex_indices: i32,
}

impl<'a> FromReader<&'a Globals> for Material {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Material, io::Error> {
        let name = rdr.read_nstring(globals.encoding)?;
        let name_en = rdr.read_nstring(globals.encoding)?;
        let diffuse = rdr.read_vec4()?;
        let specular = rdr.read_vec3()?;
        let intensity = rdr.read_f32::<LE>()?;
        let ambient = rdr.read_vec3()?;
        let draw_mode = BitFlags::from_bits(rdr.read_u8()?).ok_or(err_str("Invalid Draw Mode"))?;
        let edge_color = rdr.read_vec4()?;
        let edge_size = rdr.read_f32::<LE>()?;
        let texture_id = rdr.read_index(globals.texture_index_size)?;
        let sphere_texture_id = rdr.read_index(globals.texture_index_size)?;
        let sphere_mode = SphereMode::from_u8(rdr.read_u8()?).ok_or(err_str("Invalid Sphere Mode"))?;
        let toon_mode = ToonMode::from_u8(rdr.read_u8()?).ok_or(err_str("Invalid Toon Mode"))?;
        let toon_texture_id = rdr.read_index(globals.texture_index_size)?;
        let memo = rdr.read_nstring(globals.encoding)?;
        let num_vertex_indices = rdr.read_i32::<LE>()?;

        Ok(Material {
            name,
            name_en,
            diffuse,
            specular,
            intensity,
            ambient,
            draw_mode,
            edge_color,
            edge_size,
            texture_id,
            sphere_texture_id,
            sphere_mode,
            toon_mode,
            toon_texture_id,
            memo,
            num_vertex_indices,
        })
    }
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

#[derive(Debug)]
struct IKLink {
    bone_id: i32,
    /// Ret: Some(min, max)
    limits: Option<(Vec3, Vec3)>,
}

impl<'a> FromReader<&'a Globals> for IKLink {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<IKLink, io::Error> {
        let bone_id = rdr.read_index(globals.bone_index_size)?;
        let enable_limit = match rdr.read_u8()? {
            1 => true,
            0 => false,
            _ => return Err(err_str("Invalid bool value")),
        };

        let limits = if enable_limit {
            Some((rdr.read_vec3()?, rdr.read_vec3()?))
        } else {
            None
        };

        Ok(IKLink{
            bone_id,
            limits,
        })
    }
}

#[derive(Debug)]
struct Bone {
    name: String,
    name_en: String,
    position: Vec3,
    parent_id: i32,
    deform_depth: i32,
    flags: BoneMode,
    position_offset: Option<Vec3>,
    link_id: Option<i32>,
    /// Ret: Some(bone_id, weight)
    append: Option<(i32, f32)>,
    fixed_axes: Option<Vec3>,
    /// Ret: Some(rotX, rotZ)
    local_rot: Option<(Vec3, Vec3)>,
    key_value: Option<i32>,
    /// Ret: Some(bone_id, num_iterations, limit, vec![IKLink, n])
    ik: Option<(i32, i32, f32, Vec<IKLink>)>,
}

impl<'a> FromReader<&'a Globals> for Bone {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Bone, io::Error> {
        let name = rdr.read_nstring(globals.encoding)?;
        let name_en = rdr.read_nstring(globals.encoding)?;
        let position = rdr.read_vec3()?;
        let parent_id = rdr.read_index(globals.bone_index_size)?;
        let deform_depth = rdr.read_i32::<LE>()?;
        let flags = BitFlags::from_bits(rdr.read_u16::<LE>()?).ok_or(err_str("Invalid Bone Flags"))?;
        
        let (position_offset, link_id) = if flags.contains(BoneFlags::TargetMode) {
            (None, Some(rdr.read_index(globals.bone_index_size)?))
        } else {
            (Some(rdr.read_vec3()?), None)
        };

        let append = if flags.contains(BoneFlags::AppendRotate) || flags.contains(BoneFlags::AppendTranslate) {
            Some((rdr.read_index(globals.bone_index_size)?, rdr.read_f32::<LE>()?))
        } else {
            None
        };

        let fixed_axes = if flags.contains(BoneFlags::AxesFixed) {
            Some(rdr.read_vec3()?)
        } else {
            None
        };

        let local_rot = if flags.contains(BoneFlags::LocalAxes) {
            Some((rdr.read_vec3()?, rdr.read_vec3()?))
        } else {
            None
        };

        let key_value = if flags.contains(BoneFlags::DeformOuterParent) {
            Some(rdr.read_i32::<LE>()?)
        } else {
            None
        };

        let ik = if flags.contains(BoneFlags::IK) {
            let bone_id = rdr.read_index(globals.bone_index_size)?;
            let num_iter = rdr.read_i32::<LE>()?;
            let limit = rdr.read_f32::<LE>()?;

            let n = rdr.read_i32::<LE>()? as usize;
            let links = rdr.read_structs(n, globals)?;

            Some((bone_id, num_iter, limit, links))
        } else {
            None
        };

        Ok(Bone {
            name, name_en, position, parent_id,
            deform_depth, flags, position_offset,
            link_id, append, fixed_axes, local_rot,
            key_value, ik,
        })
    }
}

enum_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum MorphType {
        Group,
        Position,
        Bone,
        UV,
        AddUV1,
        AddUV2,
        AddUV3,
        AddUV4,
        Material,
        Flip,
        Impulse,
    }
}

#[derive(Debug)]
struct PositionMorph {
    vertex_id: i32,
    position: Vec3,
}

impl<'a> FromReader<&'a Globals> for PositionMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<PositionMorph, io::Error> {
        let vertex_id = rdr.read_index(globals.vertex_index_size)?;
        let position = rdr.read_vec3()?;
        Ok(PositionMorph { vertex_id, position })
    }
}

#[derive(Debug)]
struct UVMorph {
    vertex_id: i32,
    uv: Vec4,
}

impl<'a> FromReader<&'a Globals> for UVMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<UVMorph, io::Error> {
        let vertex_id = rdr.read_index(globals.vertex_index_size)?;
        let uv = rdr.read_vec4()?;
        Ok(UVMorph { vertex_id, uv })
    }
}

#[derive(Debug)]
struct BoneMorph {
    bone_id: i32,
    position: Vec3,
    quaternion: Quat,
}

impl<'a> FromReader<&'a Globals> for BoneMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<BoneMorph, io::Error> {
        let bone_id = rdr.read_index(globals.bone_index_size)?;
        let position = rdr.read_vec3()?;
        let quaternion = rdr.read_quat()?;
        Ok(BoneMorph { bone_id, position, quaternion })
    }
}

enum_from_primitive! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum OpType {
        MUL, ADD,
    }
}

#[derive(Debug)]
struct MaterialMorph {
    material_id: i32,
    op_type: OpType,
    diffuse: Vec4,
    specular: Vec3,
    intensity: f32,
    ambient: Vec3,
    edge_color: Vec4,
    edge_size: f32,
    texture_factor: Vec4,
    sphere_texture_factor: Vec4,
    toon_texture_factor: Vec4,
}

impl<'a> FromReader<&'a Globals> for MaterialMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<MaterialMorph, io::Error> {
        let material_id = rdr.read_index(globals.material_index_size)?;
        let op_type = OpType::from_u8(rdr.read_u8()?).ok_or(err_str("Unkown Material Op type"))?;
        let diffuse = rdr.read_vec4()?;
        let specular = rdr.read_vec3()?;
        let intensity = rdr.read_f32::<LE>()?;
        let ambient = rdr.read_vec3()?;
        let edge_color = rdr.read_vec4()?;
        let edge_size = rdr.read_f32::<LE>()?;
        let texture_factor = rdr.read_vec4()?;
        let sphere_texture_factor = rdr.read_vec4()?;
        let toon_texture_factor = rdr.read_vec4()?;

        Ok(MaterialMorph {
            material_id,
            op_type,
            diffuse,
            specular,
            intensity,
            ambient,
            edge_color,
            edge_size,
            texture_factor,
            sphere_texture_factor,
            toon_texture_factor,
        })
    }
}

#[derive(Debug)]
struct GroupMorph {
    morph_id: i32,
    weight: f32,
}

impl<'a> FromReader<&'a Globals> for GroupMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<GroupMorph, io::Error> {
        let morph_id = rdr.read_index(globals.morph_index_size)?;
        let weight = rdr.read_f32::<LE>()?;
        Ok(GroupMorph { morph_id, weight })
    }
}

#[derive(Debug)]
struct FlipMorph {
    morph_id: i32,
    weight: f32,
}

impl<'a> FromReader<&'a Globals> for FlipMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<FlipMorph, io::Error> {
        let morph_id = rdr.read_index(globals.morph_index_size)?;
        let weight = rdr.read_f32::<LE>()?;
        Ok(FlipMorph { morph_id, weight })
    }
}

#[derive(Debug)]
struct ImpulseMorph {
    rigidbody_id: i32,
    /// 0: OFF, 1: ON
    local_flag: u8,
    translate_velocity: Vec3, // Force?
    rotate_torque: Vec3, // Torque?
}

impl<'a> FromReader<&'a Globals> for ImpulseMorph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<ImpulseMorph, io::Error> {
        let rigidbody_id = rdr.read_index(globals.rigidbody_index_size)?;
        let local_flag = rdr.read_u8()?;
        let translate_velocity = rdr.read_vec3()?;
        let rotate_torque = rdr.read_vec3()?;
        Ok(ImpulseMorph {
            rigidbody_id, local_flag,
            translate_velocity,  rotate_torque,
        })
    }
}

#[derive(Debug)]
struct Morph {
    name: String,
    name_en: String,
    /// 0: reserved, 1: eyebrow, 2: eye, 3: mouth, 4: others
    control_panel: u8,
    morph_type: MorphType,
    position_v: Vec<PositionMorph>,
    uv_v: Vec<UVMorph>,
    bone_v: Vec<BoneMorph>,
    material_v: Vec<MaterialMorph>,
    group_v: Vec<GroupMorph>,
    flip_v: Vec<FlipMorph>,
    impulse_v: Vec<ImpulseMorph>,
}

impl<'a> FromReader<&'a Globals> for Morph {
    fn from_reader_arg<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Morph, io::Error> {
        let name = rdr.read_nstring(globals.encoding)?;
        let name_en = rdr.read_nstring(globals.encoding)?;
        let control_panel = rdr.read_u8()?;
        let morph_type = MorphType::from_u8(rdr.read_u8()?).ok_or(err_str("Unkown morph type"))?;
        let n = rdr.read_i32::<LE>()? as usize;

        let position_v = if morph_type == MorphType::Position {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        let uv_v = if morph_type == MorphType::UV ||
                morph_type == MorphType::AddUV1 ||
                morph_type == MorphType::AddUV2 ||
                morph_type == MorphType::AddUV3 ||
                morph_type == MorphType::AddUV4 {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        let bone_v = if morph_type == MorphType::Bone {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        let material_v = if morph_type == MorphType::Material {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        let group_v = if morph_type == MorphType::Group {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        let flip_v = if morph_type == MorphType::Flip {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };
        
        let impulse_v = if morph_type == MorphType::Impulse {
            rdr.read_structs(n, globals)?
        } else { Vec::new() };

        Ok(Morph {            
            name,
            name_en,
            control_panel,
            morph_type,
            position_v,
            uv_v,
            bone_v,
            material_v,
            group_v,
            flip_v,
            impulse_v,
        })
    }
}

#[derive(Debug)]
struct Reader<R> {
    rdr: BufReader<R>,
}

impl<R: io::Read> Reader<R> {
    pub fn new(inner: R) -> Reader<R> {
        let rdr = BufReader::new(inner);
        Reader { rdr }
    }

    fn read(mut self) -> Result<PmxFile, io::Error> {
        Ok(PmxFile::from_reader(&mut self.rdr)?)
    }
}

pub fn from_file<P: AsRef<Path>>(path: P) -> Result<PmxFile, io::Error> {
    use std::fs::File;

    let f = File::open(path)?;
    let rdr = Reader::new(f);
    let pmx_file = rdr.read()?;
    Ok(pmx_file)
}
