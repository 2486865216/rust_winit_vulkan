use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use my_winit::example::buffer::*;

fn main() {
    operator_buffer();

    //winit
    let event_loop = EventLoop::new();
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
    })
}
