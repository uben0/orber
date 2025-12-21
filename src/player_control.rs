use std::f32::consts::PI;

use crate::{
    ConsoleState, InputMode, PlacingBlock, Player, SetInputMode, SetPlacingBlock,
    block::Block,
    chunk_blocks::ChunkBlocks,
    chunks::Modify,
    physics::{self, ApplyPhysics, Collider, Grounded, Velocity},
    pointed_block::{BlockPointer, Pointing},
};
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseMotion},
    prelude::*,
};

pub struct PlayerControlPlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct PlayerControl;

impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, PlayerControl.run_if(is_player_control_on))
            .add_systems(
                Update,
                (
                    (
                        control_player_rotation,
                        control_player_physics,
                        control_player_flying,
                    )
                        .before(ApplyPhysics),
                    player_acts,
                    toggle_flying.run_if(input_just_pressed(KeyCode::KeyV)),
                    change_placing_block,
                    player_activate_console,
                    consistency_check.run_if(input_just_pressed(KeyCode::KeyY)),
                )
                    .in_set(PlayerControl),
            );
    }
}

fn is_player_control_on(input_mode: Res<InputMode>) -> bool {
    *input_mode == InputMode::PlayerControl
}

fn player_activate_console(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<ConsoleState>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        console.active = true;
        console.focus = true;
        console.input.clear();
        commands.trigger(SetInputMode(InputMode::UiInteraction));
    }
}
fn player_acts(
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(&BlockPointer, &PlacingBlock), With<Player>>,
    mut commands: Commands,
    colliders: Query<(&Collider, &Transform)>,
) {
    if let Some(Pointing { global, side }) = player.0.pointing {
        if mouse.just_pressed(MouseButton::Left) {
            commands.trigger(Modify::Place {
                global,
                block: Block::Air,
            });
        }
        if mouse.just_pressed(MouseButton::Right) {
            let global = side.neighbour(global);
            let block = player.1.0;
            if !block.collides()
                || colliders.iter().all(|(collider, transform)| {
                    !physics::intersects(*transform, *collider, global)
                })
            {
                commands.trigger(Modify::Place { global, block });
            }
        }
    }
}

fn change_placing_block(keys: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    let map = [
        (KeyCode::Digit0, Block::Stone),
        (KeyCode::Digit1, Block::Sand),
        (KeyCode::Digit2, Block::Grass),
    ];
    for (key_code, block) in map {
        if keys.just_pressed(key_code) {
            commands.trigger(SetPlacingBlock(block));
        }
    }
}

fn toggle_flying(player: Single<(Entity, Has<Velocity>), With<Player>>, mut commands: Commands) {
    let (entity, has_physics) = player.into_inner();
    if has_physics {
        commands.entity(entity).remove::<Velocity>();
    } else {
        commands.entity(entity).insert(Velocity::default());
    }
}

fn control_player_rotation(
    mut mouse: MessageReader<MouseMotion>,
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
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
    pitch = pitch.clamp(-PI / 2.0, PI / 2.0);
    player.rotation = Quat::from_euler(default(), yaw, pitch, 0.0);
}

fn control_player_physics(
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&Transform, &mut Velocity, Has<Grounded>), With<Player>>,
    time: Res<Time>,
) {
    let (transform, mut velocity, grounded) = player.into_inner();
    if keys.pressed(KeyCode::Space) && grounded {
        velocity.linear.y = 12.0;
    }

    let mut dir = Vec3::ZERO;
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

    let force = match (grounded, keys.pressed(KeyCode::KeyA)) {
        (true, true) => 110.0,
        (true, false) => 90.0,
        (false, _) => 40.0,
    };

    let (yaw, _, _) = transform.rotation.to_euler(default());
    let plane_rotation = Quat::from_euler(default(), yaw, 0.0, 0.0);
    velocity.linear += plane_rotation * force * time.delta_secs() * dir;
}

fn control_player_flying(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Single<&mut Transform, (With<Player>, Without<Velocity>)>,
    time: Res<Time>,
) {
    const PLAYER_SPEED: f32 = 8.0;
    const PLAYER_SPEED_BOOST: f32 = 24.0;

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

    let speed = match keys.pressed(KeyCode::KeyA) {
        true => PLAYER_SPEED_BOOST,
        false => PLAYER_SPEED,
    };

    let (yaw, _, _) = player.rotation.to_euler(default());
    let plane_rotation = Quat::from_euler(default(), yaw, 0.0, 0.0);
    player.translation += plane_rotation * dir * time.delta_secs() * speed;
}

fn consistency_check(blocks: Query<&ChunkBlocks>) {
    for blocks in blocks {
        blocks.assert_consistency();
    }
    info!("consistency check successful");
}
