use bitworks::*;

use bevy_egui::{egui, EguiContext, EguiPlugin};

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(VoxelPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(Setup);
    app.run();
}

struct Setup;

impl Plugin for Setup {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(tool_ui.system());
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Tool {
    Clear,
    Spring,
    Glassblower,
    Tap,
    Trash,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Clear
    }
}

fn tool_ui(mut local_tool: Local<Tool>, egui_ctx: Res<EguiContext>) {
    let tool = &mut *local_tool;

    egui::Window::new("Tool")
        .scroll(true)
        .default_width(100.0)
        .show(egui_ctx.ctx(), |ui| {
            ui.selectable_value(tool, Tool::Clear, "❌ Clear");
            ui.selectable_value(tool, Tool::Spring, "💧 Spring");
            ui.selectable_value(tool, Tool::Glassblower, "🥃 Glassblower");
            ui.selectable_value(tool, Tool::Tap, "🚰 Tap");
            ui.selectable_value(tool, Tool::Trash, "🗑 Trash");
        });
}
