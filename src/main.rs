use crate::{
    axis_overlay::AxisOverlayPlugin,
    block::Block,
    chunk_blocks::{ChunkBlocks, chunk_generation, chunk_generation_struct},
    chunk_render::ChunkRenderPlugin,
    chunks::{Chunk, Loader, chunk_indexer, chunks_setup, reset_chunks},
    keybindings::KeyBindings,
    physics::{Collider, PhysicsPlugin, Velocity},
    player_control::{
        PlacingBlock, PlayerControlPlugin, PlayerControlSystemSet, SetPlacingBlock, ToggleFlying,
    },
    pointed_block::{BlockPointer, BlockPointingPlugin},
};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::{
        common_conditions::input_just_pressed,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_fix_cursor_unlock_web::FixPointerUnlockPlugin;
use bevy_framepace::FramepacePlugin;
use command::UserCommandParser;
use lalrpop_util::ParseError;

mod array_queue;
mod axis_overlay;
mod block;
mod chunk_blocks;
mod chunk_render;
mod chunks;
mod command;
mod keybindings;
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
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#bevy-render".to_string()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin::default(),
            FramepacePlugin,
            FixPointerUnlockPlugin,
            AxisOverlayPlugin::<Player>::default(),
            BlockPointingPlugin,
            PhysicsPlugin,
            ChunkRenderPlugin,
            PlayerControlPlugin,
        ))
        .add_systems(Startup, (setup, chunks_setup))
        .add_systems(
            Update,
            (
                // chunk_state_show,
                chunk_indexer,
                chunk_generation,
                chunk_generation_struct,
                leave_player_control,
                reset_chunks.run_if(input_just_pressed(KeyCode::KeyU)),
                type_in_console,
                enter_player_control,
                console_cursor_blink,
                trigger_ui_event,
            ),
        )
        .add_observer(on_set_input_mode)
        .add_observer(on_set_render_distance)
        .configure_sets(
            Update,
            PlayerControlSystemSet.run_if(input_mode_is_player_control),
        )
        .run();
}

pub fn chunk_state_show(
    chunks: Query<(&Chunk, Has<ChunkBlocks>, Has<Mesh3d>)>,
    mut gizmos: Gizmos,
) {
    for (&chunk, has_blocks, has_mesh) in &chunks {
        let color = match (has_blocks, has_mesh) {
            (false, false) => Color::srgb(1.0, 0.0, 0.0),
            (true, false) => Color::srgb(1.0, 1.0, 0.0),
            (true, true) => Color::srgb(0.0, 0.0, 1.0),
            (false, true) => panic!(),
        };
        gizmos.cube(
            Transform {
                translation: chunk.center(),
                rotation: default(),
                scale: Vec3::splat(CHUNK_WIDTH as f32 - 1.0),
            },
            color,
        );
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut fps_overlay: ResMut<FpsOverlayConfig>,
) {
    fps_overlay.text_config.font = asset_server.load("fonts/FiraCode-Medium.ttf");
    commands.insert_resource(ClearColor(Color::srgb(0.7, 0.9, 1.0)));
    commands.insert_resource(KeyBindings::classic_wasd());
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(default(), -0.4, -1.2, 0.0)),
        DirectionalLight { ..default() },
    ));
    commands.spawn((
        Player,
        BlockPointer::new(16.0),
        PlacingBlock(Block::Stone),
        Collider {
            size: vec3(0.8, 1.9, 0.8),
            anchor: vec3(0.4, 1.7, 0.4),
        },
        Loader::new(64.0, 16.0),
        Transform::from_xyz(5.0, 16.0, 10.0).with_rotation(Quat::look_to_lh(Vec3::X, Vec3::Y)),
        Camera { ..default() },
        Camera3d::default(),
        Velocity::default(),
        AmbientLight {
            brightness: 800.0,
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 85.0_f32.to_radians(),
            ..default()
        }),
    ));
    commands.insert_resource(Console::default());
    commands.spawn(ui_entity(asset_server));

    if cfg!(target_arch = "wasm32") {
        commands.insert_resource(InputMode::UiInteraction);
        commands.trigger(SetInputMode(InputMode::UiInteraction));
    } else {
        commands.insert_resource(InputMode::PlayerControl);
        commands.trigger(SetInputMode(InputMode::PlayerControl));
    }
}

fn ui_outisde() -> impl Bundle {
    (
        // BackgroundColor(Color::srgba(0.1, 0.9, 0.1, 0.9)),
        Node {
            flex_grow: 1.0,
            ..default()
        },
        NodeBehind,
        Interaction::None,
    )
}

#[derive(Component, Clone, Debug)]
enum UiEvent {
    Place(Block),
    Fly(bool),
}

fn trigger_ui_event(
    interactable: Query<(&UiEvent, &Interaction), Changed<Interaction>>,
    mut commands: Commands,
) {
    for (event, interaction) in interactable {
        if *interaction == Interaction::Pressed {
            match event {
                UiEvent::Place(block) => {
                    commands.trigger(SetPlacingBlock(*block));
                }
                UiEvent::Fly(flying) => {
                    commands.trigger(ToggleFlying(Some(*flying)));
                }
            }
        }
    }
}

enum GroupButtonPlace {
    First,
    Between,
    Last,
}
fn ui_build_group_button(
    font: TextFont,
    label: &str,
    event: UiEvent,
    place: GroupButtonPlace,
) -> impl Bundle {
    let button_background_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
    let button_border_color = BorderColor::all(Color::srgb(0.27, 0.27, 0.27));
    let border_radius = match place {
        GroupButtonPlace::First => BorderRadius {
            top_left: px(4),
            top_right: px(0),
            bottom_right: px(0),
            bottom_left: px(4),
        },
        GroupButtonPlace::Between => BorderRadius {
            top_left: px(0),
            top_right: px(0),
            bottom_right: px(0),
            bottom_left: px(0),
        },
        GroupButtonPlace::Last => BorderRadius {
            top_left: px(0),
            top_right: px(4),
            bottom_right: px(4),
            bottom_left: px(0),
        },
    };
    let border = match place {
        GroupButtonPlace::First => UiRect {
            left: px(2),
            right: px(1),
            top: px(2),
            bottom: px(2),
        },
        GroupButtonPlace::Between => UiRect {
            left: px(1),
            right: px(1),
            top: px(2),
            bottom: px(2),
        },
        GroupButtonPlace::Last => UiRect {
            left: px(1),
            right: px(2),
            top: px(2),
            bottom: px(2),
        },
    };
    (
        Button,
        Node {
            // width: px(100),
            // height: px(40),
            border,
            padding: UiRect {
                left: px(6),
                right: px(6),
                top: px(2),
                bottom: px(2),
            },
            border_radius,
            // align_items: AlignItems::End,
            ..default()
        },
        button_background_color,
        button_border_color,
        Interaction::None,
        event,
        children![(Text::new(label), font.clone())],
    )
}

fn ui_entity(asset_server: Res<AssetServer>) -> impl Bundle {
    let font = TextFont {
        font_size: 16.0,
        font: asset_server.load("fonts/FiraCode-Medium.ttf"),
        ..default()
    };
    (
        NodeRoot,
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            ..default()
        },
        children![
            (
                Node {
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    ui_outisde(),
                    (
                        // Children(),
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: px(8),
                            padding: UiRect::all(px(2)),
                            // width: px(100),
                            ..default()
                        },
                        // BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                        children![
                            (
                                Node { ..default() },
                                children![
                                    ui_build_group_button(
                                        font.clone(),
                                        "walk",
                                        UiEvent::Fly(false),
                                        GroupButtonPlace::First
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "fly",
                                        UiEvent::Fly(true),
                                        GroupButtonPlace::Last
                                    ),
                                    ui_outisde(),
                                ]
                            ),
                            (
                                Node { ..default() },
                                children![
                                    ui_build_group_button(
                                        font.clone(),
                                        "stone",
                                        UiEvent::Place(Block::Stone),
                                        GroupButtonPlace::First
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "sand",
                                        UiEvent::Place(Block::Sand),
                                        GroupButtonPlace::Between
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "log",
                                        UiEvent::Place(Block::Log),
                                        GroupButtonPlace::Last
                                    ),
                                ]
                            ),
                            ui_outisde(),
                        ]
                    ),
                ]
            ),
            (
                Node { ..default() },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                children![
                    (
                        NodeConsoleInput,
                        Text::new("> "),
                        font.clone(),
                        // ThemedText,
                        Node { ..default() }
                    ),
                    (
                        // NodeConsoleInput,
                        ConsoleCursorTimer::new(),
                        Visibility::Inherited,
                        Text::new("█"),
                        font.clone(),
                        // ThemedText,
                        Node { ..default() }
                    )
                ]
            )
        ],
    )
}

#[derive(Event, Deref)]
struct SetRenderDistance(f32);

fn on_set_render_distance(
    new_render_distance: On<SetRenderDistance>,
    mut player: Single<&mut Loader, With<Player>>,
) {
    player.radius = **new_render_distance;
}

#[derive(Component, Debug, Clone, Copy)]
struct NodeConsoleInput;

#[derive(Component, Debug, Clone, Copy)]
struct NodeRoot;

#[derive(Component, Debug, Clone, Copy)]
struct NodeBehind;
fn enter_player_control(
    query: Query<&Interaction, (With<NodeBehind>, Changed<Interaction>)>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if query.iter().any(|click| *click == Interaction::Pressed)
        || keys.any_just_pressed([KeyCode::Escape, KeyCode::Enter, KeyCode::Tab])
    {
        commands.trigger(SetInputMode(InputMode::PlayerControl));
    }
}

#[derive(Component, Debug, Default)]
struct Player;

#[derive(Resource, PartialEq, Eq)]
enum InputMode {
    PlayerControl,
    UiInteraction,
}

#[derive(Event, Deref)]
struct SetInputMode(InputMode);

fn input_mode_is_player_control(input_mode: Res<InputMode>) -> bool {
    *input_mode == InputMode::PlayerControl
}

fn on_set_input_mode(
    new_input_mode: On<SetInputMode>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut input_mode: ResMut<InputMode>,
    mut root_node: Single<&mut Visibility, With<NodeRoot>>,
) {
    match **new_input_mode {
        InputMode::UiInteraction => {
            info!("input mode set to ui interaction");
            *input_mode = InputMode::UiInteraction;
            cursor_options.grab_mode = CursorGrabMode::None;
            cursor_options.visible = true;
            **root_node = Visibility::Visible;
        }
        InputMode::PlayerControl => {
            info!("input mode set to player control");
            *input_mode = InputMode::PlayerControl;
            cursor_options.grab_mode = CursorGrabMode::Confined;
            cursor_options.visible = false;
            **root_node = Visibility::Hidden;
        }
    }
}

fn leave_player_control(
    mut commands: Commands,
    input_mode: Res<InputMode>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor_options: Single<&CursorOptions, With<PrimaryWindow>>,
) {
    if *input_mode == InputMode::PlayerControl
        && (keys.any_just_pressed([KeyCode::Escape, KeyCode::Slash, KeyCode::KeyT])
            || cursor_options.grab_mode == CursorGrabMode::None)
    {
        commands.trigger(SetInputMode(InputMode::UiInteraction));
    }
}

#[derive(Resource, Default)]
struct Console {
    input: String,
    parser: UserCommandParser,
    completion: Vec<String>,
}

#[derive(Component, Debug, Clone, Deref, DerefMut)]
struct ConsoleCursorTimer(Timer);
impl ConsoleCursorTimer {
    const BLINK_DELAY: f32 = 0.4;
    fn new() -> Self {
        Self(Timer::from_seconds(Self::BLINK_DELAY, TimerMode::Once))
    }
}

fn console_cursor_blink(
    input_mode: Res<InputMode>,
    cursor: Query<(&mut ConsoleCursorTimer, &mut Visibility)>,
    time: Res<Time>,
) {
    if *input_mode == InputMode::UiInteraction {
        for (mut timer, mut visibility) in cursor {
            if timer.tick(time.delta()).just_finished() {
                timer.reset();
                *visibility = match *visibility {
                    Visibility::Visible => Visibility::Visible,
                    Visibility::Hidden => Visibility::Inherited,
                    Visibility::Inherited => Visibility::Hidden,
                };
            }
        }
    }
}

fn type_in_console(
    mut console: ResMut<Console>,
    mut commands: Commands,
    input_mode: Res<InputMode>,
    mut keys: MessageReader<KeyboardInput>,
    mut node: Single<&mut Text, With<NodeConsoleInput>>,
    cursor: Single<(&mut ConsoleCursorTimer, &mut Visibility)>,
) {
    if *input_mode == InputMode::UiInteraction {
        let mut changed = false;
        for key in keys.read() {
            if key.state.is_pressed() {
                match &key.logical_key {
                    Key::Character(c) => {
                        console.input.push_str(c);
                        changed = true;
                    }
                    Key::Backspace => {
                        console.input.pop();
                        changed = true;
                    }
                    Key::Space => {
                        console.input.push(' ');
                        changed = true;
                    }
                    Key::Enter => {
                        match console.parser.parse(&console.input) {
                            Ok(command) => {
                                command.dispatch(&mut commands);
                            }
                            Err(ParseError::UnrecognizedEof {
                                location: _,
                                expected,
                            }) => {
                                println!("{expected:?}");
                            }
                            Err(err) => {
                                error!("{:?}", err);
                            }
                        }
                        console.input.clear();
                        changed = true;
                    }
                    _ => {}
                };
            }
        }
        if changed {
            ***node = format!("> {}", console.input);
            let (mut timer, mut visibility) = cursor.into_inner();
            timer.reset();
            *visibility = Visibility::Inherited;
        }
    }
}
