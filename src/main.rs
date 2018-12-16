extern crate assimp;
extern crate gl;
extern crate imgui;
extern crate nalgebra_glm as na;
/// This app demonstrates different ways of rendering wireframes
///
extern crate sdl2;
extern crate time;
extern crate regex;

mod pipeline;
mod imgui_gl;

use crate::pipeline::Pipeline;

use imgui::ImGui;
use std::ffi::CString;
use std::mem;
use time::PreciseTime;

use gl::types::*;

// Mode to control what program to use
enum WireframeMode {
    None,
    SinglePass,
    SinglePassCorrection,
    MultiPass,
}
impl WireframeMode{
    fn from_int(mode : u32) -> WireframeMode {
        match mode {
            0 => WireframeMode::None,
            1 => WireframeMode::SinglePass,
            2 => WireframeMode::SinglePassCorrection,
            3 => WireframeMode::MultiPass,
            _ => WireframeMode::None,
        }
    }
}

/// OpenGL callback
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

fn main() {
    // Setup SDL2
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("Wireframe Techniques", 800, 600)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    // Disable vsync
    video_subsystem
        .gl_set_swap_interval(0)
        .expect("Failed to set swap interval.");

    // Enable opengl callbacks
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(callback, std::ptr::null());
    }

    // Initialize IMGUI
    let mut imgui = ImGui::init();
    let mut imgui_renderer = imgui_gl::ImGuiGl::new(&mut imgui);

    // Build some shaders
    let shader_building = PreciseTime::now();
    let mut draw_mode = WireframeMode::None;
    let mut paused = false;

    // Create the default shader programs
    let mut default_program = Pipeline::create_simple(include_bytes!("../shaders/default.vert"), include_bytes!("../shaders/default.frag")).unwrap();
    default_program.flush();

    let mut wireframe_program = 0;
    let mut wireframe_singlepass = 0;
    {
        use std::ffi::CString;
        let vertex_source = CString::new(include_str!("../shaders/default.vert")).unwrap();
        let fragment_source = CString::new(include_str!("../shaders/default.frag")).unwrap();
        let geom_source = CString::new(include_str!("../shaders/default.geom")).unwrap();
        let wireframe_source = CString::new(include_str!("../shaders/wireframe.frag")).unwrap();
        let geom_wireframe_source =
            CString::new(include_str!("../shaders/default_wireframe.frag")).unwrap();

        let vert = match shaders::shader_from_source(&vertex_source, gl::VERTEX_SHADER) {
            Ok(shader) => shader,
            Err(msg) => {
                panic!("Failed to compile shader with following log:\n {}", msg);
            }
        };

        let frag = match shaders::shader_from_source(&fragment_source, gl::FRAGMENT_SHADER) {
            Ok(e) => e,
            Err(e) => {
                panic!("Failed to compile shader with following log:\n {}", e);
            }
        };

        let wireframe_frag =
            match shaders::shader_from_source(&wireframe_source, gl::FRAGMENT_SHADER) {
                Ok(e) => e,
                Err(_e) => panic!("Failed to compile wireframe shader."),
            };

        let geom_wireframe_frag =
            match shaders::shader_from_source(&geom_wireframe_source, gl::FRAGMENT_SHADER) {
                Ok(e) => e,
                Err(_e) => panic!("Failed to compile wireframe shader."),
            };

        let geom = match shaders::shader_from_source(&geom_source, gl::GEOMETRY_SHADER) {
            Ok(e) => e,
            Err(e) => {
                panic!("Failed to compile shader with following log:\n {}", e);
            }
        };

        let mut success = 1;
        unsafe {
            // Create Solid color black program!
            wireframe_program = gl::CreateProgram();
            gl::AttachShader(wireframe_program, vert);
            gl::AttachShader(wireframe_program, wireframe_frag);
            gl::LinkProgram(wireframe_program);

            gl::GetProgramiv(wireframe_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(wireframe_program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));

                let error = CString::from_vec_unchecked(buffer);
                gl::GetProgramInfoLog(
                    wireframe_program,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            // Create singlepass wireframe program
            wireframe_singlepass = gl::CreateProgram();
            gl::AttachShader(wireframe_singlepass, vert);
            gl::AttachShader(wireframe_singlepass, geom);
            gl::AttachShader(wireframe_singlepass, geom_wireframe_frag);
            gl::LinkProgram(wireframe_singlepass);

            gl::GetProgramiv(wireframe_singlepass, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(wireframe_singlepass, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));

                let error = CString::from_vec_unchecked(buffer);
                gl::GetProgramInfoLog(
                    wireframe_singlepass,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }
        }
    }
    println!(
        "Shader compiling and setup took {}ms",
        shader_building.to(PreciseTime::now()).num_milliseconds()
    );

    // Load our mesh and setup buffers

    let mesh_process_start = PreciseTime::now();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    {
        use assimp::Importer;
        let importer = Importer::new();
        // let scene = importer.read_file("assets/cube.obj").unwrap();
        let scene = importer.read_file("assets/suzanne.obj").unwrap();
        println!("Loaded scene with {} meshes", scene.num_meshes());

        let first_mesh = scene.mesh(0).unwrap();
        println!(
            "{} vertices, {} faces",
            first_mesh.num_vertices(),
            first_mesh.num_faces()
        );

        for i in 0..first_mesh.num_vertices() {
            let mut pos = assimp::Vector3D::new(0.0, 0.0, 0.0);
            let mut norm = assimp::Vector3D::new(0.0, 0.0, 0.0);
            let mut tan = assimp::Vector3D::new(0.0, 0.0, 0.0);
            let mut bitangent = assimp::Vector3D::new(0.0, 0.0, 0.0);

            if first_mesh.has_positions() {
                pos = first_mesh.get_vertex(i).unwrap();
            }

            if first_mesh.has_normals() {
                norm = first_mesh.get_normal(i).unwrap();
            }

            if first_mesh.has_tangents_and_bitangents() {
                tan = first_mesh.get_tangent(i).unwrap();
                bitangent = first_mesh.get_bitangent(i).unwrap();
            }

            vertices.push(GlVert {
                pos: [pos.x, pos.y, pos.z, 1.0],
                norm: [norm.x, norm.y, norm.z, 0.0],
                tangent: [tan.x, tan.y, tan.z, 0.0],
                bitangent: [bitangent.x, bitangent.y, bitangent.z, 0.0],
                uv: [0.0, 0.0],
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
            } else {
                let i0 = face[0];
                let i1 = face[1];
                let i2 = face[2];
                indices.push(i0);
                indices.push(i1);
                indices.push(i2);
            }
        }
    }
    let mesh_process_end = PreciseTime::now();
    println!(
        "Loading and processing the mesh took {}ms",
        mesh_process_start.to(mesh_process_end).num_milliseconds()
    );

    // Construct our setup
    let mut suzanne_vertex_buffer = 0;
    let mut suzanne_index_buffer = 0;
    unsafe {
        gl::GenBuffers(1, &mut suzanne_vertex_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, suzanne_vertex_buffer);
        let buffer_size = vertices.len() * std::mem::size_of::<GlVert>();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            buffer_size as isize,
            vertices.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );

        gl::GenBuffers(1, &mut suzanne_index_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, suzanne_index_buffer);
        let buffer_size = indices.len() * std::mem::size_of::<u32>();
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            buffer_size as isize,
            indices.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );
    }
    let suzanne_vao = unsafe { GlVert::setup_vao(suzanne_vertex_buffer) };

    let duration = mesh_process_end.to(PreciseTime::now());
    println!(
        "Submitting mesh data to GPU took {}ms",
        duration.num_milliseconds()
    );

    let mut event_pump = sdl.event_pump().unwrap();
    unsafe {
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    }

    // Set some default states
    unsafe {
        gl::CullFace(gl::BACK);
        gl::Enable(gl::DEPTH_TEST);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let window_size = window.size();
    let aspect = window_size.0 as f32 / window_size.1 as f32;

    // Define some default matrices
    let view = na::look_at(
        &na::Vec3::new(4.0, 1.8, 4.0),
        &na::Vec3::new(0.0, 0.0, 0.0),
        &na::Vec3::new(0.0, 1.0, 0.0),
    );
    let model = na::Mat4::new_translation(&na::Vec3::new(0.0, 0.0, 0.0));


    // Record the start timings
    let _start_time = time::precise_time_s();
    let mut elapsed = 0.0;
    let mut curr_time = 0.0;
    let mut curr_item = 0;

    // Properties
    let mut line_thickness = 0.01;

    let mut line_color = [0.0,0.0,0.0,1.0];
    let mut solid_color = [1.0,1.0,1.0,1.0];
    // Run the application
    'app: loop {
        let prev_time = curr_time;
        curr_time = time::precise_time_s();

        let dt = curr_time - prev_time;
        if !paused {
            elapsed += dt;
        }

        for e in event_pump.poll_iter() {
            use sdl2::event::Event;
            use sdl2::keyboard::Keycode;

            // Handle the input events for IMGUI first
            imgui_renderer.handle_event(&mut imgui, &e);

            match e {
                Event::Quit { .. } => {
                    break 'app;
                },
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => paused = !paused,
                _ => {}
            }
        }

        let size = window.size();
        let frame_size = imgui::FrameSize {
            logical_size: (size.0 as f64, size.1 as f64),
            hidpi_factor: 1.0,
        };
        let ui = imgui.frame(frame_size, dt as f32);
        // #TODO: proper structuring needed for the ui, for now do inline ui render
        {
            use imgui::im_str;
            use imgui::ImGuiCond;

            ui.window(im_str!("Wireframe-rs"))
                .size((300.0, 100.0), ImGuiCond::FirstUseEver)
                .build(|| {
                    ui.combo(im_str!("Draw mode"), &mut curr_item, &[im_str!("Default"), im_str!("Singlepass"), im_str!("Singlepass correction"), im_str!("Multipass")], 10);
                    ui.slider_float(im_str!("Line thickness"), &mut line_thickness, 0.001, 1.0).build();


                    ui.color_edit(im_str!("Solid color"), &mut solid_color ).build();
                    ui.color_edit(im_str!("Wireframe color"), &mut line_color ).build();
                });

            // Update draw mode using the IMGUI result
            draw_mode = WireframeMode::from_int(curr_item as u32);
        }

        let model = na::rotation(elapsed as f32, &na::Vec3::new(0.0, 1.0, 0.0));
        let aspect = size.0 as f32 / size.1 as f32;
        let projection = na::Mat4::new_perspective(aspect, 3.14 / 4.0, 0.01, 1000.0);

        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let final_mat = projection * &view * model;
            // Render our loaded mesh
            gl::BindVertexArray(suzanne_vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, suzanne_index_buffer);
            use crate::pipeline::ShaderUniform;
            match draw_mode {
                WireframeMode::None => {
                    // First draw
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    gl::UseProgram(default_program.program());
                    default_program.set_uniform("model", ShaderUniform::Mat4(model.into()));
                    default_program.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    default_program.flush();

                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as GLsizei,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                },
                WireframeMode::SinglePass | WireframeMode::SinglePassCorrection => {
                    let u_correction = gl::GetUniformLocation(
                        wireframe_singlepass,
                        CString::new("u_correction").unwrap().as_ptr(),
                    );
                    let model_loc = gl::GetUniformLocation(
                        wireframe_singlepass,
                        CString::new("model").unwrap().as_ptr(),
                    );
                    let vp_loc = gl::GetUniformLocation(
                        wireframe_singlepass,
                        CString::new("projection").unwrap().as_ptr(),
                    );

                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    gl::UseProgram(wireframe_singlepass);
                    let line_thickness_loc = gl::GetUniformLocation(
                        wireframe_singlepass,
                        CString::new("u_line_thickness").unwrap().as_ptr(),
                    );
                    gl::Uniform1fv(line_thickness_loc, 1, &line_thickness);
                    match draw_mode {
                        WireframeMode::SinglePassCorrection => {
                            gl::Uniform1iv(u_correction, 1, &1);
                        }
                        _ => {
                            gl::Uniform1iv(u_correction, 1, &0);
                        }
                    }

                    gl::UniformMatrix4fv(vp_loc, 1, gl::FALSE, final_mat.as_slice().as_ptr());
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_slice().as_ptr());
                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as GLsizei,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                }
                WireframeMode::MultiPass => {
                    // let model_loc = gl::GetUniformLocation(
                    //     default_program,
                    //     CString::new("model").unwrap().as_ptr(),
                    // );
                    // let vp_loc = gl::GetUniformLocation(
                    //     default_program,
                    //     CString::new("projection").unwrap().as_ptr(),
                    // );
                    // First draw
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    gl::UseProgram(default_program.program());
                    default_program.set_uniform("model", ShaderUniform::Mat4(model.into()));
                    default_program.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    default_program.flush();
                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as GLsizei,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );

                    let model_loc = gl::GetUniformLocation(
                        wireframe_program,
                        CString::new("model").unwrap().as_ptr(),
                    );
                    let vp_loc = gl::GetUniformLocation(
                        wireframe_program,
                        CString::new("projection").unwrap().as_ptr(),
                    );
                    // Second draw
                    gl::Disable(gl::DEPTH_TEST);
                    gl::LineWidth(line_thickness);
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
                    gl::UseProgram(wireframe_program);
                    gl::UniformMatrix4fv(vp_loc, 1, gl::FALSE, final_mat.as_slice().as_ptr());
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_slice().as_ptr());
                    gl::DrawElements(
                        gl::TRIANGLES,
                        indices.len() as GLsizei,
                        gl::UNSIGNED_INT,
                        std::ptr::null(),
                    );
                    gl::Enable(gl::DEPTH_TEST);
                }
            }

            // Setup our Imgui rendering
            let width = window.size().0 as f32;
            let height = window.size().1 as f32;
            let matrix = na::Mat4::from([
                [(2.0 / width) as f32, 0.0, 0.0, 0.0],
                [0.0, -(2.0 / height) as f32, 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ]);
            // Initiate the draw for all lists
            imgui_renderer.render(&matrix, ui);
        }

        window.gl_swap_window();
    }
}
mod helpers {
    use gl::types::*;

    /// Creates a simple program containing vertex and fragment shader
    pub fn create_simple_program( vertex_shader : GLuint, fragment_shader : GLuint) -> Result<GLuint, String> {
        // Check if the functions are loaded
        debug_assert!(gl::CreateProgram::is_loaded());
        debug_assert!(gl::AttachShader::is_loaded());
        debug_assert!(gl::LinkProgram::is_loaded());
        debug_assert!(gl::GetProgramiv::is_loaded());
        debug_assert!(gl::GetProgramInfoLog::is_loaded());

        // Check if atleast the shaders aren't, we should probably check if they are valid, using some kind of abstraction 
        debug_assert!(vertex_shader != 0 && fragment_shader != 0 );

        unsafe{
            let program =  gl::CreateProgram(); 
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut log_length = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
                let buffer = crate::helpers::alloc_buffer(log_length as usize);
                let error =  std::ffi::CString::from_vec_unchecked(buffer);

                gl::GetProgramInfoLog(
                    program,
                    log_length,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );

                return Err( error.to_str().unwrap().to_string() );
            }
            Ok(program)
        }
    }
    /// Sets the enumeration of openGL enabled and returns the previous state
    pub fn gl_set_enabled(enumeration: GLenum, enabled: bool) -> bool {
        debug_assert!(gl::Enable::is_loaded() && gl::Disable::is_loaded() && gl::GetIntegerv::is_loaded());
        unsafe {
            let mut previous_status = 0;
            gl::GetIntegerv(enumeration, &mut previous_status );

            match enabled {
                true => gl::Enable(enumeration),
                false => gl::Disable(enumeration),
            }

            previous_status != 0
        }
    }

    /// Allocates a byte buffer for usage with opengl error info logs
    pub fn alloc_buffer(len: usize) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(len as usize + 1);
        buffer.extend([b' '].iter().cycle().take(len as usize));
        buffer
    }

    /// Checks if any opengl errors occurred (flushes the error log)
    pub fn check_gl_errors() {
        debug_assert!(gl::GetError::is_loaded());
        unsafe {
            let mut result = gl::GetError();
            while result != gl::NO_ERROR {
                println!("OpenGL error {:?}", result);

                result = gl::GetError();
            }
        }
    }

}
mod shaders {
    use gl::types::*;
    use std::ffi::{CStr, CString};

    pub fn shader_from_source(source: &CString, kind: GLuint) -> Result<GLuint, String> {
        let id = unsafe { gl::CreateShader(kind) };

        let mut success: GLint = 1;
        unsafe {
            gl::ShaderSource(id, 1, &(source.as_ptr() as *const i8), std::ptr::null());
            gl::CompileShader(id);

            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let buffer = crate::helpers::alloc_buffer(len as usize);
            let error = unsafe { CString::from_vec_unchecked(buffer) };
            unsafe {
                gl::GetShaderInfoLog(
                    id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }
            return Err(error.to_string_lossy().into_owned());
        }
        Ok(id)
    }
}

#[repr(C)]
struct GlVert {
    pos: [f32; 4],
    norm: [f32; 4],
    tangent: [f32; 4],
    bitangent: [f32; 4],
    uv: [f32; 2],
}
impl GlVert {
    unsafe fn setup_vao(vtx: GLuint) -> GLuint {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vtx);

        let struct_size = std::mem::size_of::<GlVert>() as i32;

        gl::EnableVertexArrayAttrib(vao, 0);
        //Position 
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, struct_size, std::ptr::null());

        // Normal
        gl::EnableVertexArrayAttrib(vao, 1);
        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (4 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        // Tangent
        gl::EnableVertexArrayAttrib(vao, 2);
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (8 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        // Bitangent
        gl::EnableVertexArrayAttrib(vao, 3);
        gl::VertexAttribPointer(
            3,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (12 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        // UV
        gl::EnableVertexArrayAttrib(vao, 4);
        gl::VertexAttribPointer(
            4,
            2,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (16 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        vao
    }
}
