#![deny(clippy::all)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cargo_common_metadata,
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::multiple_crate_versions,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::must_use_candidate,
    clippy::enum_glob_use
)]
#![feature(is_some_and)]

use bevy::{
    math::Vec3Swizzles,
    prelude::{shape::Plane, *},
    window::WindowResolution,
};
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DefaultPickingPlugins, PickingCameraBundle, PickingRaycastSet,
};
use bevy_mod_raycast::Intersection;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use common::NoHighlightPickableBundle;
use leafwing_input_manager::prelude::*;

mod common;

pub const CLEAR: Color = Color::BLACK;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::WHITE,
        })
        .insert_resource(ClearColor(CLEAR))
        // External plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(
                            WINDOW_HEIGHT * RESOLUTION,
                            WINDOW_HEIGHT,
                        )
                        .with_scale_factor_override(1.),
                        title: "ARPG".to_string(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(DebugLinesPlugin::with_depth_test(false))
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(InputManagerPlugin::<PlayerActions>::default())
        // .add_plugin(WorldInspectorPlugin::new())
        // Internal plugins
        .add_startup_system(startup)
        .add_systems((
            CharacterAnimation::on_added,
            CharacterState::moving_to,
            CharacterState::on_changed,
            PlayerActions::move_to,
        ))
        .run();
}

const MAP_WIDTH: u32 = 4 * 4; // Originally 30 * 4
const MAP_HEIGHT: u32 = 4 * 4; // Originally 22 * 4

fn startup(
    mut commands: Commands,
    ass: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    // Perfect isometric rotation
    let mut transform =
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0., 45_f32.to_radians(), 0.));
    transform.rotate_local_x(-35.264_f32.to_radians());
    // Imperfect camera placement wherever
    transform.translation = Vec3::new(
        MAP_WIDTH as f32 * 1.2,
        MAP_WIDTH as f32 * 0.5,
        MAP_HEIGHT as f32 * 1.2,
    );
    let camera = Camera3dBundle {
        transform,
        ..default()
    };

    commands.spawn((camera, PickingCameraBundle::default()));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane {
                    size: MAP_WIDTH as f32,
                    ..default()
                }
                .into(),
            ),
            material: mats.add(Color::DARK_GREEN.into()),
            transform: Transform::from_xyz(MAP_WIDTH as f32 / 2., 0., MAP_HEIGHT as f32 / 2.),
            ..default()
        },
        NoHighlightPickableBundle::default(),
    ));

    commands.spawn((
        SceneBundle {
            scene: ass.load("player.glb#Scene0"),
            transform: Transform::from_xyz(MAP_WIDTH as f32 / 2., 1., MAP_HEIGHT as f32 / 2.),
            ..default()
        },
        CharacterAnimation {
            idle_animation: ass.load("idle.glb#Animation0"),
            running_animation: ass.load("run.glb#Animation0"),
            ..default()
        },
        CharacterState::Idle,
        InputManagerBundle {
            input_map: InputMap::new([(MouseButton::Left, PlayerActions::MoveTo)]),
            ..default()
        },
    ));
}

#[derive(Component, Default)]
pub struct CharacterAnimation {
    idle_animation: Handle<AnimationClip>,
    running_animation: Handle<AnimationClip>,
    animation_player: Option<Entity>,
}

impl CharacterAnimation {
    pub fn on_added(
        mut animation_q: Query<&mut CharacterAnimation>,
        parent_q: Query<&Parent, With<Children>>,
        mut animation_player_q: Query<
            (Entity, &Parent, &mut AnimationPlayer),
            Added<AnimationPlayer>,
        >,
    ) {
        for (player_entity, parent, mut player) in &mut animation_player_q {
            if let Ok(parent) = parent_q.get(**parent) {
                if let Ok(mut animation) = animation_q.get_mut(**parent) {
                    animation.animation_player = Some(player_entity);
                    player.play(animation.idle_animation.clone()).repeat();
                }
            }
        }
    }
}

#[derive(Component)]
enum CharacterState {
    Idle,
    MovingTo(Vec2),
}

const BASE_CHARACTER_SPEED: f32 = 4.;

impl CharacterState {
    fn moving_to(time: Res<Time>, mut movings: Query<(&mut Transform, &mut CharacterState)>) {
        for (mut pos, mut action) in &mut movings {
            if let CharacterState::MovingTo(destination) = *action {
                let move_vector = destination - pos.translation.xz();
                if move_vector.length_squared() > 0.001_f32.powf(2.) {
                    let direction = move_vector.normalize_or_zero();
                    let movement = (direction * BASE_CHARACTER_SPEED * time.delta_seconds())
                        .clamp_length_max(move_vector.length())
                        .extend(0.)
                        .xzy();
                    pos.translation += movement;
                    pos.look_to(direction.extend(0.).xzy(), Vec3::Y);
                    pos.rotate_local_y(180_f32.to_radians());
                    if pos.translation.xz().distance_squared(destination) < 0.001 {
                        *action = CharacterState::Idle;
                    }
                } else {
                    *action = CharacterState::Idle;
                }
            }
        }
    }

    fn on_changed(
        characters: Query<(&CharacterState, &CharacterAnimation), Changed<CharacterState>>,
        mut players: Query<&mut AnimationPlayer>,
    ) {
        for (state, animation) in &characters {
            if let Some(player) = animation.animation_player {
                if let Ok(mut player) = players.get_mut(player) {
                    player
                        .play(match state {
                            CharacterState::Idle => animation.idle_animation.clone(),
                            CharacterState::MovingTo(_) => animation.running_animation.clone(),
                        })
                        .repeat();
                }
            }
        }
    }
}

#[derive(Actionlike, Clone, Copy)]
enum PlayerActions {
    MoveTo,
}

impl PlayerActions {
    fn move_to(
        mut player: Query<(&ActionState<PlayerActions>, &mut CharacterState)>,
        intersection: Query<&Intersection<PickingRaycastSet>>,
    ) {
        if let Ok(intersection) = intersection.get_single() {
            for (action_state, mut character_state) in &mut player {
                if action_state.pressed(PlayerActions::MoveTo) {
                    if let Some(intersection_pos) = intersection.position() {
                        *character_state = CharacterState::MovingTo(intersection_pos.xz());
                    }
                }
            }
        }
    }
}
