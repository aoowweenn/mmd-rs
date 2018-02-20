pub mod pmx;
mod newtypes;

use self::pmx::PmxFile;

use std::path::Path;
use std::io::{Read, Result};
use std::marker::Sized;

trait Load {
    fn load<R: Read>(rdr: &mut R) -> Result<Self> where Self: Sized;
}

trait FromFile {
    fn _from_file<P: AsRef<Path>>(path: P) -> Result<Self> where Self: Sized + Load {
        use std::fs::File;
        use std::io::BufReader;
        let f = File::open(path)?;
        let mut rdr = BufReader::new(f);
        Ok(Self::load(&mut rdr)?)
    }
}

impl FromFile for PmxFile {}
impl PmxFile {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::_from_file(path)
    }
}
