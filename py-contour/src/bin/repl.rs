#![feature(offset_to)]
extern crate contour;
#[macro_use] extern crate contour_derive;
extern crate cpython;
extern crate py_contour;
extern crate rustyline;

use contour::{
    Contour,
    ContourMap,
    Introspectable,
    StructField,
};
use py_contour::PythonManager;
use cpython::{
    Python,
    PyDict,
};

#[derive(Introspectable)]
struct TestStruct {
    b: usize,
    c: bool,
    d: String,
    e: char,
}

fn main() {
    let manager = PythonManager::new();
    let s = TestStruct {b: 24, c: false, d: "oh fuck".to_owned(), e: 'f'};
    TestStruct::chart(&manager);

    let gil = Python::acquire_gil();
    let py = gil.python();

    println!("ContourPython v.0.0.1");
    let mut rl = rustyline::Editor::<()>::new();
    let env = PyDict::new(py);
    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                rl.add_history_entry(&line);

                // Regenerate the environment each iteration.
                let obj = manager.superanalyze(py, &s);
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
