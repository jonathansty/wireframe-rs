use regex::Regex;

use crate::shaders;
use crate::device::PrimitiveTopology;

use gl::types::*;
use std::cell::RefCell;
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
#[derive(Clone, Copy)]
pub enum ShaderUniform{
    Invalid,
    Int(i32),
    Float(f32),
    Float2([f32;2]),
    Float3([f32;3]),
    Float4([f32;4]),
    Mat3([[f32;3];3]),
    Mat4([[f32;4];4]),
    Sampler2D(u32),
}
impl ShaderUniform {
    fn get_type (&self) -> UniformType {
        match self {
            ShaderUniform::Int(_) => UniformType::Int,
            ShaderUniform::Float(_) => UniformType::Float,
            ShaderUniform::Float2(_) => UniformType::Float2,
            ShaderUniform::Float3(_) => UniformType::Float3,
            ShaderUniform::Float4(_) => UniformType::Float4,
            ShaderUniform::Mat3(_) => UniformType::Mat3,
            ShaderUniform::Mat4(_) => UniformType::Mat4,
            ShaderUniform::Sampler2D(_) => UniformType::Sampler2D,
            _ => UniformType::Invalid,
        }
    }

    fn from_uniform_type(uniform_type : UniformType, default : String) -> Self{
        match uniform_type {
            UniformType::Int => {
                let default_value = default.parse::<i32>().unwrap_or(0);
                ShaderUniform::Int(default_value)
            },
            UniformType::Float => {
                let default_value = default.parse::<f32>().unwrap_or(0.0);
                ShaderUniform::Float(default_value)
            },
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
            UniformType::Invalid => ShaderUniform::Invalid,
            UniformType::Sampler2D => ShaderUniform::Sampler2D(0),
            // _ => panic!("Unimplemented uniform type found!")
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
    Mat4,
    Sampler2D,
    Invalid
}


type ShaderUniformLoc = (GLint, ShaderUniform);
pub struct Pipeline {
    program : GLuint,

    // Graphics pipeline properties
    depth_test : bool,
    blend_enabled : bool,
    primitive_topology : PrimitiveTopology,

    // Collection of shader uniforms found when creating the pipeline
    uniforms : RefCell<HashMap<(String, UniformType), GLint>>,

    // Overrides for each uniform
    uniform_overrides : RefCell<HashMap<(String, UniformType), ShaderUniform>>
}

impl Pipeline {
    pub fn primitive_topology(&self) -> PrimitiveTopology { self.primitive_topology }
    pub fn depth_test(&self) -> bool { self.depth_test }
    pub fn blend_enabled(&self) -> bool { self.blend_enabled }
    pub fn program(&self) -> GLuint { self.program }

    pub fn set_uniform(&self, name : &str, uniform : ShaderUniform) {
        let uniform_type = uniform.get_type();
        let mut uniforms = self.uniform_overrides.borrow_mut();
        let k = (name.to_string(), uniform_type);
        let exist_entry = self.uniforms.borrow_mut().contains_key(&k);
        if exist_entry {
            uniforms.entry(k)
                .and_modify(|e| *e = uniform)
                .or_insert(uniform);
        }
        else 
        {
            println!("Could not find uniform in the shader.");
        }
    }

    /// Uploads all bound uniforms to the GPU
    pub fn flush(&self) {
        unsafe{
            gl::UseProgram(self.program);
        }

        // Flushes all uniforms to the GPU
        for (key, value) in self.uniform_overrides.borrow_mut().iter() {
            if let Some(v) = self.uniforms.borrow().get(key) {
                let shader_loc = *v;
                unsafe{
                    if shader_loc != -1 {
                        // Upload depending on shader uniform type
                        match value {
                            ShaderUniform::Int(v) => gl::Uniform1iv(shader_loc, 1, v),
                            ShaderUniform::Float(v) => gl::Uniform1fv(shader_loc, 1, v),
                            ShaderUniform::Float3(v) => gl::Uniform3fv(shader_loc, 1,v.as_ptr()),
                            ShaderUniform::Float4(v) => gl::Uniform4fv(shader_loc, 1, v.as_ptr()),
                            ShaderUniform::Mat3(v) => gl::UniformMatrix3fv(shader_loc, 1, gl::FALSE, v[0].as_ptr()),
                            ShaderUniform::Mat4(v) => gl::UniformMatrix4fv(shader_loc, 1, gl::FALSE, v[0].as_ptr()),
                            ShaderUniform::Sampler2D(tex) => {
                                // gl::ActiveTexture(gl::TEXTURE0 + shader_loc as u32);
                                if *tex != 0 {
                                    gl::BindTextureUnit(shader_loc as u32, *tex as u32);
                                }
                            }
                            _ => {
                                println!("Unimplemented uniform while flushing...");
                            }
                        }
                    }
                }
            } else {
                println!("Could not find uniform for \"{}\"", key.0);
            }
        }
    }

    pub fn create_simple_with_geom(vertex_source : &[u8], geom_source : &[u8], fragment_source : &[u8]) -> Result<Pipeline, String> {
        use std::ffi::CString;
        let vertex_shader   = shaders::shader_from_source(&CString::new(vertex_source).unwrap(), gl::VERTEX_SHADER)?;
        let geom_shader = shaders::shader_from_source(&CString::new(geom_source).unwrap(), gl::GEOMETRY_SHADER)?;
        let fragment_shader = shaders::shader_from_source(&CString::new(fragment_source).unwrap(), gl::FRAGMENT_SHADER)?;

        let mut uniforms = HashMap::new();
        parse_uniforms(vertex_source, &mut uniforms);
        parse_uniforms(geom_source, &mut uniforms);
        parse_uniforms(fragment_source, &mut uniforms);
        for (key,value) in &uniforms {
        println!("{:?}", key.0);
        }

        let program = create_simple_program(vertex_shader, fragment_shader, Some(geom_shader))?;

        // Find locations for all uniforms
        for (key,mut value) in &mut uniforms {
            unsafe{
                let shader_loc = gl::GetUniformLocation(program, CString::from_vec_unchecked(key.0.as_bytes().to_vec()).as_ptr());
                *value = shader_loc;
            }
        }

        Ok( 
            Pipeline{
                blend_enabled: false,
                depth_test: true,
                primitive_topology: PrimitiveTopology::Triangles,
                program,
                uniforms: RefCell::new(uniforms),
                uniform_overrides: RefCell::new(HashMap::new())
            }
        )
    }

    pub fn create_simple(vertex_source : &[u8], fragment_source : &[u8]) -> Result<Pipeline, String> {
        use std::ffi::CString;

        let vertex_shader   = shaders::shader_from_source(&CString::new(vertex_source).unwrap(), gl::VERTEX_SHADER)?;
        let fragment_shader = shaders::shader_from_source(&CString::new(fragment_source).unwrap(), gl::FRAGMENT_SHADER)?;


        let mut uniforms = HashMap::new();
        parse_uniforms(vertex_source, &mut uniforms);
        parse_uniforms(fragment_source, &mut uniforms);


        println!("Found {} uniforms.", uniforms.len());

        let program = create_simple_program(vertex_shader, fragment_shader, None)?;

        // Find locations for all uniforms
        for (key,mut value) in &mut uniforms {
            unsafe{
                let shader_loc = gl::GetUniformLocation(program, CString::from_vec_unchecked(key.0.as_bytes().to_vec()).as_ptr());
                *value = shader_loc;
            }
        }

        Ok( 
            Pipeline{
                blend_enabled: false,
                depth_test: true,
                primitive_topology: PrimitiveTopology::Triangles,
                program,
                uniforms: RefCell::new(uniforms),
                uniform_overrides: RefCell::new(HashMap::new())
            }
        )
    }

}

/// Creates a simple program containing vertex and fragment shader
pub fn create_simple_program( vertex_shader : GLuint, fragment_shader : GLuint, geom : Option<GLuint>) -> Result<GLuint, String> {
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
        if let Some(geom_shader) = geom {
            gl::AttachShader(program, geom_shader);
        }
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
/// Simple uniform parsing of a source string (for openGL, does not allow layout bindings yet) 
fn parse_uniforms(source : &[u8], result : &mut HashMap<(String, UniformType), GLint>){
    // Construct the regex
    let uniform_regex = Regex::new(r"(uniform)\s(?P<type>\w*)\s(?P<var>\w*)(\s?=\s?(?P<default>.*))?;").expect("Failed to create regex!");

    // Convert our input data to a string
    let c =  String::from_utf8(source.to_vec()).unwrap();

    // Execute the regex on our source data
    let results = uniform_regex.captures_iter(&c);
    for matches in results {
        let var_name = &matches["var"];
        let var_type = match &matches["type"] {
            "int" => UniformType::Int,
            "float" => UniformType::Float,
            "vec2" => UniformType::Float2,
            "vec3" => UniformType::Float3,
            "vec4" => UniformType::Float4,
            "mat3" => UniformType::Mat3,
            "mat4" => UniformType::Mat4,
            "sampler2D" => UniformType::Sampler2D,
            _ => UniformType::Invalid,
        };

        let var_default_value = if let Some(v) = matches.get(4) { 
            ShaderUniform::from_uniform_type(var_type, v.as_str().to_string()) 
        } else {
            ShaderUniform::from_uniform_type(var_type, String::new())
        };

        let result_key = (var_name.to_string(), var_type);
        result.entry(result_key).or_insert(-1);
    }
}
