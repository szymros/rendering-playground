use framework::{
    pipelines::{ComputePipeline, RenderPipeline, RenderPipelineBuilder},
    render_context::RenderContext,
};

pub struct EffectPipelines {
    pub emit_pipeline: ComputePipeline,
    pub update_pipeline: ComputePipeline,
    pub render_pipeline: RenderPipeline,
}

impl EffectPipelines {
    pub fn new(
        render_context: &RenderContext,
        effect_name: &'static str,
        max_particles: u32,
        emit_code: String,
        update_code: String,
        render_code: String,
    ) -> EffectPipelines {
        let render_pipeline = RenderPipelineBuilder::new(
            "draw_particles",
            wgpu::ShaderModuleDescriptor {
                label: Some("emit"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(render_code)),
            },
        )
        .bind_groups(&["globals", "render_layout"])
        .render_targets(&[Some(render_context.surface_format.into())])
        .build(&render_context);
        let emit_pipeline = ComputePipeline::new(
            render_context,
            wgpu::ShaderModuleDescriptor {
                label: Some("emit"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(emit_code)),
            },
            "emit_particles",
            &["globals", effect_name],
            (1, 1, 1),
        );
        let update_pipeline = ComputePipeline::new(
            render_context,
            wgpu::ShaderModuleDescriptor {
                label: Some("emit"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(update_code)),
            },
            "update_particles",
            &["globals", effect_name],
            ((max_particles + 15) / 16, 1, 1),
        );
        return EffectPipelines {
            update_pipeline,
            emit_pipeline,
            render_pipeline,
        };
    }
}
