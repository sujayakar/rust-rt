extern crate contour;
#[macro_use] extern crate cpython;

use std::any::{
    Any,
    TypeId,
};
use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
};

use contour::{
    Contour,
    ContourMap,
    Primitive,
};
use cpython::{
    Python,
    PyDict,
    PyObject,
    PyResult,
    PyString,
    PythonObject,
    ToPyObject,
};

struct TrustMe<T>(T);
unsafe impl<T> Send for TrustMe<T> {}

#[derive(Clone)]
pub struct PythonManager {
    inner: Arc<Mutex<Inner>>,
}

impl PythonManager {
    pub fn new() -> Self {
        let inner = Inner {map: HashMap::new(), generation: 0};
        PythonManager { inner: Arc::new(Mutex::new(inner)) }
    }

    pub fn generation(&self) -> usize {
        self.inner.lock().unwrap().generation
    }

    pub fn increment_generation(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.generation += 1;
    }

    pub fn superanalyze<T: Any>(&self, py: Python, t: &T) -> PyObject {
        self.analyze(
            py,
            TypeId::of::<T>(),
            t as *const _ as *const u8,
            self.generation(),
        )
    }


    pub fn analyze(&self,
                   py: Python,
                   type_id: TypeId,
                   ptr: *const u8,
                   gen: usize)
        -> PyObject
    {
        let inner = self.inner.lock().unwrap();
        assert_eq!(gen, inner.generation);

        let contour: Contour = inner.map.get(&type_id).unwrap().clone();

        fn to_py_object<T: Copy + ToPyObject>(py: Python, ptr: *const u8) -> PyObject {
            let val = unsafe {*(ptr as *const T)};
            val.to_py_object(py).into_object()
        }

        match contour {
            Contour::Struct {..} => {
                let obj = Struct::create_instance(
                    py,
                    contour.clone(),
                    self.clone(),
                    inner.generation,
                    TrustMe(ptr),
                ).unwrap();
                obj.into_object()
            },

            Contour::Primitive { variant: Primitive::u8, .. } =>
                to_py_object::<u8>(py, ptr),
            Contour::Primitive { variant: Primitive::u16, .. } =>
                to_py_object::<u16>(py, ptr),
            Contour::Primitive { variant: Primitive::u32, .. } =>
                to_py_object::<u32>(py, ptr),
            Contour::Primitive { variant: Primitive::u64, .. } =>
                to_py_object::<u64>(py, ptr),
            Contour::Primitive { variant: Primitive::usize, .. } =>
                to_py_object::<usize>(py, ptr),
            Contour::Primitive { variant: Primitive::i8, .. } =>
                to_py_object::<i8>(py, ptr),
            Contour::Primitive { variant: Primitive::i16, .. } =>
                to_py_object::<i16>(py, ptr),
            Contour::Primitive { variant: Primitive::i32, .. } =>
                to_py_object::<i32>(py, ptr),
            Contour::Primitive { variant: Primitive::i64, .. } =>
                to_py_object::<i64>(py, ptr),
            Contour::Primitive { variant: Primitive::f32, .. } =>
                to_py_object::<f32>(py, ptr),
            Contour::Primitive { variant: Primitive::f64, .. } =>
                to_py_object::<f64>(py, ptr),
            Contour::Primitive { variant: Primitive::isize, .. } =>
                to_py_object::<isize>(py, ptr),
            Contour::Primitive { variant: Primitive::bool, .. } =>
                to_py_object::<bool>(py, ptr),
            Contour::Primitive { variant: Primitive::char, .. } => {
                let val = unsafe {*(ptr as *const char)};
                let s = format!("{}", val);
                PyString::new(py, &s).into_object()
            },
            _ => unimplemented!(),
        }
    }
}

impl ContourMap for PythonManager {
    fn register(&self, contour: Contour) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let type_id = contour.type_id();
        if let Some(current) = inner.map.get(&type_id) {
            if current == &contour {
                return true;
            } else {
                panic!("Contour mismatch: {:?} vs. {:?}", current, contour);
            }
        }
        inner.map.insert(type_id, contour);
        false
    }
}

struct Inner {
    map: HashMap<TypeId, Contour>,
    generation: usize,
}

py_class!(class Struct |py| {
    data contour: Contour;
    data manager: PythonManager;
    data generation: usize;
    data ptr: TrustMe<*const u8>;

    def __name__(&self) -> PyResult<PyString> {
        let contour = self.contour(py);
        Ok(PyString::new(py, contour.name()))
    }

    def __repr__(&self) -> PyResult<PyString> {
        let contour = self.contour(py);
        let repr = format!("{:?}", contour);
        Ok(PyString::new(py, &repr))
    }

    def __str__(&self) -> PyResult<PyString> {
        let contour = self.contour(py);
        let s = format!("{}<..>", contour.name());
        Ok(PyString::new(py, &s))
    }

    def dict(&self) -> PyResult<PyDict> {
        let contour = self.contour(py);
        let manager = self.manager(py);
        let generation = self.generation(py);
        let ptr = self.ptr(py);

        let result = PyDict::new(py);
        let fields = match contour {
            &Contour::Struct {ref fields, ..} => fields,
            _ => unimplemented!(),
        };
        for field in fields {
            let key = PyString::new(py, &field.name);
            let subptr = unsafe {ptr.0.offset(field.offset as isize)};
            let field_obj = manager.analyze(
                py,
                field.type_id,
                subptr,
                *generation,
            );
            result.set_item(py, key, field_obj)?;
        }
        Ok(result)
    }
});
