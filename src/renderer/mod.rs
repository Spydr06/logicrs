pub mod prelude;

use gl::{
    implement_vertex, index::PrimitiveType, program, uniform, Frame, IndexBuffer, Surface,
    VertexBuffer,
};

use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

implement_vertex!(Vertex, position, color);

pub struct Renderer {
    context: Rc<gl::backend::Context>,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    program: gl::Program,
}

impl Renderer {
    pub fn new(context: Rc<gl::backend::Context>) -> Self {
        let vertex_buffer = VertexBuffer::new(
            &context,
            &[
                Vertex {
                    position: [-0.5, -0.5],
                    color: [0., 1., 0.],
                },
                Vertex {
                    position: [0., 0.5],
                    color: [0., 0., 1.],
                },
                Vertex {
                    position: [0.5, -0.5],
                    color: [1., 0., 0.],
                },
            ],
        )
        .unwrap();
        let index_buffer =
            IndexBuffer::new(&context, PrimitiveType::TrianglesList, &[0u16, 1, 2]).unwrap();
        let program = program!(&context, 140 => {
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
        },
        )
        .unwrap();

        Renderer {
            context,
            vertex_buffer,
            index_buffer,
            program,
        }
    }

    pub fn draw(&self) {
        let mut frame = Frame::new(
            self.context.clone(),
            self.context.get_framebuffer_dimensions(),
        );

        let uniforms = uniform! {
            matrix: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1f32]
            ]
        };

        frame.clear_color(0., 0., 0., 0.);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();
    }
}
