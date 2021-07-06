use std::borrow::Cow;

use amethyst_assets::{
    DefaultLoader,
    Handle,
    Loader,
    ProcessingQueue,
};
use amethyst_rendy::{
    batch::OrderedOneLevelBatch,
    pipeline::{
        PipelineDescBuilder,
        PipelinesBuilder,
    },
    rendy::{
        command::{
            QueueId,
            RenderPassEncoder,
        },
        graph::{
            render::{
                PrepareResult,
                RenderGroup,
            },
            GraphContext,
            NodeBuffer,
            NodeImage,
        },
        hal::{
            device::{
                Device,
                ShaderError,
            },
            image::Layout,
            pass::Subpass,
            pso::{
                BlendState,
                ColorBlendDesc,
                ColorMask,
                CreationError,
                DepthTest,
                InputAssemblerDesc,
                Primitive,
                ShaderStageFlags,
                VertexInputRate,
            },
        },
        mesh::AsVertex,
        resource::{
            Filter,
            Lod,
            PackedColor,
            SamplerDesc,
            ViewKind,
            WrapMode,
        },
        shader::{
            ShaderSetBuilder,
            SpirvShader,
        },
        texture::TextureBuilder,
    },
    submodules::{
        DynamicUniform,
        DynamicVertexBuffer,
        TextureId,
        TextureSub,
    },
    system::GraphAuxData,
    types::TextureData,
    Backend,
    ChangeDetection,
    Factory,
    Format,
    Kind,
    RenderGroupDesc,
    Texture,
};
use amethyst_window::ScreenDimensions;
use egui::{
    ClippedMesh,
    Color32,
};
use glsl_layout::Uniform;

use crate::{
    pod::{
        EguiArgs,
        EguiViewArgs,
    },
    system::{
        EguiContext,
        EguiStage,
    },
};

lazy_static::lazy_static! {
    static ref EGUI_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../shaders/compiled/egui.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref EGUI_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../shaders/compiled/egui.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();

    static ref EGUI_SHADERS: ShaderSetBuilder = ShaderSetBuilder::default()
        .with_vertex(&*EGUI_VERTEX)
        .unwrap()
        .with_fragment(&*EGUI_FRAGMENT)
        .unwrap();
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawEguiDesc;

impl<B: Backend> RenderGroupDesc<B, GraphAuxData> for DrawEguiDesc {
    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, CreationError> {
        let env = DynamicUniform::new(factory, ShaderStageFlags::VERTEX)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_egui_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawEgui::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            change: Default::default(),
            batch: Default::default(),
            egui_texture: None,
        }))
    }
}

#[derive(Debug)]
pub struct DrawEgui<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, EguiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, EguiArgs>,
    batch: OrderedOneLevelBatch<TextureId, EguiArgs>,
    change: ChangeDetection,
    egui_texture: Option<(Handle<Texture>, u64)>,
}

impl<B: Backend> DrawEgui<B> {
    fn upload_egui_texture(&mut self, texture: &egui::Texture, aux: &GraphAuxData) {
        if self
            .egui_texture
            .as_ref()
            .map(|(_, version)| *version != texture.version)
            .unwrap_or(true)
        {
            log::debug!("Egui texture changed: new_version={}", texture.version);

            let loader = aux
                .resources
                .get::<DefaultLoader>()
                .expect("default loader");
            let texture_storage = aux
                .resources
                .get::<ProcessingQueue<TextureData>>()
                .expect("texture storage");

            let texture_data = convert_into_amethyst_texture(texture);
            //let texture_data = load_from_linear_rgba(LinSrgba::new(1.0, 0.0, 0.0,
            // 1.0)).into();

            let handle = loader.load_from_data(texture_data, (), &texture_storage);
            self.egui_texture = Some((handle, texture.version));
        }
    }
}

fn convert_into_amethyst_texture(texture: &egui::Texture) -> TextureData {
    let mut b = TextureBuilder::new();
    b.set_data_width(texture.width as u32);
    b.set_data_height(texture.height as u32);
    b.set_kind(Kind::D2(texture.width as u32, texture.height as u32, 1, 1));
    b.set_view_kind(ViewKind::D2);
    b.set_sampler_info(SamplerDesc {
        min_filter: Filter::Linear,
        mag_filter: Filter::Linear,
        mip_filter: Filter::Linear,
        wrap_mode: (WrapMode::Clamp, WrapMode::Clamp, WrapMode::Clamp),
        lod_bias: Lod(0.0),
        lod_range: std::ops::Range {
            start: Lod(0.0),
            end: Lod(1000.0),
        },
        comparison: None,
        border: PackedColor(0),
        normalized: true,
        anisotropy_clamp: None,
    });

    let mut pixels = vec![];
    for a in &texture.pixels {
        let t = Color32::from_white_alpha(*a).to_tuple();
        pixels.push(t.0);
        pixels.push(t.1);
        pixels.push(t.2);
        pixels.push(t.3);
    }
    b.set_raw_data(Cow::Owned(pixels), Format::Rgba8Srgb);

    TextureData(b)
}

impl<B: Backend> RenderGroup<B, GraphAuxData> for DrawEgui<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult {
        let mut egui_ctx = aux
            .resources
            .get_mut::<EguiContext>()
            .expect("EguiState resource missing");

        if !matches!(&egui_ctx.stage, EguiStage::Render) {
            log::error!("DrawEgui::prepare called with invalid EguiState");
            return self.change.prepare_result(index, false);
        }

        let mut changed = false;

        self.upload_egui_texture(&egui_ctx.ctx.texture(), aux);

        let screen_dimensions = aux.resources.get::<ScreenDimensions>().unwrap();

        let (egui_output, shapes) = egui_ctx.ctx.end_frame();
        egui_ctx.stage = EguiStage::End(egui_output);
        let clipped_meshes = egui_ctx.ctx.tessellate(shapes);

        let textures_ref = &mut self.textures;
        let batch_ref = &mut self.batch;
        batch_ref.swap_clear();

        for ClippedMesh(_clip_rect, mesh) in clipped_meshes {
            let texture = match &mesh.texture_id {
                egui::epaint::TextureId::Egui => {
                    self.egui_texture.as_ref().map(|(handle, _)| handle)
                }
                egui::epaint::TextureId::User(id) => egui_ctx.user_textures.get(id),
            };

            if let Some((tex_id, this_changed)) = texture.and_then(|texture| {
                textures_ref.insert(
                    factory,
                    aux.resources,
                    texture,
                    Layout::ShaderReadOnlyOptimal,
                )
            }) {
                changed = changed || this_changed;

                let vertices = mesh.vertices;
                //log::debug!("num vertices: {}", vertices.len());
                batch_ref.insert(
                    tex_id,
                    mesh.indices.into_iter().map(|index| {
                        let vertex = vertices[index as usize];
                        //log::debug!("{:?}", vertex);
                        EguiArgs::new(vertex)
                    }),
                );
            }
            else {
                log::error!("Texture missing: {:?}", texture);
            }
        }

        self.textures.maintain(factory, aux.resources);
        changed = changed || self.batch.changed();

        {
            self.vertex.write(
                factory,
                index,
                self.batch.count() as u64,
                Some(self.batch.data()),
            );

            let view_args = EguiViewArgs::new(egui::Rect::EVERYTHING, &screen_dimensions);
            changed = self.env.write(factory, index, view_args.std140()) || changed;
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: Subpass<'_, B>,
        _resources: &GraphAuxData,
    ) {
        encoder.bind_graphics_pipeline(&self.pipeline);

        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        //self.index.bind(index, 0, &mut encoder);

        for (&tex, range) in self.batch.iter() {
            self.textures
                .bind(&self.pipeline_layout, 1, tex, &mut encoder);
            unsafe {
                encoder.draw(range, 0..1);
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &GraphAuxData) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_egui_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), CreationError> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let mut shaders = EGUI_SHADERS
        .build(factory, Default::default())
        .map_err(|e| {
            match e {
                ShaderError::OutOfMemory(oom) => oom.into(),
                _ => CreationError::Other,
            }
        })?;

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(EguiArgs::vertex(), VertexInputRate::Vertex)])
                .with_input_assembler(InputAssemblerDesc::new(Primitive::TriangleList))
                //.with_rasterizer(Rasterizer::FILL)
                .with_shaders(shaders.raw().map_err(|_| CreationError::Other)?)
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![ColorBlendDesc {
                    mask: ColorMask::ALL,
                    blend: Some(BlendState::PREMULTIPLIED_ALPHA),
                }])
                /*.with_baked_states(BakedStates {
                    viewport: Some(Viewport {
                        rect: Rect {
                            x: 0,
                            y: 0,
                            w: framebuffer_width as i16,
                            h: framebuffer_height as i16,
                        },
                        depth: 0.0 .. 1.0,
                    }),
                    scissor: None,
                    ..Default::default()
                })*/
                .with_depth_test(DepthTest::PASS_TEST),
        )
        .build(factory, None);

    shaders.dispose(factory);

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}
