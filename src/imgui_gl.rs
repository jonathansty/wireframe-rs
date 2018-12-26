/// OpenGL based implementation for using IMGUI (inspired by imgui implementations in C++)
/// 
use imgui::ImGui;
use crate::helpers;
use gl::types::*;

pub struct ImGuiGl {
    font_textures : Vec<GLuint>,

    program : crate::pipeline::Pipeline,
    vertex_buffer : GLuint,
    index_buffer : GLuint,
}

impl Drop for ImGuiGl {
    fn drop(&mut self) {
        unsafe{
            gl::DeleteProgram(self.program.program());
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}
impl ImGuiGl {
    pub fn new(imgui : &mut ImGui) -> Self {
        imgui.fonts().add_default_font();

        imgui.set_imgui_key(imgui::ImGuiKey::Tab, sdl2::keyboard::Keycode::Tab as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::LeftArrow, sdl2::keyboard::Keycode::Left as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::RightArrow, sdl2::keyboard::Keycode::Right as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::UpArrow, sdl2::keyboard::Keycode::Up as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::DownArrow, sdl2::keyboard::Keycode::Down as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::Delete, sdl2::keyboard::Keycode::Delete as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::Backspace, sdl2::keyboard::Keycode::Backspace as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::Enter, sdl2::keyboard::Keycode::Return as u8);
        imgui.set_imgui_key(imgui::ImGuiKey::Escape, sdl2::keyboard::Keycode::Escape as u8);

        let mut last_texture  = 0;
        unsafe{
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut last_texture);
        }

        let result = imgui.prepare_texture(|font_tex| {
            let mut texture = 0;
            let width = font_tex.width as i32;
            let height = font_tex.height as i32;
            unsafe{
                gl::GenTextures(1,&mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                gl::PixelStorei(gl::UNPACK_ROW_LENGTH,0 );
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width,height, 0, gl::RGBA, gl::UNSIGNED_BYTE, font_tex.pixels.as_ptr() as *const std::ffi::c_void);
            }
            helpers::log_gl_errors();

            return texture;
        });

        unsafe{
            // Restore last texture state
            gl::BindTexture(gl::TEXTURE_2D, last_texture as u32);
        }
        imgui.fonts().set_texture_id(result as usize);

        let mut vtx = 0;
        let mut idx = 0;
        unsafe{
            gl::GenBuffers(1,&mut vtx);
            gl::GenBuffers(1,&mut idx);
        }

        // Create the program
        let program = crate::pipeline::Pipeline::create_simple(include_bytes!("../shaders/imgui.vert"), include_bytes!("../shaders/imgui.frag")).unwrap();
        helpers::log_gl_errors();
        program.flush();
        helpers::log_gl_errors();

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
            Event::TextInput{text, .. } => {
                for c in text.chars() {
                    imgui.add_input_character(c);
                }
            },
            Event::KeyDown{keycode, scancode, keymod, .. } => {
                imgui.set_key(keycode.unwrap() as u8, true);
                let ctrl_down = keymod.contains(sdl2::keyboard::LCTRLMOD);
                imgui.set_key_ctrl(ctrl_down);

                let shift_down = keymod.contains(sdl2::keyboard::LSHIFTMOD);
                imgui.set_key_shift(shift_down);
            },
            Event::KeyUp{keycode, scancode, keymod, .. } => {
                imgui.set_key(keycode.unwrap() as u8, false);

                let ctrl_down = keymod.contains(sdl2::keyboard::LCTRLMOD);
                imgui.set_key_ctrl(ctrl_down);

                let shift_down = keymod.contains(sdl2::keyboard::LSHIFTMOD);
                imgui.set_key_shift(shift_down);
            },
            Event::MouseMotion{x,y,.. } => {
                imgui.set_mouse_pos(*x as f32,*y as f32);
            },
            Event::MouseWheel{x, y, direction, .. } => {
                imgui.set_mouse_wheel(*y as f32);
            },
            Event::MouseButtonDown{mouse_btn,..} => {
                match mouse_btn {
                    MouseButton::Left => mouse_downs[0] = true,
                    MouseButton::Right => mouse_downs[1] = true,
                    MouseButton::Middle => mouse_downs[2] = true,
                    _ => {}
                }
            },
            Event::MouseButtonUp{mouse_btn, ..} => {
                match mouse_btn {
                    MouseButton::Left => mouse_downs[0] = false,
                    MouseButton::Middle => mouse_downs[2] = false,
                    MouseButton::Right => mouse_downs[1] = false,
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

        use crate::helpers::gl_set_enabled;
        let last_blending_enabled = gl_set_enabled(gl::BLEND, true);
        let last_depth_test_enabled = gl_set_enabled(gl::DEPTH_TEST, false);
        let last_culling_enabled = gl_set_enabled(gl::CULL_FACE, false);
        let last_enabled_scissor_test = gl_set_enabled(gl::SCISSOR_TEST, true);

        unsafe{
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        }


        let imgui::FrameSize{logical_size: (fb_width, fb_height), ..} = ui.frame_size();
        let _result = ui.render::<_, ()>(|_, data|{

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

                gl::UseProgram(self.program.program());
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
                use crate::pipeline::ShaderUniform;
                self.program.set_uniform("u_proj", ShaderUniform::Mat4((*proj).into()));
                self.program.set_uniform("u_font", ShaderUniform::Sampler2D(self.font_textures[0]));
                self.program.flush();
            }


            for draw_list in &data {
                let mut offset = 0;
                unsafe{
                    gl::BindVertexArray(vao);
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
                    gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);

                    use imgui::{ImDrawVert, ImDrawIdx};
                    let size = draw_list.vtx_buffer.len() * std::mem::size_of::<ImDrawVert>();
                    use std::ffi::c_void;
                    gl::BufferData(gl::ARRAY_BUFFER, size as isize, draw_list.vtx_buffer.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);

                    let size = draw_list.idx_buffer.len() * std::mem::size_of::<ImDrawIdx>();
                    gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, size as isize , draw_list.idx_buffer.as_ptr() as *const c_void, gl::DYNAMIC_DRAW);
                }

                for cmd in draw_list.cmd_buffer {
                    unsafe{
                        gl::Scissor(cmd.clip_rect.x as i32, fb_height as i32 - cmd.clip_rect.w as i32, (cmd.clip_rect.z - cmd.clip_rect.x) as i32, (cmd.clip_rect.w - cmd.clip_rect.y) as i32);
                        gl::ActiveTexture(gl::TEXTURE0);
                        gl::BindTexture(gl::TEXTURE_2D, cmd.texture_id as u32);

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

            Ok(())
        });

        // Re-enable all state
        gl_set_enabled(gl::BLEND, last_blending_enabled);
        gl_set_enabled(gl::DEPTH_TEST, last_depth_test_enabled);
        gl_set_enabled(gl::CULL_FACE, last_culling_enabled);
        gl_set_enabled(gl::SCISSOR_TEST, last_enabled_scissor_test);
    }
}

