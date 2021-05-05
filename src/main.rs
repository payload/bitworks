// preludes
use bevy::{prelude::*, utils::HashMap};

// special uses
use bevy::input::system::exit_on_esc_system;

pub type Pos = (usize, usize);

pub enum Dir { W, E, N, S }

impl Dir {
    fn pos(&self, base: &Pos) -> Pos {
        let (x, y) = base;
        match *self {
            Dir::W => (x - 1, y + 0),
            Dir::E => (x + 1, y + 0),
            Dir::N => (x + 0, y - 1),
            Dir::S => (x + 0, y + 1),
        }
    }

    fn invert(&self) -> Self {
        match *self {
            Dir::W => Dir::E,
            Dir::E => Dir::W,
            Dir::N => Dir::S,
            Dir::S => Dir::N,
        }
    }
}

impl Default for Dir {
    fn default() -> Self {
        Self::E
    }
}

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system.system())
        .add_system(item_pusher_system.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, map_cache_system.system())
        .add_startup_system(setup.system());
    app.run();
}

fn setup(mut cmds: Commands) {
    let to_pos = |x| (x, 0);

    for (x, char) in "    >=====o   ".chars().enumerate() {
        let pos = to_pos(x);
        match char {
            '>' => { cmds.spawn_bundle(place_extractor(pos)); }
            '=' => { cmds.spawn_bundle(place_belt(pos)); }
            'o' => { cmds.spawn_bundle(place_oven(pos)); }
            _ => (),
        }
    }

    cmds.insert_resource(MapCache::default());
}

fn item_pusher_system(pusher: Query<(&ItemPusher, &Pos)>, map: Res<MapCache>, mut receiver: Query<(Option<&mut Belt>, Option<&mut ItemSink>)>) {
    println!("item pusher system");
    for (pusher, pos) in pusher.iter() {
        let next = pusher.direction.pos(pos);
        if let Some(&dest) = map.pos_cache.get(&next) {
            println!("try push to {:?}", &next);
            for (belt, sink) in receiver.get_mut(dest) {
                if let Some(mut belt) = belt {
                    belt.receive_item();
                } else if let Some(mut sink) = sink {
                    sink.receive_item();
                }
            }
        }
    }
}

fn map_cache_system(mut map: ResMut<MapCache>, pos: Query<(Entity, &Pos, &String), Added<Pos>>) {
    println!("map cache system");
    for (e, pos, name) in pos.iter() {
        println!("map cache add {}", name);
        map.entity_cache.insert(e, pos.clone());
        map.pos_cache.insert(pos.clone(), e);
    }
}

fn map_cache_gc_system(mut map: ResMut<MapCache>, removed: RemovedComponents<Pos>) {
    for e in removed.iter() {
        map.entity_cache.remove(&e).and_then(|pos| map.pos_cache.remove(&pos));
    }
}

#[derive(Default)]
struct MapCache {
    pos_cache: HashMap<Pos, Entity>,
    entity_cache: HashMap<Entity, Pos>,
}
#[derive(Bundle)]
pub struct ExtractorBuilding {
    pub name: String,
    pub pusher: ItemPusher,
    pub pos: Pos,
}

fn place_extractor(pos: Pos) -> impl Bundle {
    ("extractor".to_string(), ItemPusher::default(), pos)
}

fn place_belt(pos: Pos) -> impl Bundle {
    println!("place belt {:?}", pos);
    ("belt".to_string(), Belt::default(), pos)
}

fn place_oven(pos: Pos) -> impl Bundle {
    ("oven".to_string(), ItemSink::default(), pos)
}

#[derive(Default)]
pub struct ItemPusher {
    pub direction: Dir,
}

trait ItemReceiver {
    fn receive_item(&mut self);
}
impl ItemReceiver for Belt {
    fn receive_item(&mut self) {
        self.pass_on_item();
    }
}
impl ItemReceiver for ItemSink {
    fn receive_item(&mut self) {
        self.consume_item();
    }
}

#[derive(Default)]
pub struct Belt;

impl Belt {
    fn pass_on_item(&self) {
        println!("belt pass on item");
    }
}

#[derive(Default)]
pub struct ItemSink;

impl ItemSink {
    fn consume_item(&self) {
        println!("item sink consume item");
    }
}