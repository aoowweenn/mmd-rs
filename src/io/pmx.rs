use num::FromPrimitive;
use enumflags::*;

use std::io;
use std::io::BufReader;
use std::path::Path;

use byteorder::{ReadBytesExt, LE};

use encoding::all::UTF_16LE;
use encoding::DecoderTrap;
use encoding::Encoding;

use cgmath::{Vector2, Vector3, Vector4};

/*
#[derive(Debug)]
enum PmxError {
    Io(io::Error),
}
*/

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type DrawMode = BitFlags<DrawModeFlags>;

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

    /// TODO: handle String Error
    fn read_string(&mut self, n: usize) -> Result<String, io::Error> {
        unsafe { Ok(String::from_utf8_unchecked(self.read_vec(n)?)) }
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
}

impl<T: ReadBytesExt> ReaderExt for T {}

#[derive(Debug)]
pub struct PmxFile {
    header: Header,
    vertices: Vec<Vertex>,
    faces: Vec<Face>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
}

impl PmxFile {
    fn from_reader<R: io::Read>(rdr: &mut R) -> Result<PmxFile, io::Error> {
        let header = Header::from_reader(rdr)?;

        let num_vertices = rdr.read_u32::<LE>()? as usize;
        let mut vertices = Vec::with_capacity(num_vertices);
        for _ in 0..num_vertices {
            vertices.push(Vertex::from_reader(rdr, &header.globals)?);
        }

        let num_face_indices = rdr.read_u32::<LE>()? as usize;
        let num_faces = num_face_indices / 3;
        let mut faces = Vec::with_capacity(num_faces);
        for _ in 0..num_faces {
            faces.push(Face::from_reader(rdr, &header.globals)?);
        }

        let num_textures = rdr.read_u32::<LE>()? as usize;
        let mut textures = Vec::with_capacity(num_textures);
        for _ in 0..num_textures {
            textures.push(Texture::from_reader(rdr, &header.globals)?);
        }
        
        let num_materials = rdr.read_u32::<LE>()? as usize;
        let mut materials = Vec::with_capacity(num_materials);
        for _ in 0..num_materials {
            materials.push(Material::from_reader(rdr, &header.globals)?);
        }

        Ok(PmxFile { header, vertices, faces, textures, materials })
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

impl Globals {
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
    magic_id: String,
    version: f32,
    num_globals: u8,
    globals: Globals,
    model_name: String,
    model_name_en: String,
    comment: String,
    comment_en: String,
}

impl Header {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt) -> Result<Header, io::Error> {
        let magic_id = rdr.read_string(4)?;
        if magic_id != "PMX " {
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

impl BoneWeight {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt, index_size: u8) -> Result<BoneWeight, io::Error> {
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
    pos: Vec3,
    normal: Vec3,
    uv: Vec2,
    additional: Vec<Vec4>,
    /// We extend all index size to 4
    bone_weight: BoneWeight,
    edge_scale: f32,
}

impl Vertex {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Vertex, io::Error> {
        let pos = rdr.read_vec3()?;
        let normal = rdr.read_vec3()?;
        let uv = rdr.read_vec2()?;
        let mut additional = Vec::with_capacity(globals.additional as usize);
        for _ in 0..globals.additional {
            additional.push(rdr.read_vec4()?);
        }
        let bone_weight = BoneWeight::from_reader(rdr, globals.bone_index_size)?;
        let edge_scale = rdr.read_f32::<LE>()?;
        
        Ok(Vertex {
            pos,
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

impl Face {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Face, io::Error> {
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

impl Texture {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Texture, io::Error> {
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

impl Material {
    fn from_reader<RExt: ReaderExt>(rdr: &mut RExt, globals: &Globals) -> Result<Material, io::Error> {
        let name = rdr.read_nstring(globals.encoding)?;
        let name_en = rdr.read_nstring(globals.encoding)?;
        let diffuse = rdr.read_vec4()?;
        let specular = rdr.read_vec3()?;
        let intensity = rdr.read_f32::<LE>()?;
        let ambient = rdr.read_vec3()?;
        // TODO: check if enumflags crate has been fixed
        //let draw_mode = BitFlags::from_bits(rdr.read_u8()?).ok_or(err_str("Invalid Draw Mode"))?;
        let draw_mode = BitFlags::from_bits_truncate(rdr.read_u8()?);
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
