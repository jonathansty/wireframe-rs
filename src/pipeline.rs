use regex::Regex;

use crate::shaders;
use crate::helpers;

use gl::types::*;
use std::collections::HashMap;

/// Available shader stages that are implemented
enum ShaderStage {
    Vertex,
    Fragment,
    Geometry,
    // Compute,
}

/// Shader uniforms are parsed from input source and can be different types
enum ShaderUniform{
    Float1([f32;1]),
    Float2([f32;2]),
    Float3([f32;3]),
    Float4([f32;4]),
    Mat3([f32;9]),
    Mat4([f32;16]),
}

pub struct Pipeline {
    program : GLuint,

    // Collection of shader uniforms found when creating the pipeline
    uniforms : HashMap<String, ShaderUniform>
}

impl Pipeline {
    pub fn create_simple(vertex_source : &[u8], fragment_source : &[u8]) -> Result<Pipeline, String> {
        use std::ffi::CString;

        let vertex_shader   = shaders::shader_from_source(&CString::new(vertex_source).unwrap(), gl::VERTEX_SHADER)?;
        let fragment_shader = shaders::shader_from_source(&CString::new(fragment_source).unwrap(), gl::FRAGMENT_SHADER)?;


        let mut uniforms = HashMap::new();
        parse_uniforms(vertex_source, &mut uniforms);
        println!("Found {} uniforms.", uniforms.len());

        let mut program = helpers::create_simple_program(vertex_shader, fragment_shader)?;

        Ok( 
            Pipeline{
                program,
                uniforms
            }
        )
    }

}

/// Simple uniform parsing of a source string (for openGL, does not allow layout bindings yet) 
fn parse_uniforms(source : &[u8], result : &mut HashMap<String, ShaderUniform>){
    // Construct the regex
    let uniform_regex = Regex::new(r"(uniform)\s(?P<type>\w*)\s(?P<var>\w*);").expect("Failed to create regex!");

    // Convert our input data to a string
    let c =  String::from_utf8(source.to_vec()).unwrap();

    // Execute the regex on our source data
    let results = uniform_regex.captures_iter(&c);
    for matches in results {
        let var_name = &matches["var"];
        let var_type = match &matches["type"] {
            "float1" => ShaderUniform::Float1([0.0]),
            "float2" => ShaderUniform::Float2([0.0,0.0]),
            "float3" => ShaderUniform::Float3([0.0,0.0,0.0]),
            "float4" => ShaderUniform::Float4([0.0,0.0,0.0,0.0]),
            "mat3" => ShaderUniform::Mat3(
                [1.0,0.0,0.0,
                0.0,1.0,0.0,
                0.0,0.0,1.0]
            ),
            "mat4" => ShaderUniform::Mat4(
                [1.0,0.0,0.0,0.0,
                0.0,1.0,0.0,0.0,
                0.0,0.0,1.0,0.0,
                0.0,0.0,0.0,1.0]
            ),
            _ => panic!("Unrecognized shader uniform found in shader!")
        };
        result.entry(var_name.to_string()).or_insert(var_type);
    }
}
