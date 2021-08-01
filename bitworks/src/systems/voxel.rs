use bevy_vox_mesh::VoxMeshPlugin;

use bevy::{reflect::TypeUuid, render::{
    pipeline::{PipelineDescriptor, RenderPipeline},
    render_graph::base::MainPass,
    shader::{ShaderStage, ShaderStages},
}};

use bevy::prelude::*;

pub struct VoxelPlugin;

impl Plugin for VoxelPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(VoxMeshPlugin::default())
            .add_startup_system(setup_voxel_render_pipeline.system());
    }
}

#[derive(Bundle)]
pub struct VoxelBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub main_pass: MainPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub trait UnlitColor {
    fn unlit_color(base_color: Color) -> Self;
}

impl UnlitColor for StandardMaterial {
    fn unlit_color(base_color: Color) -> Self {
        Self {
            base_color,
            unlit: true,
            ..Default::default()
        }
    }
}

const BASIC_COLOR_VERT: &str = r#"#version 450
layout(location = 0) in vec4 Vertex_Position;
layout(location = 1) in vec4 Vertex_Color;
layout(location = 0) out vec4 vertex_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    vec4 model_pos = Model * Vertex_Position;
    gl_Position = ViewProj * model_pos;
    
    vertex_color = vec4(Vertex_Color.rgb, Vertex_Color.a * 0.9);
}"#;

const BASIC_COLOR_FRAG: &str = r#"#version 450
layout(location = 0) in vec4 vertex_color;
layout(location = 0) out vec4 o_Target;
vec4 toLinear(vec4 sRGB)
{
    bvec4 cutoff = lessThan(sRGB, vec4(0.04045));
    vec4 higher = pow((sRGB + vec4(0.055))/vec4(1.055), vec4(2.4));
    vec4 lower = sRGB/vec4(12.92);
    return mix(higher, lower, cutoff);
}
void main() {
    o_Target = toLinear(vertex_color);
}"#;

const VOXEL_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 3107534120020256000);

fn setup_voxel_render_pipeline(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let vertex = Shader::from_glsl(ShaderStage::Vertex, BASIC_COLOR_VERT);
    let fragment = Shader::from_glsl(ShaderStage::Fragment, BASIC_COLOR_FRAG);
    let pipeline = PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(vertex),
        fragment: Some(shaders.add(fragment)),
    });

    pipelines.set_untracked(VOXEL_PIPELINE_HANDLE, pipeline);
}

impl Default for VoxelBundle {
    fn default() -> Self {
        Self {
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                VOXEL_PIPELINE_HANDLE.typed(),
            )]),
            mesh: Default::default(),
            visible: Default::default(),
            material: Default::default(),
            main_pass: Default::default(),
            draw: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
        }
    }
}
