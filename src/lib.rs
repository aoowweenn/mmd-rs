extern crate byteorder;
extern crate cgmath;
extern crate encoding;

#[macro_use]
extern crate enum_primitive_derive;
extern crate num_traits;

extern crate enumflags;
#[macro_use]
extern crate enumflags_derive;

#[macro_use]
extern crate pod_io;

/*
#[macro_use]
extern crate nom;
*/

pub mod pmd;
//pub mod pmx;
pub mod vmd;

pub mod io;

//mod types;
//mod traits;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pmx() {
        use std::path::PathBuf;

        let mut buf = PathBuf::from("asset");
        buf.push("江風ver1.05.pmx");
        assert!(buf.file_name().is_some());
        println!("{:?}", buf.to_str());
        let res = io::pmx::PmxFile::from_file(buf);
        match res {
            Ok(data) => println!("{:?}", data),
            Err(e) => {
                println!("{}", e);
                assert!(false)
            }
        }
    }
}
