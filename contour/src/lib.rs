#![cfg_attr(test, feature(offset_to))]
#[cfg(test)] #[macro_use] extern crate contour_derive;
extern crate syn;

use std::any::TypeId;

#[derive(Debug)]
pub enum Contour {
    Struct {
        name: &'static str,
        size: usize,
        type_id: TypeId,
        fields: Vec<StructField>,
    },
    Tuple {
        name: &'static str,
        size: usize,
        type_id: TypeId,
        fields: Vec<TupleField>,
    },
    Unit {
        name: &'static str,
        type_id: TypeId,
    },
    Enum {
        name: &'static str,
        size: usize,
        type_id: TypeId,
        variants: Vec<Variant>,
        tag: unsafe extern "C" fn(*const u8) -> usize,
    },
}

#[derive(Debug)]
pub struct StructField {
    pub name: &'static str,
    pub type_id: TypeId,
    pub offset: usize,
}

#[derive(Debug)]
pub struct TupleField {
    pub ix: usize,
    pub type_id: TypeId,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Variant {
    pub name: &'static str,
    pub fields: VariantFields,
}

#[derive(Debug)]
pub enum VariantFields {
    Struct(Vec<StructField>),
    Tuple(Vec<TupleField>),
    Unit,
}


pub trait HasContour {
    fn contour() -> Contour;
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    #[derive(HasContour)]
    struct StructTest {
        a: u32,
        b: u64,
        c: f64,
    }

    #[derive(HasContour)]
    struct TupleTest(u64, String, f64);

    #[derive(HasContour)]
    struct UnitTest;

    #[derive(HasContour)]
    enum EnumTest {
        A,
        B(u32, u64),
        C {
            c1: String,
            c2: f64,
        },
    }

    #[derive(HasContour)]
    enum SecondEnum {
        Really,
        Now,
        Question,
        Mark,
    }

    #[derive(HasContour)]
    struct GenericTest<A: 'static> {
        a: A,
        b: u32,
    }

    #[test]
    fn test_simple() {
        println!("{:#?}", StructTest::contour());
        println!("{:#?}", TupleTest::contour());
        println!("{:#?}", UnitTest::contour());
        println!("{:#?}", EnumTest::contour());
    }

    #[test]
    fn test_tag() {
        let (e1v, e1t) = match EnumTest::contour() {
            Contour::Enum { variants, tag, .. } => (variants, tag),
            _ => panic!("Wrong variant!"),
        };

        let e1 = EnumTest::B(0, 1);
        let e1p = &e1 as *const _ as *const u8;
        assert_eq!(unsafe {e1t(e1p)}, 1);
        let fields = match e1v[1].fields {
            VariantFields::Tuple(ref v) => v.clone(),
            _ => panic!("Really wrong variant!"),
        };
        assert_eq!(unsafe {*(e1p.offset(fields[0].offset as isize) as *const u32)},
                   0u32);
        assert_eq!(unsafe {*(e1p.offset(fields[1].offset as isize) as *const u64)},
                   1u64);

        let e2t = match SecondEnum::contour() {
            Contour::Enum { tag, .. } => tag,
            _ => panic!("Wrong variant!"),
        };
        assert_eq!(unsafe {e2t(&SecondEnum::Really as *const _ as *const u8)}, 0);
        assert_eq!(unsafe {e2t(&SecondEnum::Now as *const _ as *const u8)}, 1);
        assert_eq!(unsafe {e2t(&SecondEnum::Question as *const _ as *const u8)}, 2);
        assert_eq!(unsafe {e2t(&SecondEnum::Mark as *const _ as *const u8)}, 3);
    }

    #[test]
    fn test_generic() {
        println!("{:#?}", GenericTest::<u64>::contour());
    }
}
