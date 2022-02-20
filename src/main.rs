extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::CStr;
use gl::types::*;


fn main() {
    //GLSL source of shader programs:
    let vert_source = "#version 330 core

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inColor;
out vec3 fragColor;
void main()
{
    gl_Position = vec4(inPosition, 1.0);
    fragColor = inColor;
}\0";

    let frag_source = "#version 330 core
out vec4 Color;
in vec3 fragColor;

vec3 gammaCorrect(vec3 c)
{
    c = pow(c, vec3(1.0/2.2));
    return c;
}

void main()
{
    Color = vec4(gammaCorrect(fragColor), 1.0f);
}\0";

    //Init SDL2
    let sdl_context = sdl2::init().expect("Failed to SDL_Init()");
    let video_subsystem = sdl_context.video().expect("Cannot acquire video_subsystem from SDL2");
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);
    gl_attr.set_framebuffer_srgb_compatible(true);
    //gl_attr.set_doublebuffer(true);

    let window = video_subsystem
        .window("test", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .expect("Cannot create window");

    let _gl_context = window.gl_create_context().expect("Failed to create GL context");
    window.gl_set_context_to_current().expect("Failed to make GL context current");

    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);
    gl::Viewport::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    let vertices: Vec<f32> = vec![
        //X Y Z R G B = 6 floats. XYZ offset = 0, RGB offset = 3
        -0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
        0.0, 0.5, 0.0, 0.0, 0.0, 1.0,
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2
    ];

    let mut vao : gl::types::GLuint = 0;
    let mut vbo : gl::types::GLuint = 0;
    let mut ebo : gl::types::GLuint = 0;
    let program : gl::types::GLuint;

    unsafe
    {
        // Upload Vertex buffer
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, 
            vertices.as_ptr() as * const gl::types::GLvoid, 
            gl::STATIC_DRAW);

        // Upload Index buffer
        gl::GenBuffers(1, &mut ebo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
            indices.as_ptr() as * const gl::types::GLvoid, 
            gl::STATIC_DRAW);

        // Configure Vertex Array Object
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Shader input location 0 is vector 3d position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (6 * std::mem::size_of::<f32>()) as gl::types::GLint, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        // Shader input location 1 is vector 3d per-vertex color
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, (6 * std::mem::size_of::<f32>()) as gl::types::GLint, (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid);
        gl::EnableVertexAttribArray(1);

        // Will use index baded draw command
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BindVertexArray(0);

        //Build GPU Program
        program = gl::CreateProgram();
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        let vertex_source_ptr : *const u8 = vert_source.as_bytes().as_ptr();
        let fragment_source_ptr : *const u8 = frag_source.as_bytes().as_ptr();
        let vertex_source_ptr_i8 : *const i8 = std::mem::transmute(vertex_source_ptr); //array of u8 to array of i8
        let fragment_source_ptr_i8 : *const i8 = std::mem::transmute(fragment_source_ptr);
        //I remember being annoyed by the C funciton glShaderSource() signature when doing it in C++, but here the pointer conversion make it next level annoying
        gl::ShaderSource(vertex_shader, 1, &(vertex_source_ptr_i8) as *const *const gl::types::GLchar, std::ptr::null());
        gl::ShaderSource(fragment_shader, 1, &(fragment_source_ptr_i8) as *const *const gl::types::GLchar, std::ptr::null());
        gl::CompileShader(vertex_shader);
        gl::CompileShader(fragment_shader);

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
    
        let vendor_ptr = gl::GetString(gl::VENDOR) as *const GLchar; 
        let renderer_ptr = gl::GetString(gl::RENDERER) as *const GLchar;

        let vendor = CStr::from_ptr(vendor_ptr);
        println!("GL_VENDOR = {}", vendor.to_string_lossy());
        let renderer = CStr::from_ptr(renderer_ptr);
        println!("GL_RENDERER = {}", renderer.to_string_lossy());
    }

    let mut event_pump = sdl_context.event_pump().expect("Failed to acquire message pump");

    'running: loop
    {

        //Window message pump
        for event in event_pump.poll_iter()
        {
            match event 
            {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        //3D render
        unsafe 
        {
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, indices.len() as gl::types::GLsizei, gl::UNSIGNED_INT, std::ptr::null());
        }

        window.gl_swap_window();
    }
}
