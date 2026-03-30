use crate::{
    Player,
    block::Block,
    chunks::Modify,
    keybindings::KeyBindings,
    physics::{self, ApplyPhysics, Collider, Grounded, Velocity},
    pointed_block::{BlockPointer, Pointing},
};
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseMotion},
    prelude::*,
};
use std::f32::consts::PI;

#[derive(EntityEvent)]
pub struct Teleport {
    #[event_target]
    pub target: Entity,
    pub position: Vec3,
}

pub struct PlayerControlPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerControlSystemSet;

impl Plugin for PlayerControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
            )
                .in_set(PlayerControlSystemSet),
        )
        .add_observer(on_set_placing_block)
        .add_observer(on_toggle_flying)
        .add_observer(on_teleport);
    }
}

#[derive(Component)]
pub struct PlacingBlock(pub Block);

fn on_teleport(teleport: On<Teleport>, mut tr: Query<&mut Transform>) {
    if let Ok(mut tr) = tr.get_mut(teleport.target) {
        tr.translation = teleport.position;
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

#[derive(Event, Debug, Clone, Deref, Copy)]
pub struct ToggleFlying(pub Option<bool>);

fn on_toggle_flying(
    flying: On<ToggleFlying>,
    player: Single<(Entity, Has<Velocity>), With<Player>>,
    mut commands: Commands,
) {
    let (entity, has_physics) = *player;
    match flying.unwrap_or(has_physics) {
        false => {
            commands.entity(entity).insert(Velocity::default());
        }
        true => {
            commands.entity(entity).remove::<Velocity>();
        }
    }
}

fn toggle_flying(mut commands: Commands) {
    commands.trigger(ToggleFlying(None));
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
    bindings: Res<KeyBindings>,
) {
    let (transform, mut velocity, grounded) = player.into_inner();
    if keys.pressed(bindings.jump) && grounded {
        velocity.linear.y = 12.0;
    }

    let mut dir = Vec3::ZERO;
    if keys.pressed(bindings.move_forward) {
        dir -= Vec3::Z;
    }
    if keys.pressed(bindings.move_backward) {
        dir += Vec3::Z;
    }
    if keys.pressed(bindings.move_right) {
        dir += Vec3::X;
    }
    if keys.pressed(bindings.move_left) {
        dir -= Vec3::X;
    }
    let dir = dir.normalize_or_zero();

    let force = match (grounded, keys.pressed(bindings.sprint)) {
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
    bindings: Res<KeyBindings>,
) {
    const PLAYER_SPEED: f32 = 8.0;
    const PLAYER_SPEED_BOOST: f32 = 24.0;

    let mut dir = Vec3::ZERO;
    if keys.pressed(bindings.jump) {
        dir += Vec3::Y;
    }
    if keys.pressed(bindings.croutch) {
        dir -= Vec3::Y;
    }
    if keys.pressed(bindings.move_forward) {
        dir -= Vec3::Z;
    }
    if keys.pressed(bindings.move_backward) {
        dir += Vec3::Z;
    }
    if keys.pressed(bindings.move_right) {
        dir += Vec3::X;
    }
    if keys.pressed(bindings.move_left) {
        dir -= Vec3::X;
    }
    let dir = dir.normalize_or_zero();

    let speed = match keys.pressed(bindings.sprint) {
        true => PLAYER_SPEED_BOOST,
        false => PLAYER_SPEED,
    };

    let (yaw, _, _) = player.rotation.to_euler(default());
    let plane_rotation = Quat::from_euler(default(), yaw, 0.0, 0.0);
    player.translation += plane_rotation * dir * time.delta_secs() * speed;
}

#[derive(Event, Debug)]
pub struct SetPlacingBlock(pub Block);

fn on_set_placing_block(
    change: On<SetPlacingBlock>,
    mut player: Single<&mut PlacingBlock, With<Player>>,
) {
    player.0 = change.0;
}
