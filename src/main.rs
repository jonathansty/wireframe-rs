/// # Wireframe-rs
/// ---
/// A simple app to demonstrate different kinds of wireframe renderings using opengl
extern crate assimp;
extern crate gl;
extern crate imgui;
extern crate nalgebra_glm as na;
extern crate sdl2;
extern crate time;
extern crate regex;

// MODULES
mod pipeline;
mod device;
mod imgui_gl;

// Imports
use imgui::ImGui;
use time::PreciseTime;

use gl::types::*;

use std::sync::Arc;


use crate::pipeline::{ ShaderUniform, Pipeline};

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

    // Create the GL device
    let mut gl = device::create_default_device(&window);

    // Enable opengl callbacks
    gl.borrow().enable_debug_layer().expect("Failed to enable debugging capabilities!");

    // Disable vsync
    video_subsystem
        .gl_set_swap_interval(1)
        .expect("Failed to set swap interval.");


    // Initialize IMGUI
    let mut imgui = ImGui::init();
    let mut imgui_renderer = imgui_gl::ImGuiGl::new(&mut imgui);

    // Build some shaders
    let shader_building = PreciseTime::now();
    let mut draw_mode = WireframeMode::None;
    let mut paused = false;

    // Create the default shader programs
    let default_vert = include_bytes!("../shaders/default.vert");
    let default_frag = include_bytes!("../shaders/default.frag");

    let default_program = Arc::new(Pipeline::create_simple(default_vert, default_frag).unwrap());
    let wireframe_program = Arc::new({
        let mut p = Pipeline::create_simple(default_vert, include_bytes!("../shaders/wireframe.frag")).expect("Failed to create the wireframe program.");
        p.set_fill_mode(crate::pipeline::FillMode::Lines);
        p.set_depth_test(false);
        p
    });
    let wireframe_singlepass = Arc::new(Pipeline::create_simple_with_geom(default_vert, include_bytes!("../shaders/default.geom"), include_bytes!("../shaders/default_wireframe.frag")).expect("Failed to create singlepass wireframe"));

    println!(
        "Shader compiling and setup took {}ms",
        shader_building.to(PreciseTime::now()).num_milliseconds()
    );

    // Load our mesh and setup buffers

    let mesh_process_start = PreciseTime::now();

    let (vertices,indices) = load_mesh("assets/suzanne.obj").expect("Failed to load model from disk!");

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

    // Record the start timings
    let _start_time = time::precise_time_s();
    let mut elapsed = 0.0;
    let mut curr_time = 0.0;
    let mut curr_item = 0;

    // Properties
    let mut line_thickness = 0.01;

    let mut line_color = [0.0,0.0,0.0,1.0];
    let mut solid_color = [1.0,1.0,1.0,1.0];

    // Create some command lists.
    let clear_color = [0.5,0.5,0.5,1.0];
    let mut default_list = gl.borrow().create_command_list();
    {
        default_list.clear(clear_color, None);
        default_list.bind_pipeline(&default_program);
        default_list.bind_vertex_buffers(0, 1, &[suzanne_vertex_buffer], &[0]);
        default_list.bind_index_buffer(&suzanne_index_buffer, 0, device::IndexType::UnsignedInt);
        default_list.draw_indexed(indices.len() as u32, 1, 0,0,0);
    }

    let mut singlepass_list = gl.borrow().create_command_list();
    {
        singlepass_list.clear(clear_color, None);
        singlepass_list.bind_pipeline(&wireframe_singlepass);
        singlepass_list.bind_vertex_buffers(0, 1, &[suzanne_vertex_buffer], &[0]);
        singlepass_list.bind_index_buffer(&suzanne_index_buffer, 0, device::IndexType::UnsignedInt);
        singlepass_list.draw_indexed(indices.len() as u32, 1, 0,0,0);
    }

    let mut multipass_list = gl.borrow().create_command_list();
    {
        multipass_list.clear(clear_color, None);
        multipass_list.bind_pipeline(&default_program);
        multipass_list.bind_vertex_buffers(0, 1, &[suzanne_vertex_buffer], &[0]);
        multipass_list.bind_index_buffer(&suzanne_index_buffer, 0, device::IndexType::UnsignedInt);
        multipass_list.draw_indexed(indices.len() as u32, 1, 0,0,0);

        multipass_list.bind_pipeline(&wireframe_program);
        multipass_list.draw_indexed(indices.len() as u32, 1, 0,0,0);
    }

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
            // gl.borrow().set_viewport(0,0, size.0 as i32, size.1 as i32);
            // gl.borrow().clear([0.0,0.0,0.0,0.0],0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let final_mat = projection * &view * model;
            // Render our loaded mesh
            gl::BindVertexArray(suzanne_vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, suzanne_index_buffer);

            // Execute rendering code for certain modes
            match draw_mode {
                WireframeMode::None => {
                    // First draw
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

                    // Manually update some uniforms
                    let p = &default_program;
                    p.set_uniform("model", ShaderUniform::Mat4(model.into()));
                    p.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    default_program.flush();

                    // Execute the command list
                    default_list.execute(&gl);
                },
                WireframeMode::SinglePass | WireframeMode::SinglePassCorrection => {
                    wireframe_singlepass.set_uniform("u_line_thickness", ShaderUniform::Float(line_thickness));
                    match draw_mode {
                        WireframeMode::SinglePassCorrection => {
                            wireframe_singlepass.set_uniform("u_correction", ShaderUniform::Int(1));
                        }
                        _ => {
                            wireframe_singlepass.set_uniform("u_correction", ShaderUniform::Int(0));
                        }
                    }
                    wireframe_singlepass.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    wireframe_singlepass.set_uniform("model", ShaderUniform::Mat4(model.into()));
                    wireframe_singlepass.set_uniform("u_object_color", ShaderUniform::Float4(solid_color.into()));
                    wireframe_singlepass.set_uniform("u_wireframe_color", ShaderUniform::Float4(line_color.into()));
                    wireframe_singlepass.flush();

                    singlepass_list.execute(&gl);
                }
                WireframeMode::MultiPass => {
                    //#TODO: Rebuild command list if needed?
                    default_program.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    default_program.set_uniform("model", ShaderUniform::Mat4(model.into()));

                    wireframe_program.set_uniform("projection", ShaderUniform::Mat4(final_mat.into()));
                    wireframe_program.set_uniform("model", ShaderUniform::Mat4(model.into()));

                    default_program.flush();
                    wireframe_program.flush();
                    gl::LineWidth(line_thickness);
                    multipass_list.execute(&gl);
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

    pub fn log_gl_errors() {
        if let Err(errors) = check_gl_errors() {
            for e in errors {
                println!("GL ERROR: {}", e);
            }
        }
    }
    /// Checks if any opengl errors occurred (flushes the error log)
    pub fn check_gl_errors() -> Result<(), Vec<GLuint>> {
        debug_assert!(gl::GetError::is_loaded());
        let mut v = Vec::new();
        unsafe {
            let mut result = gl::GetError();
            while result != gl::NO_ERROR {
                v.push(result);
                result = gl::GetError();
            }
        }

        if v.len() != 0 {
            return Err(v);
        }

        return Ok(());
    }

}
mod shaders {
    use gl::types::*;
    use std::ffi::CString;

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

fn load_mesh(path : &str) -> Result<(Vec<GlVert>, Vec<u32>), String> {
    use assimp::Importer;

    let importer = Importer::new();
    let scene = importer.read_file(path)?;
    println!("Loaded scene with {} meshes", scene.num_meshes());

    if scene.mesh(0).is_none() {
        return Err(String::from("Failed to find a mesh on index 0"));
    }

    let first_mesh = scene.mesh(0).unwrap();
    println!(
        "{} vertices, {} faces",
        first_mesh.num_vertices(),
        first_mesh.num_faces()
    );

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
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

    Ok((vertices,indices))
}