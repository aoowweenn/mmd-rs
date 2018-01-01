use std::io;
use std::io::BufReader;
use std::path::Path;

use byteorder::{ReadBytesExt, LE};

use encoding::all::UTF_16LE;
use encoding::DecoderTrap;
use encoding::Encoding;

/*
#[derive(Debug)]
enum PmxError {
    Io(io::Error),
}
*/

trait ReaderExt: ReadBytesExt {
    fn read_vec(&mut self, n: usize) -> Result<Vec<u8>, io::Error> {
        let mut vec = Vec::with_capacity(n);
        unsafe { vec.set_len(n) }
        self.read_exact(&mut vec)?;
        Ok(vec)
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
}

impl PmxFile {
    fn from_reader<R: io::Read>(rdr: &mut R) -> Result<PmxFile, io::Error> {
        let header = Header::from_reader(rdr)?;
        Ok(PmxFile { header })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringEnc {
    UTF16,
    UTF8,
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
        let encoding = match rdr.read_u8()? {
            0u8 => StringEnc::UTF16,
            1u8 => StringEnc::UTF8,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Invalid String Encoding",
                ))
            }
        };
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
            return Err(io::Error::new(io::ErrorKind::Other, "Not valid PMX file"));
        }
        let version = rdr.read_f32::<LE>()?;
        let num_globals = rdr.read_u8()?;
        if num_globals != 8 {
            return Err(io::Error::new(io::ErrorKind::Other, "num_globals != 8"));
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
