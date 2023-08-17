use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use my_winit::example::buffer::*;
use std::time::{SystemTime, UNIX_EPOCH};
use my_winit::example::compute::operator_computer;
use my_winit::example::graphics_pipeline::operator_vertex;
use my_winit::example::image_shader::operator_image_shader;
use my_winit::example::images::operator_image;

fn main() {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!("{:?}", now);

    operator_buffer();
    operator_computer();
    operator_image();
    operator_image_shader();
    operator_vertex();

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    println!("{:?}", now);

    //winit
    /*let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    })*/
}
/*#[allow(unused)]
fn main() {
mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }
        ",
    }
}
}
*/