/// OpenGL based implementation for using IMGUI (inspired by imgui implementations in C++)
/// 
use imgui::{DrawData, ImGui};
use gl::types::*;

pub struct ImGuiGl<'a> {
    font_textures : Vec<GLuint>,
    imgui : &'a mut ImGui,

}

impl<'a> ImGuiGl<'a> {
    pub fn new(imgui : &'a mut ImGui) -> Self {
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

        ImGuiGl{
            font_textures: vec![result],
            imgui: imgui
        }
    }

    pub fn imgui(&mut self) -> &mut ImGui {
        &mut self.imgui
    }

    pub fn handle_event(&mut self, event : &sdl2::event::Event){
        use sdl2::event::Event;
        use sdl2::mouse::MouseButton;

        // Request previous state
        let mut mouse_downs = self.imgui.mouse_down();
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
        self.imgui.set_mouse_down(mouse_downs);
    }

    pub fn render(&mut self, ui : &imgui::Ui) {

    }
}