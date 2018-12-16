use gl::types::*;
use std::ffi::c_void;
use std::ffi::CString;

pub enum PrimitiveTopology {
    Triangles,
    Lines,
}
pub trait Device {

    // pub fn bind_pipeline(...) {}
    // pub fn bind_buffer(...){}

    // pub fn draw(...){}

    // Topology is taken from the pipeline
    fn draw_indexed(&self, count : usize, index_type : i32, offset : i32){}

    fn enable_debug_layer(&self) -> Result< (), String>;

}

type GLError = i32;
/// GL Device
pub struct GLDevice{
    gl_context : sdl2::video::GLContext
}

impl GLDevice {
    pub fn for_sdl2(window : &sdl2::video::Window) -> Result<GLDevice, GLError> {
        let gl_context = window.gl_create_context().unwrap();
        let video_subsytem = window.subsystem();

        gl::load_with(|s| video_subsytem.gl_get_proc_address(s) as *const std::os::raw::c_void);
        Ok(
            GLDevice{
                gl_context
            }
        )
    }
}

#[allow(unused_assignments)]
extern "system" fn callback(
    _source: u32,
    _gltype: u32,
    _id: u32,
    severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut std::ffi::c_void,
) {
    if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
        unsafe {
            let string = std::ffi::CStr::from_ptr(message);
            println!("{}", string.to_str().unwrap());
        }
    }
}

impl Device for GLDevice {
    fn enable_debug_layer(&self) -> Result<(), String > {
        unsafe{
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(callback, std::ptr::null());
        }
        Ok( () )
    }
}
