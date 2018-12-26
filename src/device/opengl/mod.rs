use super::*;

use std::rc::Rc;
use gl::types::*;

use crate::pipeline::Pipeline;
use crate::device::PrimitiveTopology;

type GLError = GLint;
#[allow(dead_code)]
pub struct GLDevice{
    gl_context : sdl2::video::GLContext,

    active_pipeline : Option<Rc<Pipeline>>,
}

impl GLDevice {
    pub fn for_sdl2(window : &sdl2::video::Window) -> Result<GLDevice, GLError> {
        let gl_context = window.gl_create_context().unwrap();
        let video_subsytem = window.subsystem();

        gl::load_with(|s| video_subsytem.gl_get_proc_address(s) as *const std::os::raw::c_void);
        Ok(
            GLDevice{
                gl_context,
                active_pipeline: None,
            }
        )
    }

    fn bind_pipeline_internal(&mut self, pipeline : &Rc<Pipeline>) {
        // Set the program as used
        unsafe{
            gl::UseProgram(pipeline.program());
        }

        // Flush all the uniforms currently set on this pipeline
        pipeline.flush();

        // Do the pipeline it's state
        helpers::gl_set_enabled(gl::DEPTH_TEST,pipeline.depth_test());
        helpers::gl_set_enabled(gl::BLEND,pipeline.depth_test());

    }
}


impl Device for GLDevice {
    /// OpenGL does not support using the context from a different thread
    fn supports_multithreading(&self) -> bool {
        false
    }

    fn create_command_list(&self) -> Box<dyn CommandList> {
        Box::new(
            GLCommandList{
                commands: Vec::new(),
                active_pipeline: std::ptr::null()
            }
        )
    }

    /// Enabled the debug output and binds callbacks
    fn enable_debug_layer(&self) -> Result<(), String > {
        unsafe{
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::DebugMessageCallback(callback, std::ptr::null());
        }
        Ok( () )
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

unsafe trait GLCommand {
    unsafe fn execute(&self);
} 

struct GLCommandList {
   commands : Vec<Box<dyn GLCommand>>, 

   // Unsafe pointer
   active_pipeline : *const Pipeline,
}

impl CommandList for GLCommandList {
    fn execute(&self, device : &DeviceHandle) {
        for c in &self.commands {
            unsafe {
                c.execute();
            }
        }
    }

    fn clear(&mut self) {
        self.commands.clear();
    }

    fn draw(&mut self, vertex_count : u32, instance_count : u32, first_vertex : u32, first_instance : u32){
        debug_assert!(self.active_pipeline != std::ptr::null());

        struct DrawCommand {
            vertex_count : u32,
            instance_count : u32,
            first_vertex : u32,
            first_instance : u32,
            topology : GLenum,
        }; 

        unsafe impl GLCommand for DrawCommand {
            unsafe fn execute(&self) {
                // #TODO : Implement support for draw modes taken from the currently bound pipelines?
                gl::DrawArrays(self.topology, self.first_vertex as GLsizei,self.vertex_count as GLsizei);
            }
        }

        let topo = unsafe {
            PrimitiveTopology::to_gl_enum(&(*self.active_pipeline).primitive_topology())
        };

        self.commands.push(Box::new( DrawCommand {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
            topology: topo
        }));
    }
    fn draw_indexed(&mut self, index_count : u32, instance_count : u32, first_index : u32, vertex_offset : u32, first_instance : u32) {
        struct Command {
            index_count : u32,
            instance_count : u32,
            first_index : u32,
            vertex_offset : u32,
            first_instance : u32,
            topology: GLenum,
        }; 

        unsafe impl GLCommand for Command {
            unsafe fn execute(&self) {
                // #TODO implement support for other draw modes...
                gl::DrawElements(self.topology, self.index_count as GLsizei, gl::UNSIGNED_INT, std::ptr::null());
            }
        }

        let topo = unsafe {
            PrimitiveTopology::to_gl_enum(&(*self.active_pipeline).primitive_topology())
        };

        self.commands.push(Box::new( Command {
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
            topology: topo
        }));
    }

    fn bind_pipeline(&mut self, pipeline: &PipelineHandle) {
        struct Command {
            program : GLuint,
            blend_enabled : bool,
            depth_test : bool,
        }; 

        unsafe impl GLCommand for Command {
            unsafe fn execute(&self) {

                // Do the pipeline it's state
                helpers::gl_set_enabled(gl::DEPTH_TEST,self.depth_test);
                helpers::gl_set_enabled(gl::BLEND,self.blend_enabled);

                gl::UseProgram(self.program);

                // Flush uniforms?
            }
        }


        self.active_pipeline =  Arc::into_raw(pipeline.clone());
        self.commands.push(Box::new( Command {
            program: pipeline.program(),
            blend_enabled: pipeline.blend_enabled(),
            depth_test: pipeline.depth_test()
        }));
    }
    
    fn bind_vertex_buffers(&mut self, first_binding: u32, binding_count : u32,  buffers: &[BufferHandle], offsets: &[u32]) {
        debug_assert!(buffers.len() > 0);
        struct Command {
            buffer : GLuint
        }; 

        unsafe impl GLCommand for Command {
            unsafe fn execute(&self) {
                gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer);
            }
        }

        self.commands.push(Box::new( Command {
            buffer: buffers[0]
        }));
        
    }

    fn bind_index_buffer(&mut self, buffer : &BufferHandle, offset : u32, index_type : IndexType) {

    }
}