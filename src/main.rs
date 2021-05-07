// preludes
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ElementState},
    math::bool,
    prelude::*,
    utils::HashMap,
};

// special uses
use bevy::input::system::exit_on_esc_system;

pub type Pos = (usize, usize);

#[allow(dead_code)]
pub enum Dir {
    W,
    E,
    N,
    S,
}

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

use lazy_static::lazy_static;
use prometheus::{self, register_int_gauge, IntGauge};

macro_rules! gauge {
    ($NAME:ident) => {
        lazy_static! {
            static ref $NAME: IntGauge =
                register_int_gauge!(stringify!($NAME).to_lowercase(), "no help").unwrap();
        }
    };
}

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_system(show_metrics_system.system())
        .add_system(exit_on_esc_system.system())
        .add_system(item_pusher_system.system())
        .add_system(item_processor_system.system())
        .add_startup_system_to_stage(StartupStage::PostStartup, map_cache_system.system())
        .add_startup_system(setup.system());
    app.run();
}

pub fn show_metrics_system(mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::M {
                let mfamilies = prometheus::default_registry().gather();
                println!();
                for mfamily in mfamilies {
                    print!("{} ", mfamily.get_name());
                    let mtype = mfamily.get_field_type();
                    for m in mfamily.get_metric() {
                        match mtype {
                            prometheus::proto::MetricType::GAUGE => {
                                print!("{} ", m.get_gauge().get_value())
                            }
                            prometheus::proto::MetricType::COUNTER => {
                                print!("{} ", m.get_counter().get_value())
                            }
                            prometheus::proto::MetricType::SUMMARY => todo!(),
                            prometheus::proto::MetricType::UNTYPED => todo!(),
                            prometheus::proto::MetricType::HISTOGRAM => todo!(),
                        }
                    }
                    println!();
                }
                println!();
            }
        }
    }
}

fn setup(mut cmds: Commands) {
    let to_pos = |x| (x, 0);

    for (x, char) in "    >=====o   ".chars().enumerate() {
        let pos = to_pos(x);
        match char {
            '>' => {
                cmds.spawn_bundle(place_extractor(pos));
            }
            '=' => {
                cmds.spawn_bundle(place_belt(pos));
            }
            'o' => {
                cmds.spawn_bundle(place_oven(pos));
            }
            _ => (),
        }
    }

    cmds.insert_resource(MapCache::default());
}

fn item_pusher_system(
    pusher: Query<(&ItemPusher, &Pos)>,
    map: Res<MapCache>,
    mut sink: Query<&mut ItemSink>,
) {
    for (pusher, pos) in pusher.iter() {
        let next = pusher.direction.pos(pos);
        if let Some(&dest) = map.pos_cache.get(&next) {
            for mut sink in sink.get_mut(dest) {
                sink.consume_item(Item::Red);
            }
        }
    }
}

fn map_cache_system(mut map: ResMut<MapCache>, pos: Query<(Entity, &Pos, &String), Added<Pos>>) {
    for (e, pos, name) in pos.iter() {
        map.entity_cache.insert(e, pos.clone());
        map.pos_cache.insert(pos.clone(), e);
    }
}

fn map_cache_gc_system(mut map: ResMut<MapCache>, removed: RemovedComponents<Pos>) {
    for e in removed.iter() {
        map.entity_cache
            .remove(&e)
            .and_then(|pos| map.pos_cache.remove(&pos));
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
    println!("place extractor {:?}", pos);
    ("extractor".to_string(), ItemPusher::default(), pos)
}

fn place_belt(pos: Pos) -> impl Bundle {
    println!("place belt {:?}", pos);

    let pusher = ItemPusher::default();
    let sink = ItemSink::default();
    /*
    sink forward to belt
    belt forward to pusher
    */

    ("belt".to_string(), Belt::default(), pos)
}

fn place_oven(pos: Pos) -> impl Bundle {
    println!("place oven {:?}", pos);
    ("oven".to_string(), ItemSink::default(), pos)
}

#[derive(Default)]
pub struct ItemPusher {
    pub direction: Dir,
    items: Vec<Item>,
}

impl ItemPusher {
    fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    fn pop_item(&mut self) {
        self.items.remove(0);
    }
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
        self.consume_item(Item::Red);
    }
}

#[derive(Default)]
pub struct Belt {
    pub direction: Dir,
}

impl Belt {
    fn pass_on_item(&self) {
        BELT_PASS_ON_ITEM.inc();
    }
}

gauge!(BELT_PASS_ON_ITEM);

// Processor

#[derive(Default)]
pub struct Processor {
    item: Option<Item>,
}

impl Processor {
    fn try_process(&mut self, mut sink: Mut<ItemSink>, mut pusher: Mut<ItemPusher>) {
        if !sink.items.is_empty() {}
    }
}

// ItemSink

#[derive(Default)]
pub struct ItemSink {
    items: Vec<Item>,
}

impl ItemSink {
    fn consume_item(&mut self, item: Item) -> Option<Item> {
        if self.items.len() < 1 {
            ITEMSINK_CONSUME_ITEM.inc();
            self.items.push(item);
            None
        } else {
            Some(item)
        }
    }
}

gauge!(ITEMSINK_CONSUME_ITEM);

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
enum Item {
    Red,
    Green,
    Blue,
}

// Here, e is an EntityHandler and depending where you are you can pass an handler
// which implements item_acceptor and those functions with builders for those things
// or with querying functions for those things.
// fn cutter4_with_entityhandler(e: &mut EntityHandler) -> Option<()> {
//     use Dir::*;

//     let acceptor = e.item_acceptor()
//         .slot(1).color().pos(0, 0).dir(S)
//         .slot(2).shape().pos(1, 0).dir(S);
//     let ejector = e.item_ejector()
//         .slot(1).shape().pos(0, 1).dir(N)
//         .slot(2).shape().pos(1, 1).dir(N)
//         .slot(3).shape().pos(2, 1).dir(N)
//         .slot(4).shape().pos(3, 1).dir(N);
//     let wiring = e.wiring()
//         .pin(1).input().pos(0, 0).dir(S)
//         .pin(2).output().pos(1, 0).dir(S);

//     check(ejector.slots().all(EjectorSlot::is_free))?;
//     check(wiring.value_pin(1).is_truthy())?;
//     check(acceptor.slot(1).has_color())?;
//     check(acceptor.slot(2).has_shape())?;

//     let color = acceptor.slot(1).take().unwrap();
//     let shape = acceptor.slot(2).take().unwrap();
//     let shape = color.paint(shape);
//     wiring.set_pin(2, shape);

//     for (shape, piece) in shape.pieces() {
//         match piece {
//             Piece::TR => ejector.eject(1, shape),
//             Piece::BR => ejector.eject(2, shape),
//             Piece::BL => ejector.eject(3, shape),
//             Piece::TL => ejector.eject(4, shape),
//         }
//     }

//     Some(())
// }

#[derive(Default)]
struct ItemAcceptor {
    slots: HashMap<usize, ItemAcceptorSlot>,
}
struct ItemAcceptorSlot {
    pos: Pos,
    dir: Dir,
    filter: ItemFilter,
}
impl ItemAcceptorSlot {
    fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = (x, y);
        self
    }
    fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }
    fn filter(&mut self, filter: ItemFilter) -> &mut Self {
        self.filter = filter;
        self
    }

    fn has_color(&self) -> bool {
        false
    }
    fn has_shape(&self) -> bool {
        false
    }
    fn take(&mut self) -> Option<Item> {
        Some(Item::Red)
    }
}

enum ItemFilter {
    Color,
    Shape,
    All,
}

#[derive(Default)]
struct ItemEjector {
    slots: HashMap<usize, ItemEjectorSlot>,
}

#[derive(Default)]
struct ItemEjectorSlot {
    items: Vec<Item>,
    max_items: usize,
    pos: Pos,
    dir: Dir,
}

#[derive(Default)]
struct Wiring {
    pins: HashMap<usize, Pin>,
}

#[derive(Default)]
struct Pin {
    pin_dir: PinDir,
    pos: Pos,
    dir: Dir,
    signal: Signal,
}

enum PinDir {
    Input,
    Output,
}

impl Default for PinDir {
    fn default() -> Self {
        PinDir::Input
    }
}

enum Signal {
    None,
}

impl Default for Signal {
    fn default() -> Self {
        Signal::None
    }
}

type ItemPiece = (Item, Piece);

struct ItemProcessor {
    label: ProcessorLabel,
}

#[allow(dead_code)]
enum ProcessorLabel {
    Cutter4,
    Mixer,
}

#[allow(dead_code)]
enum Piece {
    TR,
    BR,
    BL,
    TL,
}

fn cutter4_building(cmds: &mut Commands) {
    use Dir::*;
    let processor = ItemProcessor {
        label: ProcessorLabel::Cutter4,
    };
    let mut acceptor = ItemAcceptor::default();
    acceptor.slot(1).pos(0, 0).dir(S);
    acceptor.slot(2).pos(1, 0).dir(S);
    let mut ejector = ItemEjector::default();
    ejector.slot(1).pos(0, 1).dir(N);
    ejector.slot(1).pos(1, 1).dir(N);
    ejector.slot(1).pos(2, 1).dir(N);
    ejector.slot(1).pos(3, 1).dir(N);
    let mut wiring = Wiring::default();
    wiring.pin(1).input().pos(0, 0).dir(S);
    wiring.pin(2).output().pos(1, 0).dir(S);

    cmds.spawn_bundle(("cutter4".to_string(), processor, acceptor, ejector, wiring));
}

fn cutter4_process(
    _me: Entity,
    _processor: &mut ItemProcessor,
    mut acceptor: Mut<ItemAcceptor>,
    mut ejector: Mut<ItemEjector>,
    mut wiring: Mut<Wiring>,
) -> Option<()> {
    check(ejector.slots().all(ItemEjectorSlot::is_free))?;
    check(wiring.value_pin(1).is_truthy())?;
    check(acceptor.slot(1).has_color())?;
    check(acceptor.slot(2).has_shape())?;

    let color = acceptor.take_slot(1).unwrap();
    let shape = acceptor.take_slot(2).unwrap();
    let shape = color.paint(shape);
    wiring.set_pin(2, shape.signal());

    for (shape, piece) in shape.pieces() {
        match piece {
            Piece::TR => ejector.eject(1, shape),
            Piece::BR => ejector.eject(2, shape),
            Piece::BL => ejector.eject(3, shape),
            Piece::TL => ejector.eject(4, shape),
        }
    }

    Some(())
}

fn item_processor_system(
    mut query: Query<(
        Entity,
        &mut ItemProcessor,
        &mut ItemAcceptor,
        &mut ItemEjector,
        &mut Wiring,
    )>,
) {
    for (me, mut processor, acceptor, ejector, wiring) in query.iter_mut() {
        processor.process(me, acceptor, ejector, wiring);
    }
}

impl ItemProcessor {
    fn process(
        &mut self,
        me: Entity,
        acceptor: Mut<ItemAcceptor>,
        ejector: Mut<ItemEjector>,
        wiring: Mut<Wiring>,
    ) -> Option<()> {
        match self.label {
            ProcessorLabel::Cutter4 => cutter4_process(me, self, acceptor, ejector, wiring),
            ProcessorLabel::Mixer => None,
        }
    }
}

fn check<T: Default>(b: bool) -> Option<T> {
    if b {
        Some(T::default())
    } else {
        None
    }
}

impl ItemEjector {
    fn slot(&mut self, slot: usize) -> &mut ItemEjectorSlot {
        self.slots.insert(slot, ItemEjectorSlot::default());
        self.slots.get_mut(&slot).unwrap()
    }

    fn slots(&self) -> impl Iterator<Item = &ItemEjectorSlot> {
        [].iter()
    }
    fn eject(&mut self, slot: usize, item: Item) {}
}
impl ItemEjectorSlot {
    fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = (x, y);
        self
    }
    fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }

    fn is_free(&self) -> bool {
        self.items.len() < self.max_items
    }
}
impl Wiring {
    fn pin(&mut self, pin: usize) -> &mut Pin {
        self.pins.insert(pin, Pin::default());
        self.pins.get_mut(&pin).unwrap()
    }

    fn value_pin(&self, pin: usize) -> Signal {
        Signal::None
    }
    fn set_pin(&mut self, pin: usize, signal: Signal) {}
}

impl Pin {
    fn input(&mut self) -> &mut Self {
        self.pin_dir = PinDir::Input;
        self
    }

    fn output(&mut self) -> &mut Self {
        self.pin_dir = PinDir::Output;
        self
    }

    fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = (x, y);
        self
    }

    fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }
}

impl Signal {
    fn is_truthy(&self) -> bool {
        false
    }
}
impl ItemAcceptor {
    fn slot(&mut self, slot: usize) -> &mut ItemAcceptorSlot {
        self.slots.insert(
            slot,
            ItemAcceptorSlot {
                pos: (0, 0),
                dir: Dir::S,
                filter: ItemFilter::Color,
            },
        );
        self.slots.get_mut(&slot).unwrap()
    }
    fn take_slot(&mut self, slot: usize) -> Option<Item> {
        None
    }
}
impl Item {
    fn paint(self, other: Item) -> Item {
        Item::Blue
    }
    fn signal(&self) -> Signal {
        Signal::None
    }
    fn pieces(self) -> std::array::IntoIter<ItemPiece, 1> {
        std::array::IntoIter::new([(self, Piece::TR)])
    }
}
