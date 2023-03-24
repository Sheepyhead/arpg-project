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
    prelude::{shape::Plane, *},
    window::WindowResolution,
};
use bevy_prototype_debug_lines::DebugLinesPlugin;

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
        // .add_plugin(WorldInspectorPlugin::new())
        // Internal plugins
        .add_startup_system(startup)
        .add_system(CharacterAnimation::on_added)
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

    commands.spawn(camera);

    commands.spawn((PbrBundle {
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
    },));

    commands.spawn((
        SceneBundle {
            scene: ass.load("player.glb#Scene0"),
            transform: Transform::from_xyz(MAP_WIDTH as f32 / 2., 1., MAP_HEIGHT as f32 / 2.),
            ..default()
        },
        CharacterAnimation {
            idle_animation: ass.load("idle.glb#Animation0"),
            ..default()
        },
    ));
}

#[derive(Component, Default)]
pub struct CharacterAnimation {
    idle_animation: Handle<AnimationClip>,
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
