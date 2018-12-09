/// This app demonstrates different ways of rendering wireframes
///
extern crate sdl2;
extern crate gl;

use gl::types::*;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4,5);

    let window = video_subsystem
        .window("Wireframe Techniques", 800, 600)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    let gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Build some shaders
    let mut default_program = 0;
    {
        use std::ffi::CString;
        let vertex_source = CString::new(include_str!("default.vert")).unwrap();
        let fragment_source = CString::new(include_str!("default.frag")).unwrap();
        let geom_source = CString::new(include_str!("default.geom")).unwrap();

        let vert = match shaders::shader_from_source(&vertex_source, gl::VERTEX_SHADER) {
            Ok(shader) => shader,
            Err(msg) => {
                panic!("Failed to compile shader with following log:\n {}", msg);
            }
        };

        let frag = match shaders::shader_from_source(&fragment_source, gl::FRAGMENT_SHADER) {
            Ok(e) => e,
            Err(e) =>{ 
                panic!("Failed to compile shader with following log:\n {}", e);
            }
        };

        let geom = match shaders::shader_from_source(&geom_source, gl::GEOMETRY_SHADER) {
            Ok(e) => e,
            Err(e) =>{ 
                panic!("Failed to compile shader with following log:\n {}", e);
            }
        };

        let mut success = 1;
        unsafe{
            default_program = gl::CreateProgram();
            gl::AttachShader(default_program,vert );
            gl::AttachShader(default_program,geom );
            gl::AttachShader(default_program,frag );
            gl::LinkProgram(default_program);

            gl::GetProgramiv(default_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(default_program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));

                let error = unsafe{ CString::from_vec_unchecked(buffer)};
                gl::GetProgramInfoLog(
                    default_program,len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar
                );
            }

        }
    }

    let mut event_pump = sdl.event_pump().unwrap();
    unsafe{
        gl::ClearColor(0.3,0.3,0.3,1.0);
    }
    'app: loop {
        for e in event_pump.poll_iter() {
           match e {
                sdl2::event::Event::Quit {..} => { break 'app; }
                _ => {}
            }
        }

        let size = window.size();
        unsafe{
            gl::Viewport(0,0,size.0 as i32, size.1 as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(default_program);
        }

        window.gl_swap_window();
    }
}

mod shaders{
    use std::ffi::{CString, CStr};

    pub fn shader_from_source(source : &CString, kind: gl::types::GLuint) -> Result<gl::types::GLuint, String> {
        let id = unsafe{ gl::CreateShader(kind) };
        
        let mut success : gl::types::GLint = 1;
        unsafe{
            gl::ShaderSource(id, 1, &(source.as_ptr() as *const i8), std::ptr::null());
            gl::CompileShader(id);

            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len = 0;
            unsafe{
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let mut buffer = Vec::with_capacity(len as usize + 1);
            buffer.extend([b' '].iter().cycle().take(len as usize));

            let error = unsafe{ CString::from_vec_unchecked(buffer)};
            unsafe{
                gl::GetShaderInfoLog(
                    id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar
                );
            }
            return Err(error.to_string_lossy().into_owned());
        }
        Ok(id)
    }
}
