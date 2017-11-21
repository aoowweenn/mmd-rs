use std::ops::{Deref, Shl};

//use cgmath::prelude::*;
use cgmath::{Vector2, Vector3, Vector4};
use encoding::{DecoderTrap, EncoderTrap, Encoding};
use encoding::all::UTF_16LE;
use byteorder::{LittleEndian, WriteBytesExt};
use nom::{IResult, le_i32};

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;

#[derive(PartialEq, Debug)]
pub struct PmxString {
    s: String,
}

impl PmxString {
    pub fn parse(input: &[u8], encoding: u8) -> IResult<&[u8], PmxString> {
        map!(input, length_data!(le_i32), |x| {
            Self {
                s: Self::decode_text(x, encoding),
            }
        })
    }

    fn decode_text(x: &[u8], encoding: u8) -> String {
        match encoding {
            0u8 => UTF_16LE.decode(x, DecoderTrap::Strict).unwrap(),
            1u8 => String::from_utf8(x.to_vec()).unwrap(),
            _ => panic!("Unknown encoding"),
        }
    }
}

impl Deref for PmxString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.s
    }
}

impl From<&'static str> for PmxString {
    fn from(s: &'static str) -> Self {
        Self { s: s.to_owned() }
    }
}

impl<'a> From<&'a PmxString> for Vec<u8> {
    fn from(ps: &PmxString) -> Self {
        let mut s = UTF_16LE.encode(&ps.s, EncoderTrap::Strict).unwrap();
        let mut v = vec![s.len() as u8, 0u8, 0u8, 0u8];
        v.append(&mut s);
        v
    }
}

pub struct DataBlock(Vec<u8>);

impl DataBlock {
    pub fn new() -> DataBlock {
        DataBlock(Vec::<u8>::new())
    }
    pub fn unwrap(self) -> Vec<u8> {
        let DataBlock(v) = self;
        v
    }
}

impl<'a> Shl<&'a [u8]> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: &[u8]) -> Self::Output {
        let DataBlock(mut v) = self;
        v.extend_from_slice(rhs);
        DataBlock(v)
    }
}

impl Shl<u8> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: u8) -> Self::Output {
        let DataBlock(mut v) = self;
        v.push(rhs);
        DataBlock(v)
    }
}

impl Shl<i32> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: i32) -> Self::Output {
        let DataBlock(mut v) = self;
        v.write_i32::<LittleEndian>(rhs).unwrap();
        DataBlock(v)
    }
}

impl Shl<f32> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: f32) -> Self::Output {
        let DataBlock(mut v) = self;
        v.write_f32::<LittleEndian>(rhs).unwrap();
        DataBlock(v)
    }
}

impl<'a> Shl<&'a [f32]> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: &[f32]) -> Self::Output {
        let DataBlock(mut v) = self;
        rhs.into_iter()
            .for_each(|&x| v.write_f32::<LittleEndian>(x).unwrap());
        DataBlock(v)
    }
}

impl<'a> Shl<&'a PmxString> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: &PmxString) -> Self::Output {
        let DataBlock(mut v) = self;
        v.append(&mut rhs.into());
        DataBlock(v)
    }
}
