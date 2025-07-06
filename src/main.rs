use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology::TriangleList},
};
use bevy_framepace::FramepacePlugin;
use std::f32::consts::PI;

use crate::spacial::{Side, Sides};

mod spacial;
mod swizzle;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FramepacePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, control_player)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();
    make_cube_mesh(
        &mut positions,
        &mut normals,
        &mut indices,
        Sides {
            x_pos: true,
            x_neg: false,
            y_pos: true,
            y_neg: true,
            z_pos: true,
            z_neg: true,
        },
    );
    let mesh = Mesh::new(TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices));

    commands.spawn((
        Transform::default(),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 1000.0,
        ..default()
    });
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(default(), -0.4, -1.2, 0.0)),
        DirectionalLight { ..default() },
    ));
    commands.spawn((
        Player,
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::splat(0.0), Vec3::Y),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 85.0_f32.to_radians(),
            ..default()
        }),
    ));
    // commands.spawn((
    //     Transform::default(),
    //     Mesh3d(meshes.add(Sphere::new(1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
    // ));
    commands.spawn((
        Transform::from_xyz(-2.0, 0.2, 0.1),
        Mesh3d(meshes.add(Sphere::new(0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
    ));
}

fn control_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    const PLAYER_SPEED: f32 = 4.0;
    const PLAYER_ROTATION: f32 = 0.2;

    let (mut yaw, mut pitch, _) = player.rotation.to_euler(default());
    for MouseMotion {
        delta: Vec2 { x, y },
    } in mouse.read()
    {
        yaw -= x * time.delta_secs() * PLAYER_ROTATION;
        pitch -= y * time.delta_secs() * PLAYER_ROTATION;
    }
    yaw = yaw.rem_euclid(2.0 * PI);
    pitch = pitch.clamp(-PI, PI);

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::Space) {
        dir += Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyZ) {
        dir -= Vec3::Y;
    }
    if keys.pressed(KeyCode::KeyE) {
        dir -= Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += Vec3::Z;
    }
    if keys.pressed(KeyCode::KeyF) {
        dir += Vec3::X;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= Vec3::X;
    }

    let dir = dir.normalize_or_zero();
    let plane_rotation = Quat::from_euler(default(), yaw, 0.0, 0.0);
    player.translation += plane_rotation * dir * time.delta_secs() * PLAYER_SPEED;
    player.rotation = Quat::from_euler(default(), yaw, pitch, 0.0);
}

fn make_cube_mesh(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    visible: Sides<bool>,
) {
    for side in Side::ALL {
        if visible[side] {
            let index = positions.len() as u32;
            positions.extend(side.quad());
            normals.extend([side.normal(); 4]);
            indices.extend([
                index + 0,
                index + 1,
                index + 2,
                index + 2,
                index + 3,
                index + 0,
            ]);
        }
    }
}
