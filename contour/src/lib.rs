#![cfg_attr(test, feature(offset_to))]
#![allow(non_camel_case_types)]
#[cfg(test)] #[macro_use] extern crate contour_derive;
extern crate syn;

use std::any::TypeId;

#[derive(Clone, Debug, Eq, PartialEq)]
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
    Primitive {
        name: &'static str,
        type_id: TypeId,
        size: usize,
        variant: Primitive,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Primitive {
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    isize,
    bool,
    char,
}

macro_rules! prim_impl {
    ($t:ty, $n:ident) => {
        impl HasContour for $t {
            fn contour() -> Contour {
                Contour::Primitive {
                    name: stringify!($t),
                    type_id: ::std::any::TypeId::of::<$t>(),
                    size: ::std::mem::size_of::<$t>(),
                    variant: Primitive::$n,
                }
            }
        }

        impl Chartable for $t {
            fn chart<CM: ContourMap>(map: &CM) {
                map.register(Self::contour());
            }
        }
    };
}
prim_impl!(u8, u8);
prim_impl!(u16, u16);
prim_impl!(u32, u32);
prim_impl!(u64, u64);
prim_impl!(usize, usize);
prim_impl!(i8, i8);
prim_impl!(i16, i16);
prim_impl!(i32, i32);
prim_impl!(i64, i64);
prim_impl!(f32, f32);
prim_impl!(f64, f64);
prim_impl!(isize, isize);
prim_impl!(bool, bool);
prim_impl!(char, char);

impl Contour {
    pub fn name(&self) -> &'static str {
        match *self {
            Contour::Struct {name, ..} => name,
            Contour::Tuple {name, ..} => name,
            Contour::Unit {name, ..} => name,
            Contour::Enum {name, ..} => name,
            Contour::Primitive {name, ..} => name,
        }
    }

    pub fn type_id(&self) -> TypeId {
        match *self {
            Contour::Struct {type_id, ..} => type_id,
            Contour::Tuple {type_id, ..} => type_id,
            Contour::Unit {type_id, ..} => type_id,
            Contour::Enum {type_id, ..} => type_id,
            Contour::Primitive {type_id, ..} => type_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructField {
    pub name: &'static str,
    pub type_id: TypeId,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TupleField {
    pub ix: usize,
    pub type_id: TypeId,
    pub offset: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variant {
    pub name: &'static str,
    pub fields: VariantFields,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VariantFields {
    Struct(Vec<StructField>),
    Tuple(Vec<TupleField>),
    Unit,
}
pub trait HasContour {
    fn contour() -> Contour;
}

pub trait ContourMap {
    /// Returns `true` if `type_id` exists and `contour` matches.
    /// Panics if `type_id` exists and `contour` doesn't match.
    fn register(&self, contour: Contour) -> bool;
}

pub trait Chartable: HasContour {
    /// The type is responsible for charting its descendants and *not* recursing
    /// if it's already been charted.
    fn chart<CM: ContourMap>(map: &CM);
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::cell::RefCell;

    struct SimpleMap {
        map: RefCell<HashMap<TypeId, Contour>>,
    }
    impl ContourMap for SimpleMap {
        fn register(&self, contour: Contour) -> bool {
            let mut map = self.map.borrow_mut();
            let type_id = contour.type_id();
            if let Some(current) = map.get(&type_id) {
                if current == &contour {
                    return true;
                } else {
                    panic!("Contour mismatch: {:?} vs. {:?}", current, contour);
                }
            }
            map.insert(type_id, contour);
            false
        }
    }

    #[derive(Chartable, HasContour)]
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

    #[test]
    fn test_chart() {
        let mut sm = SimpleMap {map: RefCell::new(HashMap::new())};
        StructTest::chart(&mut sm);
        assert_eq!(sm.map.len(), 4);

        #[derive(Chartable, HasContour)]
        struct A {
            b: B,
            c: C,
        }
        #[derive(Chartable, HasContour)]
        struct B {
            c: C,
        }
        #[derive(Chartable, HasContour)]
        struct C {
            d: u32,
            e: u64,
        }
        let mut sm = SimpleMap {map: HashMap::new()};
        A::chart(&mut sm);
        assert_eq!(sm.map.len(), 5);
    }
}
