use regex::Regex;

use crate::shaders;
use crate::helpers;

use gl::types::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// Available shader stages that are implemented
enum ShaderStage {
    Vertex,
    Fragment,
    Geometry,
    // Compute,
}

/// Shader uniforms are parsed from input source and can be different types
pub enum ShaderUniform{
    Int(i32),
    Float(f32),
    Float2([f32;2]),
    Float3([f32;3]),
    Float4([f32;4]),
    Mat3([[f32;3];3]),
    Mat4([[f32;4];4]),
}
impl ShaderUniform {
    fn get_type (&self) -> UniformType {
        match self {
            ShaderUniform::Int(_) => UniformType::Int,
            ShaderUniform::Float(_) => UniformType::Float,
            ShaderUniform::Float2(_) => UniformType::Float2,
            ShaderUniform::Float3(_) => UniformType::Float3,
            ShaderUniform::Mat3(_) => UniformType::Mat3,
            ShaderUniform::Mat4(_) => UniformType::Mat4,
            _ => UniformType::Int,
        }
    }

    fn from_uniform_type(uniform_type : UniformType) -> Self{
        match uniform_type {
            UniformType::Int => ShaderUniform::Int(0),
            UniformType::Float => ShaderUniform::Float(0.0),
            UniformType::Float2 => ShaderUniform::Float2([0.0,0.0]),
            UniformType::Float3 => ShaderUniform::Float3([0.0,0.0,0.0]),
            UniformType::Float4 => ShaderUniform::Float4([0.0,0.0,0.0,0.0]),
            UniformType::Mat3 => ShaderUniform::Mat3(
                [
                    [1.0,0.0,0.0],
                 [0.0,1.0,0.0],
                 [0.0,0.0,1.0]
                ]),
            UniformType::Mat4 => ShaderUniform::Mat4(
                [[1.0,0.0,0.0, 0.0],
                 [0.0,1.0,0.0, 0.0],
                 [0.0,0.0,1.0, 0.0],
                 [0.0,0.0,0.0, 1.0]
                 ])
            ,
            _ => panic!("Unimplemented uniform type found!")
        }
    }
}
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum UniformType {
    Int,
    Float,
    Float2,
    Float3,
    Float4,
    Mat3,
    Mat4
}

type ShaderUniformLoc = (GLint, ShaderUniform);
pub struct Pipeline {
    program : GLuint,

    // Graphics pipeline properties
    depth_test : bool,
    blend_enabled : bool,

    // Collection of shader uniforms found when creating the pipeline
    uniforms : HashMap<(String, UniformType), ShaderUniformLoc>
}

impl Pipeline {
    pub fn program(&self) -> GLuint { self.program }

    pub fn set_uniform(&mut self, name : &str, uniform : ShaderUniform) {
        let uniform_type = uniform.get_type();
       let entry = self.uniforms.entry((name.to_string(), uniform_type));
       match entry {
           Entry::Occupied(mut ent) => {
              (*ent.into_mut()).1 = uniform;
           },
           Entry::Vacant(vac) => {
               println!("Uniform \"{}\" not found in shader!", name);
           }
       };
    }

    /// Uploads all bound uniforms to the GPU
    pub fn flush(&self) {
        unsafe{
            gl::UseProgram(self.program);
        }

        // Flushes all uniforms to the GPU
        use std::ffi::CString;
        for (key, value) in self.uniforms.iter() {
            unsafe{
                let shader_loc = value.0;
                if shader_loc != -1 {
                    // Upload depending on shader uniform type
                    match value.1 {
                        ShaderUniform::Int(v) => gl::Uniform1iv(shader_loc, 1, &v),
                        ShaderUniform::Float(v) => gl::Uniform1fv(shader_loc, 1, &v),
                        ShaderUniform::Float3(v) => gl::Uniform3fv(shader_loc, 1,v.as_ptr()),
                        ShaderUniform::Float4(v) => gl::Uniform4fv(shader_loc, 1, v.as_ptr()),
                        ShaderUniform::Mat3(v) => gl::UniformMatrix3fv(shader_loc, 1, gl::FALSE, v[0].as_ptr()),
                        ShaderUniform::Mat4(v) => gl::UniformMatrix4fv(shader_loc, 1, gl::FALSE, v[0].as_ptr()),
                        _ => {
                            println!("Unimplemented uniform while flushing...");
                        }
                    }
                }
            }
        }
    }

    pub fn create_simple(vertex_source : &[u8], fragment_source : &[u8]) -> Result<Pipeline, String> {
        use std::ffi::CString;

        let vertex_shader   = shaders::shader_from_source(&CString::new(vertex_source).unwrap(), gl::VERTEX_SHADER)?;
        let fragment_shader = shaders::shader_from_source(&CString::new(fragment_source).unwrap(), gl::FRAGMENT_SHADER)?;


        let mut uniforms = HashMap::new();
        parse_uniforms(vertex_source, &mut uniforms);
        parse_uniforms(fragment_source, &mut uniforms);


        println!("Found {} uniforms.", uniforms.len());

        let mut program = helpers::create_simple_program(vertex_shader, fragment_shader)?;

        // Find locations for all uniforms
        for (key,mut value) in &mut uniforms {
            unsafe{
                let shader_loc = gl::GetUniformLocation(program, CString::from_vec_unchecked(key.0.as_bytes().to_vec()).as_ptr());
                value.0 = shader_loc;
            }
        }

        Ok( 
            Pipeline{
                blend_enabled: false,
                depth_test: true,
                program,
                uniforms
            }
        )
    }

}

/// Simple uniform parsing of a source string (for openGL, does not allow layout bindings yet) 
fn parse_uniforms(source : &[u8], result : &mut HashMap<(String, UniformType), ShaderUniformLoc>){
    // Construct the regex
    let uniform_regex = Regex::new(r"(uniform)\s(?P<type>\w*)\s(?P<var>\w*);").expect("Failed to create regex!");

    // Convert our input data to a string
    let c =  String::from_utf8(source.to_vec()).unwrap();

    // Execute the regex on our source data
    let results = uniform_regex.captures_iter(&c);
    for matches in results {
        let var_name = &matches["var"];
        let var_type = match &matches["type"] {
            "int" => UniformType::Int,
            "float" => UniformType::Float,
            "float2" => UniformType::Float2,
            "float3" => UniformType::Float3,
            "float4" => UniformType::Float4,
            "mat3" => UniformType::Mat3,
            "mat4" => UniformType::Mat4,
            _ => panic!("Unrecognized shader uniform found in shader!")
        };

        let var_default_value = ShaderUniform::from_uniform_type(var_type);

        let result_key = (var_name.to_string(), var_type);
        result.entry(result_key).or_insert((-1,var_default_value));
    }
}
