extern crate base64;
extern crate rand;
extern crate serde_json;
// extern crate wasm_glue;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate enum_primitive;

use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

extern "C" {
    fn js_message(mtype: *mut c_char, message: *mut c_char);
    fn rand() -> u32;
}

mod buffer;
mod frame;
mod instruction;
mod options;
mod quetzal;
mod traits;
mod ui_web;
mod zmachine;

use options::Options;
use traits::UI;
use ui_web::WebUI;
use zmachine::Zmachine;

// thread local mutable global
thread_local!(static ZVM: RefCell<Option<Zmachine>> = RefCell::new(None););

#[no_mangle]
pub fn hook() {
    // wasm_glue::hook();
}

#[no_mangle]
pub fn allocate(length: usize) -> *mut c_void {
    let mut v = Vec::with_capacity(length);
    let ptr = v.as_mut_ptr();
    std::mem::forget(v);
    ptr
}

#[no_mangle]
pub fn deallocate(ptr: *mut c_void, length: usize) {
    unsafe {
        std::mem::drop(Vec::from_raw_parts(ptr, 0, length));
    }
}

fn get_string(ptr: *mut c_char) -> String {
    let data = unsafe { CStr::from_ptr(ptr) };

    data.to_string_lossy().into_owned()
}

fn with<F, R>(func: F) -> R
where
    F: FnOnce(&mut Zmachine) -> R,
{
    ZVM.with(|cell| {
        let mut wrapper = cell.borrow_mut();
        let zvm: &mut Zmachine = wrapper.as_mut().expect(
            "Error unwrapping zmachine from cell"
        );

        func(zvm)
    })
}

fn push_updates(zvm: &mut Zmachine) {
    let map = serde_json::to_string(&zvm.get_current_room()).unwrap();
    let tree = serde_json::to_string(&zvm.get_object_tree()).unwrap();

    zvm.update_status_bar();
    zvm.ui.message("map", &map);
    zvm.ui.message("tree", &tree);

    if zvm.options.log_instructions {
        zvm.ui.message("instructions", &zvm.instr_log);
        zvm.instr_log.clear();
    }
}

#[no_mangle]
pub fn create(file_ptr: *mut u8, len: usize) {
    ZVM.with(|cell| {
        assert!(!file_ptr.is_null());

        let data = unsafe { std::vec::Vec::from_raw_parts(file_ptr, len, len) };
        let ui = WebUI::new();
        let mut opts = Options::default();
        opts.rand_seed = unsafe { [rand(), rand(), rand(), rand()] };

        let zvm = Zmachine::new(data, ui, opts);
        *cell.borrow_mut() = Some(zvm);
    });
}

#[no_mangle]
pub fn step() -> bool {
    with(|zvm| {
        let done = zvm.step();

        zvm.ui.flush();
        push_updates(zvm);
        done
    })
}

#[no_mangle]
pub fn feed(input_ptr: *mut c_char) {
    with(|zvm| zvm.handle_input(get_string(input_ptr)));
}

#[no_mangle]
pub fn restore(b64_ptr: *mut c_char) {
    with(|zvm| zvm.restore(&get_string(b64_ptr)));
}

#[no_mangle]
pub fn load_savestate(b64_ptr: *mut c_char) {
    with(|zvm| zvm.load_savestate(&get_string(b64_ptr)));
}

#[no_mangle]
pub fn get_updates() {
    with(|zvm| push_updates(zvm));
}

#[no_mangle]
pub fn undo() -> bool {
    with(|zvm| zvm.undo())
}

#[no_mangle]
pub fn redo() -> bool {
    with(|zvm| zvm.redo())
}

#[no_mangle]
pub fn enable_instruction_logs(enabled: bool) {
    with(|zvm| zvm.options.log_instructions = enabled);
}

#[no_mangle]
pub fn get_object_details(obj_num: u16) -> Box<String> {
    with(|zvm| Box::new(zvm.debug_object_details(obj_num as u16)))
}

#[no_mangle]
pub fn flush_log() {
    ZVM.with(|cell| {
        let ptr = cell.as_ptr();
        let opt: &Option<Zmachine> = unsafe { &mut *ptr };
        let zvm = opt.as_ref().unwrap();

        zvm.ui.message("instructions", &zvm.instr_log);
    });
}
