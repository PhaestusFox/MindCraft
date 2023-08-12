use bevy::prelude::*;


#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ViewDistance(pub i32);

impl ViewDistance {
    /// takes 0..1 where 0 is 5 and 1 is 25
    pub fn set(&mut self, val: f32) {
        let val = val.clamp(0., 1.);
        self.0 = 5 + (val * 20.) as i32;
    }
    /// return 0..1 where 0 is 5 and 1 is 25
    pub fn get(&self) -> f32 {
        (self.0 - 5) as f32 / 20.
    }
}

impl Default for ViewDistance {
    fn default() -> Self {
        ViewDistance(3)
    }
}