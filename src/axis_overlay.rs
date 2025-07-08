use bevy::{prelude::*, render::view::RenderLayers};

#[derive(GizmoConfigGroup, Default, Reflect)]
struct AxisOverlayGizmoConfig;

#[derive(Debug, Clone, Copy)]
pub struct AxisOverlayPlugin<T> {
    pub layer: usize,
    pub order: isize,
    pub length: f32,
    pub thickness: f32,
    pub target: T,
}

impl<T> Default for AxisOverlayPlugin<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            layer: 1,
            order: 1,
            length: 20.0,
            thickness: 4.0,
            target: default(),
        }
    }
}

impl<T> Plugin for AxisOverlayPlugin<T>
where
    T: Component,
{
    fn build(&self, app: &mut App) {
        let _ = self.target;
        let AxisOverlayPlugin {
            layer,
            order,
            length,
            thickness,
            ..
        } = *self;

        app.insert_gizmo_config(
            AxisOverlayGizmoConfig,
            GizmoConfig {
                line: GizmoLineConfig {
                    width: thickness,
                    ..default()
                },
                render_layers: RenderLayers::layer(self.layer),
                ..default()
            },
        );

        app.add_systems(Startup, move |mut commands: Commands| {
            commands.spawn((
                Transform::from_xyz(0.0, 0.0, length),
                Camera3d::default(),
                Camera {
                    order,
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
            move |mut gizmos: Gizmos<AxisOverlayGizmoConfig>,
                  transform: Single<&Transform, With<T>>| {
                let orient = transform.rotation.inverse();
                for (base, color) in [
                    (Vec3::X, Color::srgb(1.0, 0.0, 0.0)),
                    (Vec3::Y, Color::srgb(0.0, 1.0, 0.0)),
                    (Vec3::Z, Color::srgb(0.0, 0.0, 1.0)),
                ] {
                    gizmos.line(Vec3::ZERO, orient * base * length, color);
                }
            },
        );
    }
}
