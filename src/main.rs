#![windows_subsystem="windows"]
#[macro_use] extern crate sciter;
use sciter::{ Value };
use std::thread;

static mut VECTOR: Vec<String> = vec![];

struct EventHandler {}
impl EventHandler {
    fn write(&self, text: String, style: f64, legibility: f64, speed: f64, width: f64, canvas_width: f64, canvas_height: f64, callback: Value, resolve: Value) -> () {
        thread::spawn(move || {
            fill_vector(text, style as f32, legibility as f32, speed as f32, width as f32, canvas_width as f32, canvas_height as f32);
            loop {
              if (unsafe { VECTOR.len() } > 0) {
                let first_vec_entry = unsafe { 
                  VECTOR.reverse();
                  let entry = VECTOR.pop().unwrap();
                  VECTOR.reverse();
                  entry
                };
                if first_vec_entry == String::from("DONE") {
                  resolve.call(None, &make_args!("DONE"), None).unwrap();
                  break;
                } 
                callback.call(None, &make_args!(first_vec_entry), None).unwrap();
              }
            }
        });
    }
}

impl sciter::EventHandler for EventHandler {
  fn get_subscription(&mut self) -> Option<sciter::dom::event::EVENT_GROUPS> {
		Some(sciter::dom::event::default_events() | sciter::dom::event::EVENT_GROUPS::HANDLE_METHOD_CALL)
	}
  dispatch_script_call! (
    fn write(String, f64, f64, f64, f64, f64, f64, Value, Value);
  );
}
fn main() {
    sciter::set_options(sciter::RuntimeOptions::DebugMode(false)).unwrap();
    let archived = include_bytes!("../target/assets.rc");
    sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_SYSINFO  as u8 |
        sciter::SCRIPT_RUNTIME_FEATURES::ALLOW_FILE_IO  as u8
    )).unwrap();
    let mut frame = sciter::Window::new();
    frame.event_handler(EventHandler { });
    frame.archive_handler(archived).unwrap();
    frame.load_file("this://app/main.html");
    frame.run_app();
}

extern crate libc;
extern crate libloading;
use libc::c_char;
use std::ffi::{ CStr, CString };

fn fill_vector(text: String, style: f32, legibility: f32, speed: f32, width: f32, canvas_width: f32, canvas_height: f32) -> () {
  unsafe {
    thread::spawn(move || {
      let lib = match libloading::Library::new("calligrapher-ai") {
          Ok(lib) => lib,
          Err(_) => panic!("Library not found!")
      };
      let write: libloading::Symbol<fn(*const c_char, f32, f32, f32, f32, f32, f32, fn(*const c_char)) -> ()> = lib.get(b"write").unwrap();
      write(
        string_to_ptr(text), 
        style as f32, 
        legibility as f32, 
        speed as f32, 
        width as f32, 
        canvas_width as f32,
        canvas_height as f32,
        fill_vector_callback
      );
    });
  }
}

fn fill_vector_callback(ptr: *const c_char) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(ptr) };
    let str_slice: &str = c_str.to_str().unwrap();
    let str_buf: String = str_slice.to_owned();
    unsafe { VECTOR.push(str_buf) };
}

fn string_to_ptr(string: String) -> *const c_char {
    let c_string = CString::new(string).unwrap();
    let ptr = c_string.as_ptr();
    std::mem::forget(c_string);
    return ptr;
}