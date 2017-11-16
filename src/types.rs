use std::ops::{Deref, Shl};

//use cgmath::prelude::*;
use cgmath::{Vector2, Vector3, Vector4};
use encoding::{EncoderTrap, Encoding};
use encoding::all::UTF_16LE;
use byteorder::{LittleEndian, WriteBytesExt};

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;

pub struct PmxString {
    s: String,
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

impl From<PmxString> for Vec<u8> {
    fn from(ps: PmxString) -> Self {
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

/*
impl Shl<&'static str> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: &str) -> Self::Output {
        let DataBlock(mut v) = self;
        v.extend_from_slice(rhs.as_bytes());
        DataBlock(v)
    }
}
*/

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

impl Shl<PmxString> for DataBlock {
    type Output = Self;

    fn shl(self, rhs: PmxString) -> Self::Output {
        let DataBlock(mut v) = self;
        v.append(&mut rhs.into());
        DataBlock(v)
    }
}
