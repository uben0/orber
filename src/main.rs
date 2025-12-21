use crate::axis_overlay::AxisOverlayPlugin;
use crate::block::Block;
use crate::chunk_blocks::chunk_generation;
use crate::chunk_render::ChunkRenderPlugin;
use crate::chunks::{Loader, chunk_indexer, chunks_setup};
use crate::command::UserCommandParser;
use crate::physics::{Collider, PhysicsPlugin};
use crate::player_control::PlayerControlPlugin;
use crate::pointed_block::{BlockPointer, BlockPointingPlugin};
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy::{prelude::*, window::CursorGrabMode};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_framepace::FramepacePlugin;
use std::fmt::Write;

mod array_queue;
mod axis_overlay;
mod block;
mod chunk_blocks;
mod chunk_render;
mod chunks;
mod command;
mod material;
mod octahedron;
mod physics;
mod player_control;
mod pointed_block;
mod ray_travel;
mod spacial;
mod terrain;

const CHUNK_WIDTH: i32 = 32;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            AxisOverlayPlugin::<Player>::default(),
            BlockPointingPlugin,
            PhysicsPlugin,
            ChunkRenderPlugin,
            EguiPlugin::default(),
            PlayerControlPlugin,
        ))
        .add_systems(EguiPrimaryContextPass, console)
        .add_systems(Startup, (setup, chunks_setup))
        .add_systems(
            Update,
            (
                chunk_indexer,
                chunk_generation,
                // chunk_state_show,
                toggle_input_mode,
                inspect_ui,
            ),
        )
        .add_observer(on_set_placing_block)
        .add_observer(on_set_input_mode)
        .add_observer(on_set_render_distance)
        .run();
}

#[derive(Component, Default)]
struct Player;

#[derive(Resource, PartialEq, Eq)]
enum InputMode {
    PlayerControl,
    UiInteraction,
}

#[derive(Event, Deref)]
struct SetInputMode(InputMode);

fn on_set_input_mode(
    new_input_mode: On<SetInputMode>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut input_mode: ResMut<InputMode>,
) {
    match **new_input_mode {
        InputMode::UiInteraction => {
            *input_mode = InputMode::UiInteraction;
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
        }
        InputMode::PlayerControl => {
            *input_mode = InputMode::PlayerControl;
            cursor_options.grab_mode = CursorGrabMode::Locked;
            cursor_options.visible = false;
        }
    }
}

fn toggle_input_mode(
    mut commands: Commands,
    input_mode: Res<InputMode>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.trigger(SetInputMode(match *input_mode {
            InputMode::PlayerControl => InputMode::UiInteraction,
            InputMode::UiInteraction => InputMode::PlayerControl,
        }))
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
    commands.insert_resource(ConsoleState {
        input: String::new(),
        active: false,
        focus: false,
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

#[derive(Event, Deref)]
struct SetRenderDistance(f32);

fn on_set_render_distance(
    new_render_distance: On<SetRenderDistance>,
    mut player: Single<&mut Loader, With<Player>>,
) {
    player.radius = **new_render_distance;
}

#[derive(Resource)]
struct ConsoleState {
    input: String,
    active: bool,
    focus: bool,
}

fn console(
    mut console: ResMut<ConsoleState>,
    mut commands: Commands,
    mut contexts: EguiContexts,
    parser: Local<UserCommandParser>,
) {
    if !console.active {
        return;
    }
    egui::Window::new("console").show(contexts.ctx_mut().unwrap(), |ui| {
        let line = ui.text_edit_singleline(&mut console.input);
        if console.focus {
            console.focus = false;
            line.request_focus();
        }
        if line.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            console.active = false;
            commands.trigger(SetInputMode(InputMode::PlayerControl));
            if let Ok(command) = parser.parse(&console.input) {
                command.dispatch(commands);
            }
            console.input.clear();
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
