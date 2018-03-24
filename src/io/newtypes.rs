use cgmath::{Vector2, Vector3, Vector4};
use byteorder::{ByteOrder, LE};
use pod_io::{Decode, Nil};
use enumflags::*;
use std::io::{Read, Result};

#[derive(Debug)]
pub struct Vec2(pub Vector2<f32>);
#[derive(Debug)]
pub struct Vec3(pub Vector3<f32>);
#[derive(Debug)]
pub struct Vec4(pub Vector4<f32>);

impl<R: Read> Decode<R, Nil> for Vec2 {
    fn decode<B: ByteOrder>(r: &mut R, p: Nil) -> Result<Vec2> {
        Ok(Vec2(Vector2::from(<[f32; 2]>::decode::<LE>(r, p)?)))
    }
}

impl<R: Read> Decode<R, Nil> for Vec3 {
    fn decode<B: ByteOrder>(r: &mut R, p: Nil) -> Result<Vec3> {
        Ok(Vec3(Vector3::from(<[f32; 3]>::decode::<LE>(r, p)?)))
    }
}

impl<R: Read> Decode<R, Nil> for Vec4 {
    fn decode<B: ByteOrder>(r: &mut R, p: Nil) -> Result<Vec4> {
        Ok(Vec4(Vector4::from(<[f32; 4]>::decode::<LE>(r, p)?)))
    }
}

#[derive(Debug)]
pub struct Array<T>(pub Vec<T>);

impl<'a, R: Read, P, T: Decode<R, &'a P> + BigStruct> Decode<R, &'a P> for Array<T> {
    fn decode<B: ByteOrder>(r: &mut R, p: &'a P) -> Result<Array<T>> {
        let n = u32::decode::<LE>(r, Nil)? as usize;
        println!("{}", n);
        let mut buf = Vec::with_capacity(n);
        for _ in 0..n {
            buf.push(T::decode::<LE>(r, p)?);
        }
        Ok(Array(buf))
    }
}

impl<R: Read> Decode<R, usize> for Array<Vec4> {
    fn decode<B: ByteOrder>(r: &mut R, n: usize) -> Result<Array<Vec4>> {
        let mut buf = Vec::with_capacity(n);
        for _ in 0..n {
            buf.push(Vec4::decode::<LE>(r, Nil)?);
        }
        Ok(Array(buf))
    }
}

pub trait BigStruct {}

#[derive(Debug)]
pub struct ModeSet<T: RawBitFlags + BitFlagsFmt>(pub BitFlags<T>);

impl<T: RawBitFlags + BitFlagsFmt> ::std::ops::Deref for ModeSet<T> {
    type Target = BitFlags<T>;
    fn deref(&self) -> &BitFlags<T> {
        &self.0
    }
}

macro_rules! impl_decode_modeset {
    ($ty:ty, $repr:ty) => (
        impl<R: Read> Decode<R, Nil> for ModeSet<$ty> {
            fn decode<B: ByteOrder>(r: &mut R, _p: Nil) -> Result<ModeSet<$ty>> {
                BitFlags::from_bits(<$repr>::decode::<LE>(r, Nil)?)
                    .map(|x| ModeSet(x))
                    .ok_or(err(concat!("Invalid ", stringify!($ty), " ModeSet")))
            }
        }
    )
}

macro_rules! impl_decode_mode {
    ($ty:ty, $repr:ty) => (
        impl<R: Read> Decode<R, Nil> for $ty {
            fn decode<B: ByteOrder>(r: &mut R, _p: Nil) -> Result<$ty> {
                <$ty>::from_u8(<$repr>::decode::<LE>(r, Nil)?).ok_or(err(concat!("Invalid ", stringify!($ty), " Mode")))
            }
        }
    )
}
