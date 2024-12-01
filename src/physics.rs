use bevy::prelude::*;

use crate::{
    player_controller::{Player, PlayerCamera},
    prelude::BlockId,
    terrain::Map,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsDebugRendering>()
            .add_systems(FixedUpdate, gravity)
            .add_systems(PreUpdate, update_grounded)
            .add_systems(Update, (apply_velocity, give_big_velocity, toggle_debug))
            .add_systems(
                PostUpdate,
                (
                    render_player_collider
                        .run_if(|r: Res<PhysicsDebugRendering>| r.render_colliders),
                    render_velocity.run_if(|r: Res<PhysicsDebugRendering>| r.render_velocity),
                ),
            );
    }
}

#[derive(Resource, Default)]
struct PhysicsDebugRendering {
    enabled: bool,
    render_colliders: bool,
    render_velocity: bool,
}

fn toggle_debug(input: Res<ButtonInput<KeyCode>>, mut settings: ResMut<PhysicsDebugRendering>) {
    if input.just_pressed(KeyCode::F3) {
        if settings.enabled {
            settings.enabled = false;
            settings.render_colliders = false;
            settings.render_velocity = false;
            return;
        }
        settings.enabled = true;
        if input.pressed(KeyCode::KeyC) {
            settings.render_colliders = true;
        }
        if input.pressed(KeyCode::KeyV) {
            settings.render_velocity = true;
        }
    }
}

#[derive(Component, Default)]
#[require(Transform, Velocity, PhysicsOutput)]
pub struct PhysicsObject;

#[derive(Component, Default)]
pub struct PhysicsOutput {
    grounded: bool,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(Vec3);

fn update_grounded(map: Res<Map>, mut objects: Query<(&Transform, &mut PhysicsOutput)>) {
    for (transform, mut output) in &mut objects {
        let next = transform.translation - Vec3::new(0., 0.05, 0.);
        output.grounded = map.get_block(BlockId::from_translation(next)).is_solid();
    }
}

fn gravity(
    mut players: Query<(&mut Velocity, &PhysicsOutput), With<PhysicsObject>>,
    time: Res<Time>,
) {
    for (mut velocity, output) in &mut players {
        if !output.grounded {
            velocity.y += -9.8 * time.delta_secs();
        } else {
            velocity.y = 0.;
        }
    }
}

pub fn apply_velocity(
    map: Res<Map>,
    mut players: Query<(&mut Transform, &mut Velocity), With<PhysicsObject>>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    mut last: Local<Transform>,
) {
    gizmos.cuboid(*last, Color::srgb(1., 0., 0.));
    for (mut transform, mut velocity) in &mut players {
        gizmos.cuboid(
            Transform::from_translation(BlockId::from_translation(transform.translation).as_vec3()),
            Color::srgb(0., 0., 1.),
        );
        if velocity.length_squared() < 0.05 {
            continue;
        }
        let mut next = transform.translation + velocity.0 * time.delta_secs();
        let block = BlockId::from_translation(next);
        let current = BlockId::from_translation(transform.translation);
        let Some((face, path)) = ray_path(
            transform.translation,
            velocity.0,
            (block - current).length_squared() as f32,
            block,
        ) else {
            error!("No Path found");
            continue;
        };
        velocity.0 *= 0.999;
        if !path.is_empty() {
            let mut hit = None;
            for block in path {
                if map.get_block(block).is_solid() {
                    hit = Some(block);
                    break;
                }
            }
            if let Some(hit) = hit {
                if face.x > 0 {
                    next.x = hit.x as f32 + 0.50;
                } else if face.x < -0 {
                    next.x = hit.x as f32 - 0.50;
                }

                if face.y > 0 {
                    next.y = hit.y as f32 + 0.50;
                } else if face.y < 0 {
                    next.y = hit.y as f32 - 0.50;
                }

                if face.z > 0 {
                    next.z = hit.z as f32 + 0.50;
                } else if face.z < -0 {
                    next.z = hit.z as f32 - 0.50;
                }
                velocity.0 = Vec3::ZERO;
                last.translation = hit.as_vec3();
            };
        }
        transform.translation = next;
    }
}

fn render_player_collider(mut gizmos: Gizmos, players: Query<&Transform, With<Player>>) {
    let player = players.single();
    let mut t = player.with_scale(Vec3::new(1., 2., 1.));
    t.translation.y += 1.;
    gizmos.cuboid(t, Color::srgb(1., 0., 1.));
}

fn render_velocity(mut gizmos: Gizmos, objects: Query<(&GlobalTransform, &Velocity)>) {
    for (transform, velocity) in &objects {
        let end = transform.translation() + velocity.0;
        gizmos.line(transform.translation(), end, Color::srgb(0., 1., 0.));
    }
}

fn give_big_velocity(
    mut player: Query<(&PlayerCamera, &mut Velocity), With<Player>>,
    transforms: Query<&GlobalTransform>,
    input: Res<ButtonInput<MouseButton>>,
    mut power: Local<f32>,
    time: Res<Time>,
) {
    for (transform, mut velocity) in &mut player {
        if input.pressed(MouseButton::Right) {
            *power += time.delta_secs();
        }
        if input.just_released(MouseButton::Right) {
            let transform = transforms
                .get(transform.get())
                .expect("Player to have camera");
            velocity.0 += transform.forward().as_vec3() * *power * 100.;
            *power = 0.;
        }
    }
}

fn ray_path(
    mut origin: Vec3,
    mut direction: Vec3,
    radius: f32,
    end: BlockId,
) -> Option<(IVec3, Vec<BlockId>)> {
    #[inline]
    fn int_bound(s: f32, ds: f32) -> f32 {
        if (ds < 0.) {
            int_bound(-s, -ds)
        } else {
            let s = (s % 1. + 1.) % 1.;
            (1. - s) / ds
        }
    }
    let max = (BlockId::from_translation(origin) - end).length_squared() as usize;
    direction = direction.normalize();
    let step_x = direction.x.signum();
    let step_y = direction.y.signum();
    let step_z = direction.z.signum();
    let mut t_max_x = int_bound(origin.x, direction.x);
    let mut t_max_y = int_bound(origin.y, direction.y);
    let mut t_max_z = int_bound(origin.z, direction.z);
    let t_delta_x = step_x / direction.x;
    let t_delta_y = step_y / direction.y;
    let t_delta_z = step_z / direction.z;

    if direction.x == 0. && direction.y == 0. && direction.z == 0. {
        error!("Raycast in zero direction");
        return None;
    }
    let mut out = Vec::new();
    let mut face = IVec3::ZERO;
    while out.len() < max {
        if (t_max_x < t_max_y) {
            if (t_max_x < t_max_z) {
                if t_delta_x > radius {
                    break;
                };
                // Update which cube we are now in.
                origin.x += step_x;
                // Adjust tMaxX to the next X-oriented boundary crossing.
                t_max_x += t_delta_x;
                // Record the normal vector of the cube face we entered.
                face = IVec3::new(-step_x as i32, 0, 0);
                let add = BlockId::from_translation(origin);
                out.push(add);
                if add == end {
                    break;
                }
            } else {
                if (t_max_z > radius) {
                    break;
                }
                origin.z += step_z;
                t_max_z += t_delta_z;
                face = IVec3::new(0, 0, -step_z as i32);
                let add = BlockId::from_translation(origin);
                out.push(add);
                if add == end {
                    break;
                }
            }
        } else {
            if (t_max_y < t_max_z) {
                if (t_max_y > radius) {
                    break;
                };
                origin.y += step_y;
                t_max_y += t_delta_y;
                face = IVec3::new(0, -step_y as i32, 0);
                let add = BlockId::from_translation(origin);
                out.push(add);
                if add == end {
                    break;
                }
            } else {
                // Identical to the second case, repeated for simplicity in
                // the conditionals.
                if (t_max_z > radius) {
                    break;
                };
                origin.z += step_z;
                t_max_z += t_delta_z;
                face = IVec3::new(0, 0, -step_z as i32);
                let add = BlockId::from_translation(origin);
                out.push(add);
                if add == end {
                    break;
                }
            }
        }
    }
    Some((face, out))
}
