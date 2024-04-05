use bevy::{prelude::*, window::{PrimaryWindow, CursorGrabMode}, ecs::event::ManualEventReader, input::mouse::{MouseMotion, MouseWheel}};
use bevy_rapier3d::prelude::*;
use strum::EnumCount;

use crate::{prelude::{GROUND_HEIGHT, BlockId, BlockType}, GameState, cam::{MovementSettings, KeyBindings}, Playing, world::Map};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
        .add_systems(Update, (noclip, player_look, player_move, player_laser, set_selected).in_set(Playing))
        .add_systems(OnExit(GameState::GenWorld), set_play_ground)
        .init_resource::<InputState>()
        .init_resource::<SelectedBlock>();
    }
}

fn noclip(
    mut players: Query<&mut RigidBody, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F12) {
        for mut rb in &mut players {
            let next = match *rb {
                RigidBody::Dynamic => RigidBody::KinematicPositionBased,
                _ => RigidBody::Dynamic,
            };
            *rb = next;
        }
    }
}

fn set_play_ground(
    mut players: Query<&mut Transform, With<Player>>,
    map: Res<Map>,
) {
    for mut player in &mut players {
        let pos = BlockId::from_translation(player.translation);
        player.translation = pos.to_vec3();
        player.translation.y = (map.get_max_hight(pos) + 1) as f32;
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlayerCamera(Entity);

#[derive(Resource, Default)]
pub struct SelectedBlock(BlockType);

impl SelectedBlock {
    pub fn set(&mut self, block: BlockType) {
        self.0 = block;
    }

    pub fn get(&self) -> BlockType {
        self.0
    }
}

impl std::fmt::Display for SelectedBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

pub fn spawn_player(
    mut commands: Commands,
) {
    let cam = commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0., 1.75, 0.)),
        ..Default::default()
    }).id();

    commands.spawn((
        RigidBody::Dynamic,
        Player,
        LockedAxes::ROTATION_LOCKED,
        SpatialBundle{
            transform: Transform::from_translation(Vec3::new(0., GROUND_HEIGHT as f32, 0.)),
            ..Default::default()
        },
        Damping{linear_damping: 0.5, angular_damping: 1.},
        PlayerCamera(cam),
    )).with_children(|p| {
        p.spawn((
            Collider::capsule(Vec3::ZERO, Vec3::Y * 1.90, 0.4),
            Ccd{enabled: true},
            SpatialBundle{
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..Default::default()
        }));
    }).add_child(cam);
}

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Resource, Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&mut Transform, &PlayerCamera), With<Player>>,
    mut cams: Query<&mut Transform, (Without<Player>, With<Camera>)>
) {
    if let Ok(window) = primary_window.get_single() {
        for (mut transform, player_camera) in query.iter_mut() {
            let Ok(mut camera_transform) = cams.get_mut(player_camera.0) else {error!("Player has no camera;"); continue;};
            let (_, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            for ev in state.reader_motion.read(&motion) {
                let (mut yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw);
            }
            camera_transform.rotation = Quat::from_axis_angle(Vec3::X, pitch);
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        let key = *key;
                        if key == key_bindings.move_forward {
                            velocity += forward;
                        } else if key == key_bindings.move_backward {
                            velocity -= forward;
                        } else if key == key_bindings.move_left {
                            velocity -= right;
                        } else if key == key_bindings.move_right {
                            velocity += right;
                        } else if key == key_bindings.move_ascend {
                            velocity += Vec3::Y;
                        } else if key == key_bindings.move_descend {
                            velocity -= Vec3::Y;
                        }
                    }
                }

                velocity = velocity.normalize_or_zero();

                transform.translation += velocity * time.delta_seconds() * settings.speed
            }
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

fn player_laser(
    click: Res<ButtonInput<MouseButton>>,
    players: Query<&PlayerCamera, With<Player>>,
    cameras: Query<&GlobalTransform, With<Camera>>,
    context: Res<RapierContext>,
    map: Res<Map>,
    mut gizmos: Gizmos,
    selected: Res<SelectedBlock>,
) {
    for player in &players {
        let Ok(camera) = cameras.get(player.0) else {
            warn!("Player has not camera");
            continue;
        };
        if let Some((_, toi)) = context.cast_ray(camera.translation(), camera.forward(), 5., true, QueryFilter::only_fixed()) {
            let block = BlockId::from_translation(camera.translation() + (camera.forward() * (toi + 0.01)));
            gizmos.cuboid(Transform::from_translation(block.to_vec3()), Color::BLUE);
            if click.just_pressed(MouseButton::Left) {
                map.set_block(block, selected.0);
            } else if click.just_pressed(MouseButton::Right) {
                let block = BlockId::from_translation(camera.translation() + (camera.forward() * (toi - 0.01)));
                map.set_block(block, selected.0);
            }
        }        
    }
}

fn set_selected(
    input: Res<ButtonInput<KeyCode>>,
    mut mouse: EventReader<MouseWheel>,
    mut selected: ResMut<SelectedBlock>,
) {
    for key in input.get_just_pressed() {
        match key {
            KeyCode::Digit1 => selected.set(BlockType::Air),
            KeyCode::Digit2 => selected.set(BlockType::Bedrock),
            KeyCode::Digit3 => selected.set(BlockType::CoalOre),
            KeyCode::Digit4 => selected.set(BlockType::Dirt),
            KeyCode::Digit5 => selected.set(BlockType::Grass),
            KeyCode::Digit6 => selected.set(BlockType::GoldOre),
            KeyCode::Digit7 => selected.set(BlockType::IronOre),
            KeyCode::Digit8 => selected.set(BlockType::Gravel),
            KeyCode::Digit9 => selected.set(BlockType::Sand),
            KeyCode::Digit0 => selected.set(BlockType::Stone),
            _ => {}
        }
    }
    for mouse in mouse.read() {
        if mouse.y >= 1. {
            let mut next = selected.get() as usize + 1;
            if next >= BlockType::COUNT {
                next = 0;
            }
            selected.set(BlockType::from_repr(next).expect("next to be 0..BlockType::COUNT"));
        } else if mouse.y <= -1. {
            let mut next = selected.get() as usize;
            if next == 0 {
                next = BlockType::COUNT - 1;
            } else {
                next -= 1;
            }
            selected.set(BlockType::from_repr(next).expect("next to be 0..BlockType::COUNT"))
        }
    }
}