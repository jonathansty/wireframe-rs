/// OpenGL based implementation for using IMGUI (inspired by imgui implementations in C++)
/// 
use imgui::{DrawData, ImGui};
use gl::types::*;

pub struct ImGuiGl {
    font_textures : Vec<GLuint>,

    program : GLuint,
    vertex_buffer : GLuint,
    index_buffer : GLuint,
}

impl ImGuiGl {
    pub fn new(imgui : &mut ImGui) -> Self {
        imgui.fonts().add_default_font();

        let result = imgui.prepare_texture(|font_tex| {
            let mut texture = 0;
            unsafe{
                gl::GenTextures(1,&mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);

                let width = font_tex.width as i32;
                let height = font_tex.height as i32;

                gl::PixelStorei(gl::UNPACK_ROW_LENGTH,0 );
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width,height, 0, gl::RGBA, gl::UNSIGNED_BYTE, font_tex.pixels.as_ptr() as *const std::ffi::c_void);
            }

            return texture;
        });

        let mut vtx = 0;
        let mut idx = 0;
        unsafe{
            gl::GenBuffers(1,&mut vtx);
            gl::GenBuffers(1,&mut idx);
        }

        // Create the program
        let mut program = unsafe{ gl::CreateProgram() };
        unsafe{
            use std::ffi::CString;
            use super::shaders::shader_from_source;
            let vert = shader_from_source(&CString::new(include_str!("../shaders/imgui.vert")).unwrap(), gl::VERTEX_SHADER).unwrap();

            let frag = shader_from_source(&CString::new(include_str!("../shaders/imgui.frag")).unwrap(), gl::FRAGMENT_SHADER).unwrap();

            gl::AttachShader(program, vert);
            gl::AttachShader(program, frag);
            gl::LinkProgram(program);
            // Should check here.
        }



        ImGuiGl{
            font_textures: vec![result],
            program: program,
            vertex_buffer: vtx,
            index_buffer: idx,
        }
    }

    pub fn handle_event(&mut self, imgui : &mut ImGui, event : &sdl2::event::Event){
        use sdl2::event::Event;
        use sdl2::mouse::MouseButton;

        // Request previous state
        let mut mouse_downs = imgui.mouse_down();
        match event {
            Event::MouseMotion{x,y, xrel, yrel,.. } => {
                imgui.set_mouse_pos(*x as f32,*y as f32);
            },
            Event::MouseButtonDown{mouse_btn,..} => {
                match mouse_btn {
                    MouseButton::Left => mouse_downs[0] = true,
                    _ => {}
                }
            },
            Event::MouseButtonUp{mouse_btn, ..} => {
                match mouse_btn {
                    MouseButton::Left => mouse_downs[0] = false,
                    _ => {}
                }
            },
            _ => {}
        }

        // Update mouse downs
        imgui.set_mouse_down(mouse_downs);
    }

    pub fn render(&mut self, proj : &na::Mat4, ui : imgui::Ui) {
        let font_textures = &self.font_textures;

        unsafe{
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }

        let result = ui.render::<_, ()>(|_, data|{
            let vtx_size = data.total_vtx_count() as isize * std::mem::size_of::<imgui::ImDrawVert>() as isize;
            let idx_size = data.total_idx_count() as isize * std::mem::size_of::<imgui::ImDrawIdx>() as isize;
            unsafe{
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);

                gl::BufferData(gl::ARRAY_BUFFER, vtx_size, std::ptr::null(), gl::DYNAMIC_DRAW);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, idx_size , std::ptr::null(), gl::DYNAMIC_DRAW);
            }
            // Resize buffers

            // For each draw list we fill our buffers
            unsafe{
                let mut vtx_ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut imgui::ImDrawVert;
                let mut idx_ptr = gl::MapBuffer(gl::ELEMENT_ARRAY_BUFFER, gl::WRITE_ONLY) as *mut imgui::ImDrawIdx;
                for draw_list in &data {
                    let vtx_buffer = draw_list.vtx_buffer;
                    let idx_buffer = draw_list.idx_buffer;

                    std::ptr::copy_nonoverlapping(vtx_buffer.as_ptr(), vtx_ptr, vtx_buffer.len());
                    std::ptr::copy_nonoverlapping(idx_buffer.as_ptr(), idx_ptr, idx_buffer.len());
                    vtx_ptr = vtx_ptr.offset(vtx_buffer.len() as isize);
                    idx_ptr = idx_ptr.offset(idx_buffer.len() as isize);
                }

                gl::UnmapBuffer(gl::ARRAY_BUFFER);
                gl::UnmapBuffer(gl::ELEMENT_ARRAY_BUFFER);
            }

            // Execute all commands
            let mut vao = 0;
            unsafe{
                gl::GenVertexArrays(1,&mut vao);

                gl::BindVertexArray(vao);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);

                use imgui::ImDrawVert;
                let struct_size = std::mem::size_of::<ImDrawVert>() as i32;

                gl::EnableVertexArrayAttrib(vao, 0);
                gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, struct_size, std::ptr::null());

                gl::EnableVertexArrayAttrib(vao, 1);
                gl::VertexAttribPointer(
                    1,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    struct_size,
                    (2 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
                );

                gl::EnableVertexArrayAttrib(vao, 2);
                gl::VertexAttribPointer(
                    2,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    struct_size,
                    (4 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
                );

                gl::UseProgram(self.program);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
                let loc = gl::GetUniformLocation(self.program, std::ffi::CString::new("u_proj").unwrap().as_ptr());
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, proj.as_slice().as_ptr());
                gl::Uniform1i(gl::GetUniformLocation(self.program, "u_font".as_ptr() as *const i8),0);
            }
            for draw_list in &data {
                let mut offset = 0;
                unsafe{
                    gl::BindVertexArray(vao);
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
                }

                for cmd in draw_list.cmd_buffer {
                    unsafe{
                        gl::ActiveTexture(gl::TEXTURE0);
                        gl::BindTexture(gl::TEXTURE_2D, font_textures[0]);

                        let index_size = std::mem::size_of::<imgui::ImDrawIdx>();
                        let mut index_enum = gl::UNSIGNED_INT;
                        if index_size == 2
                        {
                            index_enum = gl::UNSIGNED_SHORT;
                        }

                        gl::DrawElements(
                            gl::TRIANGLES,
                            cmd.elem_count as i32,
                            index_enum,
                            (offset*2) as *const std::ffi::c_void,
                        );
                    }
                    offset += cmd.elem_count;
                }
            }

            unsafe{
                gl::DeleteVertexArrays(1, &vao);
                gl::Disable(gl::BLEND);
                gl::Enable(gl::CULL_FACE);
                gl::Enable(gl::DEPTH_TEST);
                gl::Disable(gl::SCISSOR_TEST);
            }

            Ok(())
        });
    }
}