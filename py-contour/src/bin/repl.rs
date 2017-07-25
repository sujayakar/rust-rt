#![feature(offset_to)]
extern crate contour;
#[macro_use] extern crate contour_derive;
extern crate cpython;
extern crate py_contour;
extern crate rustyline;

use std::any::TypeId;

use contour::{
    Chartable,
    Contour,
    ContourMap,
    HasContour,
    StructField,
};
use py_contour::PythonManager;
use cpython::{
    Python,
    PyDict,
};

#[derive(Chartable, HasContour)]
struct TestStruct {
    a: usize,
    b: usize,
    c: bool,
}

fn main() {
    let manager = PythonManager::new();
    let s = TestStruct {a: 0, b: 24, c: false};
    let type_id = TypeId::of::<TestStruct>();
    TestStruct::chart(&manager);
    let gil = Python::acquire_gil();
    let py = gil.python();

    println!("ContourPython v.0.0.1");
    let mut rl = rustyline::Editor::<()>::new();
    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                // Regenerate the environment each iteration.
                let obj = manager.analyze(
                    py,
                    type_id,
                    &s as *const _ as *const u8,
                    manager.generation(),
                );
                let env = PyDict::new(py);
                env.set_item(py, "test", obj).unwrap();
                if let Err(e) = py.run(&line, None, Some(&env)) {
                    println!("Hit exception: {:?}", e);
                }
                manager.increment_generation();
            },
            Err(_) => {
                break;
            },
        }
    }
}
