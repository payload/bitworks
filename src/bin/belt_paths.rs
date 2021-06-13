use bevy::prelude::*;
use bitworks::*;

fn main() {
    belts_example_app().run();
}

pub fn belts_example_app() -> AppBuilder {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(BeltPlugin)
        .add_system(exit_on_esc_system.system())
        .add_startup_system(setup.system());
    app
}

fn setup(mut cmds: Commands) {
    let cmds = &mut cmds;
    cmds.spawn_bundle(nice_camera());
    cmds.spawn_bundle(belt1());

    let belt2_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -20.0))).id();
    cmds.spawn_bundle(belt2(belt2_sink));

    let belt3_sink = cmds.spawn_bundle(item_sink(vec2(30.0, -30.0))).id();
    let belt3 = cmds.spawn_bundle(belt3(belt3_sink)).id();
    cmds.spawn_bundle(item_generator(belt3, vec2(-30.0, -30.0), 0.0));

    {
        let in1 = (ItemInput::new(1), vec2(0.0, -40.0)).spawn(cmds);
        let in2 = (ItemInput::new(1), vec2(0.0, -45.0)).spawn(cmds);
        (NullSink::new(&[in1]),).spawn(cmds);

        let belt1 = belt(-30, -40, in1).spawn(cmds);
        let belt2 = belt(-30, -45, in2).spawn(cmds);

        let merge1 = (ItemInput::new(1), vec2(-35.0, -40.0)).spawn(cmds);
        let merge2 = (ItemInput::new(1), vec2(-35.0, -45.0)).spawn(cmds);

        (Merger {
            cooldown: 0.0,
            next_time: 0.0,
            items_per_step: 1,
            input_cursor: 0,
            output_cursor: 0,
            inputs: vec![merge1, merge2],
            outputs: vec![belt1, belt2],
        },)
            .spawn(cmds);

        let belt_merge1 = belt(-65, -40, merge1).spawn(cmds);
        let belt_merge2 = belt(-65, -45, merge2).spawn(cmds);

        item_generator(belt_merge1, vec2(-65.0, -40.0), 0.5).spawn(cmds);
        item_generator(belt_merge2, vec2(-65.0, -45.0), 0.0).spawn(cmds);
    }
}

fn belt1() -> impl Bundle {
    (
        Belt {
            segments: vec![
                BeltSegment::straight(-30, 30, 0, 30),
                BeltSegment::straight(0, 30, 30, 50),
            ],
            items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
            output: None,
        },
        ItemInput::new(2),
    )
}

fn belt2(output: Entity) -> impl Bundle {
    (
        Belt {
            segments: vec![
                BeltSegment::straight(-30, 10, 0, 0),
                BeltSegment::straight(0, 0, 30, -20),
            ],
            items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
            output: Some(output),
        },
        ItemInput::new(2),
    )
}

fn belt3(output: Entity) -> impl Bundle {
    (
        Belt {
            segments: vec![
                BeltSegment::straight(-30, -30, 0, -30),
                BeltSegment::straight(0, -30, 30, -30),
            ],
            items: vec![BeltItem::red(0.0), BeltItem::green(30.0)],
            output: Some(output),
        },
        ItemInput::new(2),
    )
}

fn belt(x: i32, y: i32, output: Entity) -> impl Bundle {
    (
        Belt {
            segments: vec![BeltSegment::straight(x, y, x + 30, y)],
            items: vec![],
            output: Some(output),
        },
        ItemInput::new(2),
    )
}

fn item_sink(pos: Vec2) -> impl Bundle {
    (pos, ItemInput::new(2))
}

fn item_generator(belt: Entity, pos: Vec2, cooldown: f32) -> impl Bundle {
    (
        pos,
        RandomItemGenerator {
            cooldown,
            next_time: 0.0,
            output: Some(belt),
        },
    )
}
