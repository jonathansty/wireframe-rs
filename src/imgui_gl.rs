/// OpenGL based implementation for using IMGUI (inspired by imgui implementations in C++)
/// 
use imgui::{DrawData, ImGui};
use gl::types::*;

pub struct ImGuiGl {
    font_textures : Vec<GLuint>,

    vertex_buffer : GLuint,
    index_buffer : GLuint,
}

impl ImGuiGl {
    pub fn new(imgui : &mut ImGui) -> Self {
        let result = imgui.prepare_texture(|font_tex| {
            let mut texture = 0;
            unsafe{
                gl::GenTextures(1,&mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);

                let width = font_tex.width as i32;
                let height = font_tex.height as i32;
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

        ImGuiGl{
            font_textures: vec![result],
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
            Event::MouseMotion{x,y, xrel, yrel,.. } => {},
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

    pub fn render(&mut self, ui : imgui::Ui) {
        let font_textures = &self.font_textures;

        let result = ui.render::<_, ()>(|_, data|{
            // Resize buffers
            let vtx_size = data.total_vtx_count() as isize * std::mem::size_of::<imgui::ImDrawVert>() as isize;
            let idx_size = data.total_idx_count() as isize * std::mem::size_of::<imgui::ImDrawIdx>() as isize;
            unsafe{
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
                gl::BufferData(gl::ARRAY_BUFFER, vtx_size , std::ptr::null(), gl::DYNAMIC_DRAW);
                gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, idx_size , std::ptr::null(), gl::DYNAMIC_DRAW);
            }

            // For each draw list we fill our buffers
            // unsafe{
            //     let mut vtx_ptr = gl::MapBuffer(gl::ARRAY_BUFFER, gl::WRITE_ONLY) as *mut imgui::ImDrawVert;
            //     let mut idx_ptr = gl::MapBuffer(gl::ELEMENT_ARRAY_BUFFER, gl::WRITE_ONLY) as *mut imgui::ImDrawIdx;
            //     for draw_list in &data {
            //         let idx_buffer = draw_list.idx_buffer;
            //         let vtx_buffer = draw_list.vtx_buffer;

            //         let vtx_size = vtx_buffer.len();
            //         let idx_size = idx_buffer.len();

            //         std::ptr::copy(vtx_buffer.as_ptr(), vtx_ptr, vtx_size);
            //         std::ptr::copy(idx_buffer.as_ptr(), idx_ptr, idx_size);
            //         vtx_ptr = vtx_ptr.offset(vtx_size as isize);
            //         idx_ptr = idx_ptr.offset(idx_size as isize);
            //     }

            //     gl::UnmapBuffer(gl::ARRAY_BUFFER);
            //     gl::UnmapBuffer(gl::ELEMENT_ARRAY_BUFFER);
            // }

            // Execute all commands
            for draw_list in &data {
                for cmd in draw_list.cmd_buffer {

                }
            }

            Ok(())
        });
    }
}