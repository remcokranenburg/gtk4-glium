use core::ffi::c_void;
use glium::backend::{Backend, Context, Facade};
use glium::debug::DebugCallbackBehavior;
use glium::IncompatibleOpenGl;
use glium::SwapBuffersError;
use gtk4::prelude::*;
use gtk4::GLArea;
use std::rc::Rc;

struct GLAreaBackend {
    glarea: GLArea,
}

unsafe impl Backend for GLAreaBackend {
    fn swap_buffers(&self) -> Result<(), SwapBuffersError> {
        // GTK swaps the buffers after each "render" signal itself
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const c_void {
        gl_loader::get_proc_address(symbol) as *const _
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let width = self.glarea.width();
        let height = self.glarea.height();

        // On high-resolution screens, the number of pixels in the frame buffer
        // is higher than the allocation. This is indicated by the scale
        // factor.
        let scale = self.glarea.scale_factor();

        ((width * scale) as u32, (height * scale) as u32)
    }

    fn resize(&self, _: (u32, u32)) {
        // do nothing
    }

    fn is_current(&self) -> bool {
        // GTK makes OpenGL current itself on each "render" signal
        true
    }

    unsafe fn make_current(&self) {
        self.glarea.make_current();
    }
}

impl GLAreaBackend {
    fn new(glarea: GLArea) -> Self {
        Self { glarea }
    }
}

pub struct GtkFacade {
    context: Rc<Context>,
}

impl GtkFacade {
    pub fn from_glarea(glarea: &GLArea) -> Result<Self, IncompatibleOpenGl> {
        gl_loader::init_gl();

        let context = unsafe { Context::new(
            GLAreaBackend::new(glarea.clone()),
            true,
            DebugCallbackBehavior::DebugMessageOnError,
        ) }?;

        Ok(Self {
            context: context,
        })
    }
}

impl Facade for GtkFacade {
    fn get_context(&self) -> &Rc<Context> {
        &self.context
    }
}

