use crate::{
    axis_overlay::AxisOverlayPlugin,
    chunk_blocks::{ChunkBlocks, chunk_generation},
    chunk_meshing::chunk_meshing,
    chunks::{ChunksIndex, Loader, chunk_indexer, chunk_state_show},
    ray_travel::RayTraveler,
    spacial::Side,
};
use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_framepace::FramepacePlugin;
use std::f32::consts::PI;

mod axis_overlay;
mod chunk_blocks;
mod chunk_meshing;
mod chunks;
mod octahedron;
mod ray_travel;
mod spacial;
mod swizzle;

const CHUNK_WIDTH: i32 = 32;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            AxisOverlayPlugin {
                target: Player,
                ..default()
            },
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                control_player,
                chunk_meshing,
                chunk_indexer,
                chunk_generation,
                chunk_state_show,
                pointed_block,
            ),
        )
        .run();
}

#[derive(Component, Default)]
struct Player;

#[derive(Component)]
struct PointedBlock {
    global: IVec3,
    side: Side,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(ChunksIndex::new());
    commands.insert_resource(ClearColor(Color::srgb(0.7, 0.9, 1.0)));
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
        Loader::new(40.0, 10.0),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::splat(0.0), Vec3::Y),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 85.0_f32.to_radians(),
            ..default()
        }),
    ));
    commands.spawn((
        Transform::from_xyz(-2.0, 0.2, 0.1),
        Mesh3d(meshes.add(Sphere::new(0.6))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
    ));
}

fn pointed_block(
    player: Single<(Entity, &Transform, Option<&PointedBlock>), With<Player>>,
    blocks: Query<&ChunkBlocks>,
    index: Res<ChunksIndex>,
    mut commands: Commands,
    mut gizmos: Gizmos,
) {
    let (entity, transform, pointed) = player.into_inner();

    for step in RayTraveler::new(
        transform.translation,
        transform.rotation * Dir3::NEG_Z,
        16.0,
    ) {
        gizmos.cuboid(
            Transform {
                translation: step.voxel.as_vec3() + 0.5 * Vec3::ONE,
                rotation: default(),
                scale: Vec3::splat(1.0),
            },
            Color::srgb(1.0, 1.0, 1.0),
        );
    }
    // commands.entity(entity).remove();
}

fn control_player(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    const PLAYER_SPEED: f32 = 8.0;
    const PLAYER_SPEED_BOOST: f32 = 24.0;
    const PLAYER_ROTATION: f32 = 0.2;

    // TODO: fix behaviour at max and min pitch
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

    let speed = match keys.pressed(KeyCode::KeyA) {
        true => PLAYER_SPEED_BOOST,
        false => PLAYER_SPEED,
    };

    let dir = dir.normalize_or_zero();
    let plane_rotation = Quat::from_euler(default(), yaw, 0.0, 0.0);
    player.translation += plane_rotation * dir * time.delta_secs() * speed;
    player.rotation = Quat::from_euler(default(), yaw, pitch, 0.0);
}
