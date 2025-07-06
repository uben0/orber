use bevy::{prelude::*, render::view::RenderLayers};
use std::marker::PhantomData;

#[derive(GizmoConfigGroup, Default, Reflect)]
struct AxisOverlayGizmoConfig;

#[derive(Debug, Clone, Copy)]
pub struct AxisOverlayPlugin<T> {
    layer: usize,
    order: isize,
    phantom: PhantomData<T>,
}

impl<T> AxisOverlayPlugin<T> {
    pub fn new(layer: usize, order: isize) -> Self {
        Self {
            layer,
            order,
            phantom: PhantomData,
        }
    }
}

impl<T> Plugin for AxisOverlayPlugin<T>
where
    T: Component,
{
    fn build(&self, app: &mut App) {
        let layer = self.layer;
        let order = self.order;

        app.insert_gizmo_config(
            AxisOverlayGizmoConfig,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: 4.0,
                    ..default()
                },
                render_layers: RenderLayers::layer(self.layer),
                ..default()
            },
        );

        app.add_systems(Startup, move |mut commands: Commands| {
            // axis overlay camera
            commands.spawn((
                // TODO: remove looking_at
                Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
                Camera3d::default(),
                Camera {
                    // renders in front of everything
                    order: order,
                    // do not clear background
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                RenderLayers::layer(layer),
                Projection::Orthographic(OrthographicProjection::default_3d()),
            ));
        });

        app.add_systems(
            Update,
            |mut gizmos: Gizmos<AxisOverlayGizmoConfig>, transform: Single<&Transform, With<T>>| {
                const SCALE: f32 = 20.0;
                let orient = transform.rotation.inverse();
                for (base, color) in [
                    (Vec3::X, Color::srgb(1.0, 0.0, 0.0)),
                    (Vec3::Y, Color::srgb(0.0, 1.0, 0.0)),
                    (Vec3::Z, Color::srgb(0.0, 0.0, 1.0)),
                ] {
                    gizmos.line(Vec3::ZERO, orient * base * SCALE, color);
                }
            },
        );
    }
}
