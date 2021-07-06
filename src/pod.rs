use amethyst_core::math::{
    Matrix4,
    Vector3,
};
use amethyst_rendy::{
    rendy::mesh::{
        AsVertex,
        VertexFormat,
    },
    Format,
};
use amethyst_window::ScreenDimensions;
use egui::epaint;
use glsl_layout::{
    mat4,
    vec2,
    vec4,
    Uniform,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Uniform)]
#[repr(C, align(16))]
pub(crate) struct EguiArgs {
    pub pos: vec2,
    pub uv: vec2,
    pub color: vec4,
}

impl EguiArgs {
    pub fn new(vertex: epaint::Vertex) -> Self {
        //let color = epaint::Rgba::from(vertex.color);
        //log::debug!("{:?}", vertex.uv)
        Self {
            pos: [vertex.pos.x, vertex.pos.y].into(),
            uv: [vertex.uv.x, vertex.uv.y].into(),
            //color: [color.r(), color.g(), color.b(), color.a()].into(),
            color: [
                vertex.color.r() as f32,
                vertex.color.g() as f32,
                vertex.color.b() as f32,
                vertex.color.a() as f32,
            ]
            .into(),
        }
    }
}

impl AsVertex for EguiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "pos"),
            (Format::Rg32Sfloat, "uv"),
            (Format::Rgba32Sfloat, "color"),
        ))
    }
}

#[derive(Clone, Copy, Debug, Uniform)]
#[repr(C, align(16))]
pub(crate) struct EguiViewArgs {
    pub clip_rect: vec4,
    pub view: mat4,
}

impl EguiViewArgs {
    pub fn new(clip_rect: egui::Rect, screen_dimensions: &ScreenDimensions) -> Self {
        let mut view = Matrix4::identity();
        view.append_nonuniform_scaling_mut(&Vector3::new(
            2.0 / screen_dimensions.width() as f32,
            2.0 / screen_dimensions.height() as f32,
            1.0,
        ));
        view.append_translation_mut(&Vector3::new(-1.0, -1.0, 0.0));
        let view: [[f32; 4]; 4] = view.into();

        Self {
            clip_rect: [
                clip_rect.min.x,
                clip_rect.min.y,
                clip_rect.max.x,
                clip_rect.max.y,
            ]
            .into(),
            view: view.into(),
        }
    }
}
