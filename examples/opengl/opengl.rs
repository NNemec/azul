extern crate azul;

use azul::prelude::*;

const CSS: &str = "
    texture {
        width: 100%;
        height: 100%;
        border: 4px solid green;
        box-sizing: border-box;
    }
";

struct OpenGlAppState { }

impl Layout for OpenGlAppState {
    fn layout(&self, _info: LayoutInfo<Self>) -> Dom<Self> {
        Dom::gl_texture(GlTextureCallback(render_my_texture), StackCheckedPointer::new(self, self).unwrap())
    }
}

fn render_my_texture(
    _state: &StackCheckedPointer<OpenGlAppState>,
    info: LayoutInfo<OpenGlAppState>,
    hi_dpi_bounds: HidpiAdjustedBounds
) -> Texture {

    println!("rendering opengl state!");

    let physical_size = hi_dpi_bounds.get_physical_size();

    let gl_context = info.window.get_gl_context();
    let mut texture = Texture::new(
        gl_context.clone(),
        physical_size.width as usize,
        physical_size.height as usize
    );

    {
        let mut fb = FrameBuffer::new(&mut texture);
        fb.bind();
        gl_context.clear_color(0.0, 1.0, 0.0, 1.0);
        fb.unbind();
        fb.finish();
    }

    texture
}

fn main() {
    let mut app = App::new(OpenGlAppState { }, AppConfig::default()).unwrap();
    let css = css::override_native(CSS).unwrap();
    let window = app.create_window(WindowCreateOptions::default(), css).unwrap();
    app.run(window).unwrap();
}