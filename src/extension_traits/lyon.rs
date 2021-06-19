use bevy::prelude::*;
use bevy_prototype_lyon::{entity::ShapeBundle, prelude::*, shapes::*};

pub use lyon_path::geom as lyon_geom;
pub use lyon_path::path::Builder as LyonBuilder;
pub use lyon_path::traits::*;

pub fn lyon() -> GeometryBuilder {
    GeometryBuilder::new()
}

pub trait GeometryBuilderExt {
    fn add_geometry(self, geometry: &impl Geometry) -> Self;
    fn polygon(self, sides: usize, radius: f32) -> Self;
    fn rectangle(self, width: f32, height: f32) -> Self;
    fn circle(self, radius: f32) -> Self;
    fn outlined(self, fill: Color, stroke: Color, width: f32) -> ShapeBundle;
    fn outlined_pos(self, fill: Color, stroke: Color, width: f32, vec: Vec2) -> ShapeBundle;
    fn outlined_pos3(self, fill: Color, stroke: Color, width: f32, vec: Vec3) -> ShapeBundle;
}

impl GeometryBuilderExt for GeometryBuilder {
    fn add_geometry(mut self, geometry: &impl Geometry) -> Self {
        self.add(geometry);
        self
    }

    fn circle(mut self, radius: f32) -> Self {
        self.add(&Circle {
            center: Vec2::ZERO,
            radius,
        });
        self
    }

    fn outlined(self, fill: Color, stroke: Color, width: f32) -> ShapeBundle {
        self.build(
            ShapeColors::outlined(fill, stroke),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(width),
            },
            Transform::default(),
        )
    }

    fn outlined_pos(self, fill: Color, stroke: Color, width: f32, vec: Vec2) -> ShapeBundle {
        self.build(
            ShapeColors::outlined(fill, stroke),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(width),
            },
            Transform::from_translation(vec.extend(0.0)),
        )
    }

    fn outlined_pos3(self, fill: Color, stroke: Color, width: f32, vec: Vec3) -> ShapeBundle {
        self.build(
            ShapeColors::outlined(fill, stroke),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(width),
            },
            Transform::from_translation(vec),
        )
    }

    fn polygon(mut self, sides: usize, radius: f32) -> Self {
        self.add(&RegularPolygon {
            center: Vec2::ZERO,
            sides,
            feature: RegularPolygonFeature::Radius(radius),
        });
        self
    }

    fn rectangle(mut self, width: f32, height: f32) -> Self {
        self.add(&Rectangle {
            width,
            height,
            origin: RectangleOrigin::TopLeft,
        });
        self
    }
}
