use nom::{IResult, le_f32};
use types::{Vec2, Vec3, Vec4};

pub trait Parse<VEC=Self> {
    const COUNT: usize;
    type ARRAY;
    
    fn parse(input: &[u8]) -> IResult<&[u8], VEC> where VEC: From<Self::ARRAY> {
        Self::nom_macro(input).map(|o| VEC::from(o))
    }

    fn nom_macro(input: &[u8]) -> IResult<&[u8], Self::ARRAY>;
}

impl Parse for Vec2 {
    const COUNT: usize = 2;
    type ARRAY = [f32; 2];

    fn nom_macro(input: &[u8]) -> IResult<&[u8], Self::ARRAY> {
        count_fixed!(input, f32, le_f32, Self::COUNT)
    }
}

impl Parse for Vec3 {
    const COUNT: usize = 3;
    type ARRAY = [f32; 3];

    fn nom_macro(input: &[u8]) -> IResult<&[u8], Self::ARRAY> {
        count_fixed!(input, f32, le_f32, Self::COUNT)
    }
}

impl Parse for Vec4 {
    const COUNT: usize = 4;
    type ARRAY = [f32; 4];

    fn nom_macro(input: &[u8]) -> IResult<&[u8], Self::ARRAY> {
        count_fixed!(input, f32, le_f32, Self::COUNT)
    }
}

#[cfg(test)]
#[test]
fn test_vec() {
    use std::mem::transmute;
    let arr = [1.1f32, 2.2f32, 3.3f32, 4.4f32];
    let v2 = [arr[0], arr[1]];
    let v3 = [arr[0], arr[1], arr[2]];
    let v4 = arr;
    let raw: [u8; 16] = unsafe {transmute(arr)};
    macro_rules! test {
        ( $( ($ty:ty, $id:ident, $n:expr) ),* ) => {{
            $(
            let r = <$ty>::parse(&raw[0..$n*4]);
            assert_eq!(r, IResult::Done(&b""[..], <$ty>::from($id)));
            )*
        }};
    }
    test![(Vec2, v2, 2), (Vec3, v3, 3), (Vec4, v4, 4)];
}
