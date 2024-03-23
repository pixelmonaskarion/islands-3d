use bespoke_engine::model::{Model, Render};

pub struct Billboard {
    model: Model,
}

impl Billboard {
    pub fn new() -> Self {
        todo!()
    }
}

impl Render for Billboard {
    fn render<'a: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) {
        self.model.render(render_pass);
    }
}