use bevy::prelude::*;

pub trait SpawnBundle {
    fn spawn(self, cmds: &mut Commands) -> Entity;
}

impl<T: Bundle> SpawnBundle for T {
    fn spawn(self, cmds: &mut Commands) -> Entity {
        cmds.spawn_bundle(self).id()
    }
}
