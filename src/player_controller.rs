use avian3d::prelude::*;
use bevy::{
    color::palettes::css::BLUE,
    input::mouse::{AccumulatedMouseMotion, MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use strum::EnumCount;

use crate::{
    cam::{KeyBindings, MovementSettings},
    physics::PhysicsObject,
    prelude::{BlockId, BlockType, ChunkId, CHUNK_SIZE, GROUND_HEIGHT},
    terrain::Map,
    GameState, Playing,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (noclip, player_look, player_laser, set_selected).in_set(Playing),
            )
            .add_systems(
                Update,
                player_move
                    .chain()
                    .in_set(Playing)
                    .run_if(in_state(PlayerMode::Normal)),
            )
            .add_systems(
                Update,
                noclip_move
                    .in_set(Playing)
                    .run_if(in_state(PlayerMode::NoClip)),
            )
            .init_resource::<SelectedBlock>()
            .init_state::<PlayerMode>();
    }
}

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
enum PlayerMode {
    #[default]
    Normal,
    NoClip,
}

fn noclip(
    mut players: Query<&mut RigidBody, With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<PlayerMode>>,
    mut next: ResMut<NextState<PlayerMode>>,
) {
    if input.just_pressed(KeyCode::F12) {
        match state.get() {
            PlayerMode::Normal => {
                for mut p in &mut players {
                    *p = RigidBody::Kinematic;
                }
                next.set(PlayerMode::NoClip);
            }
            PlayerMode::NoClip => {
                for mut p in &mut players {
                    *p = RigidBody::Dynamic;
                }
                next.set(PlayerMode::Normal);
            }
        }
    }
}

#[derive(Component)]
#[require(PhysicsObject)]
pub struct Player;

#[derive(Component, Deref)]
pub struct PlayerCamera(Entity);

impl PlayerCamera {
    pub fn get(&self) -> Entity {
        self.0
    }
}

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

pub fn spawn_player(mut commands: Commands) {
    let cam = commands
        .spawn((
            Camera3d::default(),
            Transform::from_translation(Vec3::new(0., 1.75, 0.)),
        ))
        .id();

    commands
        .spawn((
            RigidBody::Static,
            Player,
            ExternalImpulse::default(),
            LockedAxes::ROTATION_LOCKED,
            Transform::from_translation(Vec3::new(0., GROUND_HEIGHT as f32, 0.)),
            AngularDamping(1.),
            LinearDamping(0.5),
            PlayerCamera(cam),
        ))
        .with_children(|p| {
            p.spawn((
                Collider::capsule(0.4, 1.),
                Transform::default(),
                Visibility::default(),
            ));
        })
        .add_child(cam);
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut Transform, &PlayerCamera), With<Player>>,
    mut cams: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (mut transform, player_camera) in query.iter_mut() {
            let Ok(mut camera_transform) = cams.get_mut(player_camera.0) else {
                error!("Player has no camera;");
                continue;
            };
            let (_, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            if mouse_motion.delta.length_squared() < 0.1 {
                return;
            }
            let (mut yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
            match window.cursor_options.grab_mode {
                CursorGrabMode::None => (),
                _ => {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());
                    pitch -=
                        (settings.sensitivity * mouse_motion.delta.y * window_scale).to_radians();
                    yaw -=
                        (settings.sensitivity * mouse_motion.delta.x * window_scale).to_radians();
                }
            }
            pitch = pitch.clamp(-1.57, 1.57);

            // Order is important to prevent unintended roll
            transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw);
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
    mut query: Query<(&mut Transform), With<Player>>,
    map: Res<Map>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (mut transform) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor_options.grab_mode {
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
                            velocity += Vec3::Y * 10.;
                        } else if key == key_bindings.move_descend {
                            velocity -= Vec3::Y * 10.;
                        }
                    }
                }
            }

            let next = transform.translation + velocity * time.delta_secs() * settings.speed;
            if !map.get_block(BlockId::from_translation(next)).is_solid() {
                transform.translation = next;
            } else {
                let end = transform.translation + velocity * settings.speed * time.delta_secs();
                let hit = BlockId::from_translation(next).as_vec3();
                let current = BlockId::from_translation(transform.translation).as_vec3();
                let mut start = end;
                let dif = hit - current;
                if dif.x != 0. {
                    start.x = transform.translation.x;
                    let fin = start + (start - transform.translation);
                    if !map.get_block(BlockId::from_translation(fin)).is_solid() {
                        transform.translation = fin;
                    }
                }

                if dif.z != 0. {
                    start.z = transform.translation.z;
                    let fin = start + (start - transform.translation);
                    if !map.get_block(BlockId::from_translation(fin)).is_solid() {
                        transform.translation = fin;
                    }
                }
            };
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

/// Handles keyboard input and movement
fn noclip_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut query: Query<(&mut Transform), With<Player>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor_options.grab_mode {
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

                transform.translation += velocity * time.delta_secs() * settings.speed
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
    map: Res<Map>,
    mut gizmos: Gizmos,
    selected: Res<SelectedBlock>,
    mut error: Local<bool>,
) {
    for player in &players {
        if !*error {
            error!("add back raycast");
            *error = true;
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
