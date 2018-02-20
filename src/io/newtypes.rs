use cgmath::{Vector2, Vector3, Vector4};
use byteorder::{ByteOrder, LE};
use pod_io::{Decode, Nil};
use std::io::{Read, Result};

#[derive(Debug)]
pub struct Vec2(Vector2<f32>);
#[derive(Debug)]
pub struct Vec3(Vector3<f32>);
#[derive(Debug)]
pub struct Vec4(Vector4<f32>);

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
