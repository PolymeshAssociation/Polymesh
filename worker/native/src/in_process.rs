use polymesh_worker::backend::BackendModule;
#[path = "dart.rs"]
mod dart;
#[path = "testing.rs"]
mod testing;

pub fn run_child_if_requested() {}

pub fn dart_module() -> Option<Box<dyn BackendModule>> {
    Some(Box::new(dart::NativeDartModule))
}

pub fn testing_module() -> Option<Box<dyn BackendModule>> {
    Some(Box::new(testing::NativeTestingModule))
}
