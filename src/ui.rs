use crate::app::App;
use winit::keyboard::PhysicalKey::Code;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowBuilder,
};

pub async fn render(speed: u8) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut app = App::new_with_window(&window, speed).await;

    event_loop.set_control_flow(ControlFlow::Wait);

    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
                WindowEvent::RedrawRequested => {
                    app.update();
                    match app.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            let size = app.surface.as_ref().unwrap().surface_size;
                            app.resize(size);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state.is_pressed() && event.physical_key == Code(KeyCode::Escape) {
                        elwt.exit()
                    }
                }

                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size);
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    app.resize(window.inner_size());
                }
                _ => {}
            },
            Event::AboutToWait => {
                app.window().request_redraw();
            }
            _ => {}
        })
        .unwrap();
}

pub async fn run(speed: u8, filename: &str, resolution: [u16; 2]) {
    let mut app = App::new_without_window(speed, resolution).await;
    let mut frames: Vec<Vec<u8>> = Vec::new();
    for _i in 1..60 {
        app.update();
        app.run(&mut frames).await;
    }
    save_gif(filename, &mut frames, 15, resolution[0], resolution[1]).unwrap();
}

fn save_gif(
    path: &str,
    frames: &mut Vec<Vec<u8>>,
    speed: i32,
    width: u16,
    height: u16,
) -> anyhow::Result<()> {
    use gif::{Encoder, Frame, Repeat};

    let mut image = std::fs::File::create(path)?;
    let mut encoder = Encoder::new(&mut image, width, height, &[])?;
    encoder.set_repeat(Repeat::Infinite)?;

    for frame in frames {
        encoder.write_frame(&Frame::from_rgba_speed(width, height, frame, speed))?;
    }

    Ok(())
}
