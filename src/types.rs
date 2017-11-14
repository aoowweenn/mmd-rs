use std::ops::Deref;

//use cgmath::prelude::*;
use cgmath::{Vector2, Vector3, Vector4};
use encoding::{EncoderTrap, Encoding};
use encoding::all::UTF_16LE;

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
