extern crate assimp;
extern crate gl;
extern crate imgui;
extern crate nalgebra_glm as na;
/// This app demonstrates different ways of rendering wireframes
///
extern crate sdl2;
extern crate time;

mod imgui_gl;

use imgui::ImGui;
use std::mem;
use time::PreciseTime;

use gl::types::*;

// Mode to control what program to use
enum WireframeMode {
    MultiPass,
    SinglePassNoCorrection,
    SinglePassCorrection,
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

    unsafe {
        #[allow(unused_assignments)]
        extern "system" fn callback(
            _source: u32,
            _gltype: u32,
            _id: u32,
            _severity: u32,
            _length: i32,
            message: *const i8,
            _user_param: *mut std::ffi::c_void,
        ) {
            unsafe {
                let string = std::ffi::CStr::from_ptr(message);
                println!("{}", string.to_str().unwrap());
            }
        }
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(callback, std::ptr::null());
    }

    // Initialize IMGUI
    let mut imgui = ImGui::init();
    let mut imgui_renderer = imgui_gl::ImGuiGl::new(&mut imgui);

    // Build some shaders
    let shader_building = PreciseTime::now();
    let mut draw_mode = WireframeMode::SinglePassNoCorrection;
    let mut paused = false;

    // Create the default shader programs
    let mut default_program = 0;
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
                Err(e) => panic!("Failed to compile wireframe shader."),
            };

        let geom_wireframe_frag =
            match shaders::shader_from_source(&geom_wireframe_source, gl::FRAGMENT_SHADER) {
                Ok(e) => e,
                Err(e) => panic!("Failed to compile wireframe shader."),
            };

        let geom = match shaders::shader_from_source(&geom_source, gl::GEOMETRY_SHADER) {
            Ok(e) => e,
            Err(e) => {
                panic!("Failed to compile shader with following log:\n {}", e);
            }
        };

        let mut success = 1;
        unsafe {
            // Create default shaded drawing program
            default_program = gl::CreateProgram();
            gl::AttachShader(default_program, vert);
            gl::AttachShader(default_program, frag);
            gl::LinkProgram(default_program);

            gl::GetProgramiv(default_program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(default_program, gl::INFO_LOG_LENGTH, &mut len);

                let mut buffer = Vec::with_capacity(len as usize + 1);
                buffer.extend([b' '].iter().cycle().take(len as usize));

                let error = CString::from_vec_unchecked(buffer);
                gl::GetProgramInfoLog(
                    default_program,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

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
    let mut vtx_buffer = 0;
    let mut idx_buffer = 0;
    unsafe {
        gl::GenBuffers(1, &mut vtx_buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer);
        let buffer_size = vertices.len() * std::mem::size_of::<GlVert>();
        gl::BufferData(
            gl::ARRAY_BUFFER,
            buffer_size as isize,
            vertices.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );

        gl::GenBuffers(1, &mut idx_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer);
        let buffer_size = indices.len() * std::mem::size_of::<u32>();
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            buffer_size as isize,
            indices.as_ptr() as *const std::ffi::c_void,
            gl::STATIC_DRAW,
        );
    }
    let duration = mesh_process_end.to(PreciseTime::now());
    println!(
        "Submitting mesh data to GPU took {}ms",
        duration.num_milliseconds()
    );

    let mut event_pump = sdl.event_pump().unwrap();
    unsafe {
        gl::ClearColor(0.3, 0.3, 0.3, 1.0);
    }

    unsafe {
        gl::CullFace(gl::BACK);
        gl::Enable(gl::DEPTH_TEST);
    }

    let window_size = window.size();
    let aspect = window_size.0 as f32 / window_size.1 as f32;

    let projection = na::Mat4::new_perspective(aspect, 3.14 / 4.0, 0.01, 1000.0);
    let view = na::look_at(
        &na::Vec3::new(3.0, 1.8, 3.0),
        &na::Vec3::new(0.0, 0.0, 0.0),
        &na::Vec3::new(0.0, 1.0, 0.0),
    );
    let model = na::Mat4::new_translation(&na::Vec3::new(0.0, 0.0, 0.0));

    let mut start_time = time::precise_time_s();
    let mut elapsed = 0.0;
    let mut curr_time = 0.0;
    // Run the application
    'app: loop {
        let prev_time = curr_time;
        curr_time = time::precise_time_s();

        let dt = (curr_time - prev_time);
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
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => {
                    println!("Switching to single pass no correction");
                    draw_mode = WireframeMode::SinglePassNoCorrection;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => {
                    draw_mode = WireframeMode::SinglePassCorrection;
                    println!("Switching to singlepass with correction");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => {
                    draw_mode = WireframeMode::MultiPass;
                    println!("Switching to multipass");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => paused = !paused,
                _ => {}
            }
        }

        let size = window.size();
        let frame_size = imgui::FrameSize{logical_size: (size.0 as f64, size.1 as f64), hidpi_factor: 1.0};
        let ui = imgui.frame(frame_size, dt as f32);
        // #TODO: proper structuring needed for the ui, for now do inline ui render
        {
            use imgui::im_str;
            use imgui::ImGuiCond;
            ui.window(im_str!("Hello"))
                .size((300.0,100.0), ImGuiCond::FirstUseEver)
                .build(||{
                    ui.text(im_str!("Hello!"));
                });
        }


        let model = na::rotation(elapsed as f32, &na::Vec3::new(0.0, 1.0, 0.0));
        let aspect = size.0 as f32 / size.1 as f32;
        let projection = na::Mat4::new_perspective(aspect, 3.14 / 4.0, 0.01, 1000.0);
        unsafe {
            gl::Viewport(0, 0, size.0 as i32, size.1 as i32);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let final_mat = projection * &view * model;
            use std::ffi::CString;

            let vao = GlVert::setup_vao();
            gl::BindBuffer(gl::ARRAY_BUFFER, vtx_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, idx_buffer);
            match draw_mode {
                WireframeMode::SinglePassNoCorrection | WireframeMode::SinglePassCorrection => {
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
                    match draw_mode {
                        WireframeMode::SinglePassNoCorrection => {
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
                    let model_loc = gl::GetUniformLocation(
                        default_program,
                        CString::new("model").unwrap().as_ptr(),
                    );
                    let vp_loc = gl::GetUniformLocation(
                        default_program,
                        CString::new("projection").unwrap().as_ptr(),
                    );
                    // First draw
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                    gl::UseProgram(default_program);
                    gl::UniformMatrix4fv(vp_loc, 1, gl::FALSE, final_mat.as_slice().as_ptr());
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_slice().as_ptr());
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
                    // gl::Disable(gl::DEPTH_TEST);
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
                    // gl::Enable(gl::DEPTH_TEST);
                }
            }

            gl::DeleteVertexArrays(1, &vao);

            // Start IMGUI rendering
            imgui_renderer.render(ui);
            // render_ui(&ui);
        }

        window.gl_swap_window();
    }
}
pub fn render_ui(ui : &imgui::Ui){

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
            let mut buffer = Vec::with_capacity(len as usize + 1);
            buffer.extend([b' '].iter().cycle().take(len as usize));

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
    unsafe fn setup_vao() -> GLuint {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let struct_size = std::mem::size_of::<GlVert>() as i32;

        gl::EnableVertexArrayAttrib(vao, 0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, struct_size, std::ptr::null());

        gl::EnableVertexArrayAttrib(vao, 1);
        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (4 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        gl::EnableVertexArrayAttrib(vao, 2);
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (8 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        gl::EnableVertexArrayAttrib(vao, 3);
        gl::VertexAttribPointer(
            3,
            4,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (12 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        gl::EnableVertexArrayAttrib(vao, 4);
        gl::VertexAttribPointer(
            4,
            2,
            gl::FLOAT,
            gl::FALSE,
            struct_size,
            (16 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
        );

        vao
    }
}
