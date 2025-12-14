use crate::atlas_material::AtlasMaterial;
use crate::axis_overlay::AxisOverlayPlugin;
use crate::block::Block;
use crate::chunk_blocks::{ChunkBlocks, chunk_generation};
use crate::chunk_meshing::{chunk_demeshing, chunk_meshing, chunks_mesh_setup};
use crate::chunks::{Loader, Modify, chunk_indexer, chunks_setup};
use crate::command::UserCommandParser;
use crate::physics::{ApplyPhysics, Collider, Grounded, PhysicsPlugin, Velocity};
use crate::pointed_block::{BlockPointer, BlockPointingPlugin, Pointing};
use crate::water_material::WaterMaterial;
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseMotion},
    prelude::*,
    window::CursorGrabMode,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_framepace::FramepacePlugin;
use std::f32::consts::PI;
use std::fmt::Write;

mod array_queue;
mod atlas_material;
mod axis_overlay;
mod block;
mod chunk_blocks;
mod chunk_meshing;
mod chunks;
mod command;
mod octahedron;
mod physics;
mod pointed_block;
mod ray_travel;
mod spacial;
mod terrain;
mod water_material;

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
            BlockPointingPlugin,
            PhysicsPlugin,
            MaterialPlugin::<AtlasMaterial>::default(),
            MaterialPlugin::<WaterMaterial>::default(),
            EguiPlugin::default(),
        ))
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Startup, (setup, chunks_setup, chunks_mesh_setup))
        .configure_sets(Update, PlayerControl.run_if(is_player_control_on))
        .add_systems(
            Update,
            (
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
                    .in_set(PlayerControl),
                chunk_meshing,
                chunk_demeshing,
                chunk_indexer,
                chunk_generation,
                // chunk_state_show,
                toggle_input_mode,
                inspect_ui,
                consistency_check.run_if(input_just_pressed(KeyCode::KeyY)),
            ),
        )
        .add_observer(on_set_placing_block)
        .run();
}

#[derive(Component, Default)]
struct Player;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct PlayerControl;

fn is_player_control_on(input_mode: Res<InputMode>) -> bool {
    *input_mode == InputMode::PlayerControl
}

#[derive(Resource, PartialEq, Eq)]
enum InputMode {
    PlayerControl,
    UiInteraction,
}

fn toggle_input_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut input_mode: ResMut<InputMode>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match *input_mode {
            InputMode::PlayerControl => {
                *input_mode = InputMode::UiInteraction;
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
            }
            InputMode::UiInteraction => {
                *input_mode = InputMode::PlayerControl;
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;

    commands.insert_resource(InputMode::PlayerControl);
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
        BlockPointer::new(16.0),
        PlacingBlock(Block::Sand),
        Collider {
            size: vec3(0.8, 1.9, 0.8),
            anchor: vec3(0.4, 1.7, 0.4),
        },
        Loader::new(64.0, 16.0),
        Transform::from_xyz(5.0, 8.0, 5.0),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 85.0_f32.to_radians(),
            ..default()
        }),
    ));
    let font = TextFont {
        font_size: 12.0,
        ..default()
    };
    commands.spawn((
        InspectUi,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.2)),
        Node {
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            padding: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (Text(default()), font.clone()),
            (Text(default()), font.clone()),
            (Text(default()), font.clone()),
            (Text(default()), font.clone()),
        ],
    ));
}

fn ui_system(
    commands: Commands,
    mut contexts: EguiContexts,
    mut text: Local<String>,
    parser: Local<UserCommandParser>,
) {
    egui::Window::new("console").show(contexts.ctx_mut().unwrap(), |ui| {
        let line = ui.text_edit_singleline(&mut *text);
        if line.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(command) = parser.parse(&*text) {
                command.dispatch(commands);
            }
            text.clear();
        }
    });
}

#[derive(Event, Debug)]
struct SetPlacingBlock(Block);

fn on_set_placing_block(
    change: On<SetPlacingBlock>,
    mut player: Single<&mut PlacingBlock, With<Player>>,
) {
    player.0 = change.0;
}

fn consistency_check(blocks: Query<&ChunkBlocks>) {
    for blocks in blocks {
        blocks.assert_consistency();
    }
    info!("consistency check successful");
}

#[derive(Component)]
struct InspectUi;

macro_rules! text {
    ($text:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        let text = &mut $text.0;
        text.clear();
        write!(text, $fmt $(, $arg)*).unwrap();
    };
}

fn inspect_ui(
    mut texts: Query<&mut Text>,
    root: Single<(Entity, &Children), With<InspectUi>>,
    player: Single<&Transform, With<Player>>,
    time: Res<Time>,
    mut fps: Local<f32>,
) {
    let (_, children) = root.into_inner();

    *fps = 0.99 * *fps + 0.01 * (1.0 / time.delta_secs().max(0.001));
    let mut fps_text = texts.get_mut(children[0]).unwrap();
    text!(fps_text, "fps: {:>4.1}", *fps);

    for (axis, child, value) in [
        ("x", 1, player.translation.x),
        ("y", 2, player.translation.y),
        ("z", 3, player.translation.z),
    ] {
        let mut text = texts.get_mut(children[child]).unwrap();
        text!(text, "{}: {:>+8.3}", axis, value);
    }
}

#[derive(Component)]
struct PlacingBlock(Block);

fn player_acts(
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(&BlockPointer, &PlacingBlock), With<Player>>,
    mut commands: Commands,
) {
    if let Some(Pointing { global, side }) = player.0.pointing {
        if mouse.just_pressed(MouseButton::Left) {
            commands.trigger(Modify::Place {
                global,
                block: Block::Air,
            });
        }
        if mouse.just_pressed(MouseButton::Right) {
            commands.trigger(Modify::Place {
                global: side.neighbour(global),
                block: player.1.0,
            });
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
