extern crate byteorder;
extern crate cgmath;
extern crate encoding;
#[macro_use]
extern crate nom;

pub mod pmd;
pub mod pmx;
pub mod vmd;

mod types;
mod traits;

#[cfg(test)]
mod tests {

    #[test]
    fn parse_pmx() {}
}
