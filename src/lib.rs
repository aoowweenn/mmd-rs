#[macro_use]
extern crate nom;
extern crate byteorder;
extern crate encoding;

pub mod pmd;
pub mod pmx;
pub mod vmd;

use nom::IResult::*;
use nom::HexDisplay;

enum State {
    Header,
    Images,
    Ended,
}

pub struct Decoder<'a> {
    data: &'a [u8],
    position: usize,
    state: State,
}

impl<'a> Decoder<'a> {
    pub fn init(d: &'a [u8]) -> Option<Decoder<'a>> {
        match d {
            _ => {
                Some(Decoder {
                    data: d,
                    position: 123,
                    state: State::Header,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pmx() {
        let asset = include_bytes!("../asset/江風ver1.05.pmx");
    }
}
