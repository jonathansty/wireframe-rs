/// This app demonstrates different ways of rendering wireframes
///
extern crate sdl2;
extern crate gl;
extern crate assimp;

use gl::types::*;

fn main() {
    // Setup SDL2
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

    unsafe {
        #[allow(unused_assignments)]
        extern "system" fn callback(_source : u32, _gltype : u32, _id : u32, _severity : u32, _length : i32, message : *const i8, _user_param : *mut std::ffi::c_void){
            unsafe{
                let string = std::ffi::CStr::from_ptr(message);
                println!("{}", string.to_str().unwrap());
            }
        }
        gl::DebugMessageCallback(callback, std::ptr::null());
    }
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
        unsafe {
            default_program = gl::CreateProgram();
            gl::AttachShader(default_program,vert );
            // gl::AttachShader(default_program,geom );
            gl::AttachShader(default_program,frag );
            gl::LinkProgram(default_program);

            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            gl::GetProgramiv(default_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(default_program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));

                let error =  CString::from_vec_unchecked(buffer);
                gl::GetProgramInfoLog(
                    default_program,len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar
                );
            }
        }
    }

    // Load our mesh and setup buffers
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    {
        use assimp::Importer;
        let importer = Importer::new();
        let scene = importer.read_file("assets/suzanne.fbx").unwrap();
        println!("Loaded scene with {} meshes", scene.num_meshes());

        let first_mesh = scene.mesh(0).unwrap();
        println!("{} vertices, {} faces", first_mesh.num_vertices(), first_mesh.num_faces());

        for i in 0..first_mesh.num_vertices() {
            let pos = first_mesh.get_vertex(i).unwrap();
            // let col = first_mesh.get_vertex_color(i).unwrap();

            vertices.push(GlVert{
               pos: [pos.x,pos.y,pos.y,1.0],
               uv: [0.0,0.0],
            });
        }
        for face in first_mesh.face_iter() {
            // Triangulate?
            if face.num_indices == 4 {
                let i0 = face[0];
                let i1 = face[1];
                let i2 = face[2];
                let i3 = face[3];
                indices.push(i0);
                indices.push(i1);
                indices.push(i2);

                indices.push(i0);
                indices.push(i2);
                indices.push(i3);
            } else 
            {
                let i0 = face[0];
                let i1 = face[1];
                let i2 = face[2];
                indices.push(i0);
                indices.push(i1);
                indices.push(i2);

            }
        }
    }

    // Construct our setup
    let mut vtx_buffer = 0;
    let mut idx_buffer = 0;
    unsafe{
        gl::GenBuffers(1, &mut vtx_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer);
        let buffer_size = vertices.len() * std::mem::size_of::<GlVert>();
        gl::BufferData(gl::ARRAY_BUFFER, buffer_size as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

        gl::GenBuffers(1, &mut idx_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer);
        let buffer_size = indices.len() * std::mem::size_of::<u32>();
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, buffer_size as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);
    }



    let mut event_pump = sdl.event_pump().unwrap();
    unsafe{
        gl::ClearColor(0.3,0.3,0.3,1.0);
    }

    unsafe{
        gl::CullFace(gl::BACK);
    }
    // Run the application
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

            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::EnableVertexArrayAttrib(vao, 0);
            gl::VertexAttribPointer(0, 4, gl::FLOAT,gl::FALSE, std::mem::size_of::<GlVert>() as i32, std::ptr::null());

            gl::EnableVertexArrayAttrib(vao, 1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT,gl::FALSE, std::mem::size_of::<GlVert>() as i32, (4*std::mem::size_of::<f32>()) as *const std::ffi::c_void);

            gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer);
            gl::DrawElements(gl::TRIANGLES, indices.len() as gl::types::GLsizei, gl::UNSIGNED_INT, std::ptr::null());

            gl::DeleteVertexArrays(1, &vao);
            // Render our loaded mesh with 
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

#[repr(C)]
struct GlVert {
    pos : [f32;4],
    uv  : [f32;2],
}
