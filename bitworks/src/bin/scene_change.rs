use bevy::render::{
    pipeline::{PipelineDescriptor, RenderPipeline},
    shader::{ShaderStage, ShaderStages},
};
use bevy_inspector_egui::InspectableRegistry;
use bevy_vox_mesh::VoxMeshPlugin;

use bitworks::*;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        //.add_plugin(CameraPlugin)
        .add_plugin(VoxMeshPlugin::default())
        .add_plugin(SetupPlugin)
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LookTransformPlugin)
        .insert_resource(InspectableRegistry::default());
    app.run();
}

struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup_stuff.system());
    }
}

fn setup_stuff(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh2: Handle<Mesh> = asset_server.load("8x8x8.vox");

    let handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, BASIC_COLOR_VERT)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, BASIC_COLOR_FRAG))),
    }));

    asset_server.watch_for_changes().unwrap();

    let vox_mesh_render_pipeline = RenderPipeline::new(handle);

    commands.spawn_bundle(MeshBundle {
        mesh: mesh2,
        render_pipelines: RenderPipelines::from_pipelines(vec![vox_mesh_render_pipeline]),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere::default())),
        material: materials.add(pbr_flatcolor(Color::ORANGE_RED)),
        ..Default::default()
    });

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
        ..Default::default()
    });

    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController {
            enabled: true,
            mouse_rotate_sensitivity: vec2(0.002, 0.002),
            mouse_translate_sensitivity: vec2(0.4, 0.4),
            mouse_wheel_zoom_sensitivity: 0.1,
            smoothing_weight: 0.9,
        },
        PerspectiveCameraBundle::new_3d(),
        vec3(0.0, 0.0, 50.0), // be reasonably far away for 2D entities
        Vec3::ZERO,           // look towards the forward facing 2D entities
    ));
}

fn pbr_flatcolor(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        unlit: true,
        ..Default::default()
    }
}

pub const BASIC_COLOR_VERT: &str = r#"#version 450
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

pub const BASIC_COLOR_FRAG: &str = r#"#version 450
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
