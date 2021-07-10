use tiny_skia::*;

#[derive(Clone, Copy)]
enum CompassDir {
    W,
    E,
    N,
    S,
}

impl CompassDir {
    fn p(&self, f: f32) -> (f32, f32) {
        let o = 0.0;
        let h = 0.5 * f;
        match self {
            CompassDir::W => (o, h),
            CompassDir::E => (f, h),
            CompassDir::N => (h, o),
            CompassDir::S => (h, f),
        }
    }

    fn opposite(&self) -> Self {
        match self {
            CompassDir::W => CompassDir::E,
            CompassDir::E => CompassDir::W,
            CompassDir::N => CompassDir::S,
            CompassDir::S => CompassDir::N,
        }
    }
}

fn main() {
    let output_path = std::env::args()
        .nth(1)
        .expect("expect output path as first argument");
    std::env::set_current_dir(output_path).expect("can't use output path");

    output_pixmap(
        &create_belt_example_pixmap(&[W, E, E, S, S, W, S, E, E, N, N, E, N, E, N, N, W, W, S, E]),
        "belt_example.png",
    );

    output_pixmap(&create_belt_atlas_pixmap(), "belt_atlas.png");

    output_pixmap(&create_item_pixmap(), "item.png");
}

use CompassDir::*;

struct BeltBounds {
    size: (u32, u32),
    start: (u32, u32),
}

fn belt_bounds(dirs: &[CompassDir]) -> BeltBounds {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut cur_x = 0;
    let mut cur_y = 0;

    for dir in dirs[1..].iter() {
        match dir {
            W => {
                cur_x -= 1;
                min_x = min_x.min(cur_x);
            }
            E => {
                cur_x += 1;
                max_x = max_x.max(cur_x);
            }
            N => {
                cur_y -= 1;
                min_y = min_y.min(cur_y);
            }
            S => {
                cur_y += 1;
                max_y = max_y.max(cur_y);
            }
        }
    }

    BeltBounds {
        size: ((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32),
        start: (-min_x as u32, -min_y as u32),
    }
}

fn draw_belt_segment_dir(
    pixmap: &mut Pixmap,
    from: CompassDir,
    to: CompassDir,
    transform: Transform,
    anim: f32,
) {
    draw_belt_segment(
        pixmap,
        from.p(48.0).into(),
        to.p(48.0).into(),
        transform,
        anim,
    );
}

fn draw_belt_segment(pixmap: &mut Pixmap, from: Point, to: Point, transform: Transform, anim: f32) {
    let path = {
        let mut pb = PathBuilder::new();
        pb.move_to(from.x, from.y);
        pb.quad_to(24.0, 24.0, to.x, to.y);
        pb.finish().unwrap()
    };

    let mut paint1 = Paint::default();
    paint1.set_color_rgba8(50, 127, 150, 200);
    paint1.anti_alias = false;

    let mut stroke3 = Stroke::default();
    stroke3.width = 32.0;

    let mut stroke4 = Stroke::default();
    stroke4.width = 32.0;
    stroke4.dash = StrokeDash::new(vec![4.0, 4.0], -8.0 * anim);

    pixmap.stroke_path(&path, &paint1, &stroke3, transform, None);
    pixmap.stroke_path(&path, &paint1, &stroke4, transform, None);
}

fn draw_belt(pixmap: &mut Pixmap, belt: &[CompassDir], start: Point) {
    let mut transform = Transform::from_translate(start.x, start.y);
    let mut from = *belt.first().unwrap();

    for to in belt[1..].iter() {
        draw_belt_segment_dir(pixmap, from, *to, transform, 0.0);

        let f = 48.0;
        let o = 0.0;
        transform = match to {
            W => transform.post_translate(-f, o),
            E => transform.post_translate(f, o),
            N => transform.post_translate(o, -f),
            S => transform.post_translate(o, f),
        };
        from = to.opposite();
    }
}

fn create_belt_example_pixmap(belt: &[CompassDir]) -> Pixmap {
    let bounds = belt_bounds(&belt);
    let start = (bounds.start.0 as f32 * 48.0, bounds.start.1 as f32 * 48.0);
    let mut pixmap = Pixmap::new(48 * bounds.size.0, 48 * bounds.size.1).unwrap();
    draw_belt(&mut pixmap, &belt, start.into());
    pixmap
}

fn create_belt_atlas_pixmap() -> Pixmap {
    let mut pixmap = Pixmap::new(48 * 8 + 2 * 7, 48 * 8 + 2 * 7).unwrap();
    let p = &mut pixmap;

    for x in 0..8 {
        let anim = x as f32 / 8.0;
        draw_belt_segment_dir(p, W, E, t(x, 0), anim);
        draw_belt_segment_dir(p, N, S, t(x, 1), anim);
        draw_belt_segment_dir(p, W, N, t(x, 2), anim);
        draw_belt_segment_dir(p, S, E, t(x, 3), anim);
    }

    fn t(x: i32, y: i32) -> Transform {
        Transform::from_translate(x as f32 * 50.0, y as f32 * 50.0)
    }

    pixmap
}

fn create_item_pixmap() -> Pixmap {
    let mut paint = Paint::default();
    paint.set_color_rgba8(240, 190, 90, 255);

    let mut pixmap = Pixmap::new(48, 48).unwrap();
    pixmap.fill_rect(
        Rect::from_xywh(16.0, 16.0, 16.0, 16.0).unwrap(),
        &paint,
        Transform::identity(),
        None,
    );

    pixmap
}

fn output_pixmap(pixmap: &Pixmap, png_name: &str) {
    pixmap.save_png(png_name).unwrap();
    println!(
        "{}/{}",
        std::env::current_dir().unwrap().to_str().unwrap(),
        png_name
    );
}
