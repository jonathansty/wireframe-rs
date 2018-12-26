pub mod opengl;
use std::cell::RefCell;
use std::sync::Arc;
use crate::pipeline::Pipeline;
use crate::helpers;


#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum PrimitiveTopology {
    Triangles,
    Lines,
}

impl PrimitiveTopology {
    fn to_gl_enum(top : &Self) -> gl::types::GLenum {
        match top {
            PrimitiveTopology::Triangles => gl::TRIANGLES,
            PrimitiveTopology::Lines => gl::LINES,
        }
    }
}

#[derive(Clone)]
pub struct DeviceHandle {
    inner : Arc<RefCell<dyn Device>>,
}
use std::ops::Deref;
impl Deref for DeviceHandle {
    type Target=Arc<RefCell<dyn Device>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

type BufferHandle = gl::types::GLuint;

type PipelineHandle = Arc<Pipeline>;
// type BufferHandle = Arc<Buffer>;
#[allow(dead_code)]
pub enum IndexType {
    UnsignedShort,
    UnsignedInt,
}
pub trait CommandList {
    fn execute(&self, device : &DeviceHandle);
    fn clear(&mut self, clear_color : [f32; 4], depth : Option<f32>);

    fn draw(&mut self, vertex_count : u32, instance_count : u32, first_vertex : u32, first_instance : u32);
    fn draw_indexed(&mut self, index_count : u32, instance_count : u32, first_index : u32, vertex_offset : u32, first_instance : u32);

    fn bind_pipeline(&mut self, pipeline: &PipelineHandle);
    fn bind_vertex_buffers(&mut self, first_binding: u32, binding_count : u32,  buffers: &[BufferHandle], offsets: &[u32]);
    fn bind_index_buffer(&mut self, buffer : &BufferHandle, offset : u32, index_type : IndexType);
}

pub trait Device {
    // Each device has it's own type of handle
    // eg: OpenGL has Rc as it does not support any multithreading anyways
    fn supports_multithreading(&self) -> bool;

    fn enable_debug_layer(&self) -> Result< (), String>;

    fn create_command_list(&self) -> Box<dyn CommandList>;
}

pub fn create_default_device(window : &sdl2::video::Window) -> DeviceHandle {
    let device_handle = Arc::new(RefCell::new(opengl::GLDevice::for_sdl2(window).unwrap()));
    DeviceHandle{
        inner: device_handle
    }
}
