use bevy::prelude::*;


#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ViewDistance(pub i32);

impl Default for ViewDistance {
    fn default() -> Self {
        ViewDistance(3)
    }
}