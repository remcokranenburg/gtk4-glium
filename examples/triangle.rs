use gtk4_glium::GtkFacade;
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::{
    Frame,
    IndexBuffer,
    Program,
    VertexBuffer,
    Surface,
    implement_vertex,
    program,
    uniform,
};
use gtk4::prelude::*;
use gtk4::{
    Application,
    ApplicationWindow,
    Box,
    GLArea,
    Inhibit,
    Orientation,
    Scale,
};
use std::time::Duration;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

fn create_triangle_buffer<F>(display: &F) -> VertexBuffer<Vertex>
    where F: Facade {

    VertexBuffer::new(display,
        &[
            Vertex { position: [-1.0, -1.0], color: [0.0, 1.0, 0.0] },
            Vertex { position: [ 0.0,  1.0], color: [0.0, 0.0, 1.0] },
            Vertex { position: [ 1.0, -1.0], color: [1.0, 0.0, 0.0] },
        ]
    ).unwrap()
}

fn create_program<F>(display: &F) -> Program
        where F: Facade {
    program!(display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec3 color;
                out vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",
            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        }
    ).unwrap()
}

fn main() {
    let application = Application::builder()
        .application_id("com.remcokranenburg.Triangle")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Triangle")
            .default_width(600)
            .default_height(400)
            .build();

        let rows = Box::new(Orientation::Vertical, 0);

        let glarea = GLArea::builder()
            .vexpand(true)
            .build();

        let rotation = Scale::with_range(Orientation::Horizontal, -10.0, 10.0, 0.1);
        rotation.set_value(0.0);

        rows.append(&glarea);
        rows.append(&rotation);
        window.set_child(Some(&rows));
        window.show();

        let facade = GtkFacade::from_glarea(&glarea).unwrap();

        let vertex_buffer = create_triangle_buffer(&facade);
        let index_buffer = IndexBuffer::new(&facade, PrimitiveType::TrianglesList,
            &[0u16, 1, 2]).unwrap();
        let program = create_program(&facade);

        glarea.connect_render(move |_glarea, _glcontext| {
            let r = rotation.value() as f32;

            let uniforms = uniform! {
                matrix: [
                    [r.cos(), -r.sin(), 0.0, 0.1 * r],
                    [r.sin(),  r.cos(), 0.0, 0.0],
                    [    0.0,      0.0, 1.0, 0.0],
                    [    0.0,      0.0, 0.0, 1.0f32]
                ]
            };

            let context = facade.get_context();
            let mut frame = Frame::new(context.clone(), context.get_framebuffer_dimensions());

            frame.clear_color(0.0, 0.0, 0.0, 1.0);
            frame.draw(&vertex_buffer, &index_buffer, &program, &uniforms,
                &Default::default()).unwrap();

            frame.finish().unwrap();
            Inhibit(true)
        });

        // This makes the GLArea redraw 60 times per second
        // You can remove this if you want to redraw only when focused/resized
        let frame_time = Duration::new(0, 1_000_000_000 / 60);
        glib::source::timeout_add_local(frame_time, move || {
            glarea.queue_draw();
            glib::source::Continue(true)
        });
    });

    application.run();
}
