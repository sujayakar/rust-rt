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

    unsafe extern "C" fn enum_variant(_self: *const u8) -> isize;
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
    fn test_generic() {
        println!("{:#?}", GenericTest::<u64>::contour());
    }
}
