mod math;
mod geometry;
mod loader;
mod renderer;
mod camera;
mod input;
mod app;

use app::App;

use pixels::{Pixels, SurfaceTexture};

use winit::{
    event::{
        DeviceEvent,
        Event,
    },
    event_loop::{
        ControlFlow,
        EventLoop,
    },
    window::{
        CursorGrabMode,
        WindowBuilder,
    },
};

fn main() {
    let width = 800;
    let height = 600;

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("3D Demo")
        .with_inner_size(
            winit::dpi::LogicalSize::new(
                width,
                height,
            )
        )
        .build(&event_loop)
        .unwrap();

    window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| {
            window.set_cursor_grab(
                CursorGrabMode::Confined
            )
        })
        .ok();

    window.set_cursor_visible(false);

    let surface_texture =
        SurfaceTexture::new(width, height, &window);

    let mut pixels =
        Pixels::new(
            width,
            height,
            surface_texture,
        )
        .unwrap();

    let mut app = App::new(width, height);

    event_loop.run(
        move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::DeviceEvent {
                    event,
                    ..
                } => {
                    app.handle_device_event(&event);
                }

                Event::WindowEvent {
                    event,
                    ..
                } => {
                    if app.handle_window_event(&event) {
                        *control_flow =
                            ControlFlow::Exit;
                    }
                }

                Event::MainEventsCleared => {
                    app.update();

                    window.request_redraw();
                }

                Event::RedrawRequested(_) => {
                    let frame =
                        pixels.frame_mut();

                    app.render(frame);

                    pixels.render().unwrap();
                }

                _ => {}
            }
        },
    );
}