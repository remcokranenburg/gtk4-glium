use glium::backend::Facade;
use glium::index::{NoIndices, PrimitiveType};
use glium::texture::{CompressedTexture2d, RawImage2d};
use glium::{implement_vertex, program, uniform, Frame, Program, Surface, VertexBuffer};
use gtk4::prelude::*;
use gtk4::{glib::signal::Propagation, Application, ApplicationWindow, GLArea};
use gtk4_glium::GtkFacade;
use std::io::Cursor;
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct TexVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(TexVertex, position, tex_coords);

fn create_rectangle_buffer<F: Facade>(context: &F) -> VertexBuffer<TexVertex> {
    glium::VertexBuffer::new(
        context,
        &[
            TexVertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            TexVertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            TexVertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            TexVertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
        ],
    )
    .unwrap()
}

fn load_texture<F: Facade>(context: &F) -> CompressedTexture2d {
    let image = image::load(
        Cursor::new(&include_bytes!("opengl.png")[..]),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();

    let image_dimensions = dbg!(image.dimensions());
    let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

    CompressedTexture2d::new(context, image).unwrap()
}

fn create_program<F: Facade>(context: &F) -> Program {
    program!(context,
        140 => {
            vertex: "
                #version 140

                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;

                out vec2 v_tex_coords;

                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140

                uniform sampler2D tex;
                in vec2 v_tex_coords;

                out vec4 f_color;

                void main() {
                    f_color = texture(tex, v_tex_coords);

                    // Just setting the color draws a rectangle.
                    // So everything in the setup seems to work, except sampling the texture
                    // f_color = vec4(0.5, 0.1, 0.2, 1.0);
                }
            "
        },
    )
    .unwrap()
}

fn main() {
    let application = Application::builder()
        .application_id("com.remcokranenburg.Texture")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Texture")
            .default_width(600)
            .default_height(400)
            .build();

        let glarea = GLArea::builder().vexpand(true).build();

        window.set_child(Some(&glarea));
        window.show();

        let facade = GtkFacade::from_glarea(&glarea).unwrap();

        let opengl_texture = load_texture(&facade);
        let vertex_buffer = create_rectangle_buffer(&facade);
        let program = create_program(&facade);

        glarea.connect_render(move |_glarea, _glcontext| {
            let context = facade.get_context();

            let uniforms = uniform! {
                matrix: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &opengl_texture
            };

            let mut frame = Frame::new(context.clone(), context.get_framebuffer_dimensions());

            frame.clear_color(0.0, 0.0, 0.0, 1.0);

            frame
                .draw(
                    &vertex_buffer,
                    NoIndices(PrimitiveType::TriangleStrip),
                    &program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();

            frame.finish().unwrap();
            Propagation::Proceed
        });

        // This makes the GLArea redraw 60 times per second
        // You can remove this if you want to redraw only when focused/resized
        let frame_time = Duration::new(0, 1_000_000_000 / 60);
        glib::source::timeout_add_local(frame_time, move || {
            glarea.queue_draw();
            glib::ControlFlow::Continue
        });
    });

    application.run();
}
