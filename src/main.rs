use crate::{
    axis_overlay::AxisOverlayPlugin,
    block::Block,
    chunk_blocks::{ChunkBlocks, chunk_generation, chunk_generation_struct},
    chunk_render::ChunkRenderPlugin,
    chunks::{Chunk, Loader, chunk_indexer, chunks_setup},
    keybindings::KeyBindings,
    physics::{Collider, PhysicsPlugin, Velocity},
    player_control::{PlacingBlock, PlayerControlPlugin, PlayerControlSystemSet},
    pointed_block::{BlockPointer, BlockPointingPlugin},
};
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PresentMode, PrimaryWindow},
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
                    // present_mode: PresentMode::Mailbox,
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
                type_in_console.before(leave_player_control),
                leave_player_control,
                // reset_chunks.run_if(input_just_pressed(KeyCode::KeyU)),
                enter_player_control,
                console_cursor_blink,
                trigger_ui_event,
                switch_input_mode
                    .after(leave_player_control)
                    .after(enter_player_control),
            ),
        )
        .add_observer(on_set_render_distance)
        .add_observer(on_console_completion_fill)
        .add_observer(on_console_completion_render)
        .add_observer(on_console_input_change)
        .add_message::<SetInputMode>()
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
    let font = asset_server.load("fonts/FiraCode-Medium.ttf");
    commands.insert_resource(FontStore {
        regular: font.clone(),
    });
    fps_overlay.text_config.font = font.clone();
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
    commands.spawn(ui_entity(font));
    commands.trigger(ConsoleCompletionFill);

    if cfg!(target_arch = "wasm32") {
        commands.insert_resource(InputMode::UiInteraction);
        commands.write_message(SetInputMode(InputMode::UiInteraction));
    } else {
        commands.insert_resource(InputMode::PlayerControl);
        commands.write_message(SetInputMode(InputMode::PlayerControl));
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
    Console(&'static str),
    // Place(Block),
    // Fly(bool),
}

fn trigger_ui_event(
    interactable: Query<(&UiEvent, &Interaction), Changed<Interaction>>,
    mut commands: Commands,
    mut console: ResMut<Console>,
) {
    for (event, interaction) in interactable {
        if *interaction == Interaction::Pressed {
            match event {
                // UiEvent::Place(block) => {
                //     commands.trigger(SetPlacingBlock(*block));
                // }
                // UiEvent::Fly(flying) => {
                //     commands.trigger(ToggleFlying(Some(*flying)));
                // }
                UiEvent::Console(command) => {
                    console.input = command.to_string();
                    commands.trigger(ConsoleInputChanged);
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

#[derive(Resource)]
struct FontStore {
    regular: Handle<Font>,
}

fn ui_entity(font: Handle<Font>) -> impl Bundle {
    let font = TextFont {
        font_size: 16.0,
        font,
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
                                        UiEvent::Console("fly false"),
                                        GroupButtonPlace::First
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "fly",
                                        UiEvent::Console("fly true"),
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
                                        UiEvent::Console("place stone"),
                                        GroupButtonPlace::First
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "sand",
                                        UiEvent::Console("place sand"),
                                        GroupButtonPlace::Between
                                    ),
                                    ui_build_group_button(
                                        font.clone(),
                                        "log",
                                        UiEvent::Console("place log"),
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
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                NodeCompletion,
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
struct NodeCompletion;

#[derive(Component, Debug, Clone, Copy)]
struct NodeConsoleInput;

#[derive(Component, Debug, Clone, Copy)]
struct NodeRoot;

#[derive(Component, Debug, Clone, Copy)]
struct NodeBehind;

fn enter_player_control(
    input_mode: Res<InputMode>,
    query: Query<&Interaction, (With<NodeBehind>, Changed<Interaction>)>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if *input_mode == InputMode::UiInteraction {
        if query.iter().any(|click| *click == Interaction::Pressed)
            || keys.any_just_pressed([KeyCode::Escape, KeyCode::Enter, KeyCode::Tab])
        {
            commands.write_message(SetInputMode(InputMode::PlayerControl));
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
        commands.write_message(SetInputMode(InputMode::UiInteraction));
        commands.trigger(ConsoleCompletionFill);
    }
}

#[derive(Component, Debug, Default)]
struct Player;

#[derive(Resource, PartialEq, Eq)]
enum InputMode {
    PlayerControl,
    UiInteraction,
}

#[derive(Deref, Message)]
struct SetInputMode(InputMode);

fn input_mode_is_player_control(input_mode: Res<InputMode>) -> bool {
    *input_mode == InputMode::PlayerControl
}

fn switch_input_mode(
    mut messages: MessageReader<SetInputMode>,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut input_mode: ResMut<InputMode>,
    mut root_node: Single<&mut Visibility, With<NodeRoot>>,
) {
    for new_input_mode in messages.read() {
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
}
// fn on_set_input_mode(
//     new_input_mode: On<SetInputMode>,
//     mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
//     mut input_mode: ResMut<InputMode>,
//     mut root_node: Single<&mut Visibility, With<NodeRoot>>,
// ) {
//     match **new_input_mode {
//         InputMode::UiInteraction => {
//             info!("input mode set to ui interaction");
//             *input_mode = InputMode::UiInteraction;
//             cursor_options.grab_mode = CursorGrabMode::None;
//             cursor_options.visible = true;
//             **root_node = Visibility::Visible;
//         }
//         InputMode::PlayerControl => {
//             info!("input mode set to player control");
//             *input_mode = InputMode::PlayerControl;
//             cursor_options.grab_mode = CursorGrabMode::Confined;
//             cursor_options.visible = false;
//             **root_node = Visibility::Hidden;
//         }
//     }
// }

#[derive(Resource, Default)]
struct Console {
    input: String,
    parser: UserCommandParser,
    completion: Vec<String>,
    completion_offset: usize,
    is_valid: bool,
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
    player: Single<Entity, With<Player>>,
) {
    match *input_mode {
        InputMode::PlayerControl => {
            keys.clear();
        }
        InputMode::UiInteraction => {
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
                                    command.dispatch(*player, &mut commands);
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
                commands.trigger(ConsoleInputChanged);
            }
        }
    }
}
fn on_console_input_change(
    _: On<ConsoleInputChanged>,
    console: ResMut<Console>,
    mut commands: Commands,
    mut node: Single<&mut Text, With<NodeConsoleInput>>,
    cursor: Single<(&mut ConsoleCursorTimer, &mut Visibility)>,
) {
    ***node = format!("> {}", console.input);
    let (mut timer, mut visibility) = cursor.into_inner();
    timer.reset();
    *visibility = Visibility::Inherited;
    commands.trigger(ConsoleCompletionFill);
}
#[derive(Event, Debug, Clone, Copy)]
struct ConsoleInputChanged;

#[derive(Event, Debug, Clone, Copy)]
struct ConsoleCompletionFill;

#[derive(Event, Debug, Clone, Copy)]
struct ConsoleCompletionRender;

fn on_console_completion_fill(
    _: On<ConsoleCompletionFill>,
    mut console: ResMut<Console>,
    mut commands: Commands,
) {
    match console.parser.parse(&console.input) {
        Ok(_) => {
            console.completion.clear();
            console.is_valid = true;
        }
        Err(ParseError::UnrecognizedEof {
            location: _,
            expected,
        }) => {
            console.completion.clear();
            console.is_valid = false;
            console.completion_offset = console.input.chars().count();
            for expected in expected {
                let Ok(token) = ron::from_str::<String>(&expected) else {
                    continue;
                };
                console.completion.push(token);
            }
        }
        Err(_) => {
            console.is_valid = false;
        }
    }
    commands.trigger(ConsoleCompletionRender);
}

fn on_console_completion_render(
    _: On<ConsoleCompletionRender>,
    console: ResMut<Console>,
    mut commands: Commands,
    completion_ui: Single<Entity, With<NodeCompletion>>,
    font_store: Res<FontStore>,
    node_input: Single<Entity, With<NodeConsoleInput>>,
) {
    let text_color = match console.is_valid {
        true => TextColor(Color::srgb(0.6, 1.0, 0.8)),
        false => TextColor(Color::WHITE),
    };
    commands.entity(*node_input).insert(text_color);

    let offset: String = std::iter::repeat_n(' ', console.completion_offset + 2).collect();
    let parent = commands.entity(*completion_ui).despawn_children().id();
    let text_font = TextFont {
        font_size: 16.0,
        font: font_store.regular.clone(),
        ..default()
    };
    let start: String = console
        .input
        .chars()
        .skip(console.completion_offset)
        .collect();
    for expected in &console.completion {
        if !expected.chars().all(|c| c.is_alphanumeric() || c == '-')
            || expected.starts_with(&start)
        {
            commands.spawn((
                Text::new(format!("{offset}{expected}")),
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                text_font.clone(),
                Node { ..default() },
                ChildOf(parent),
                Interaction::None,
            ));
        }
    }
}
