use bevy::prelude::*;

#[derive(Resource, Reflect, DerefMut, Deref)]
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

pub fn change_view_distance(input: Res<ButtonInput<KeyCode>>, mut view: ResMut<ViewDistance>) {
    if input.just_pressed(KeyCode::NumpadAdd) {
        view.0 += 1;
        println!("view: {}", view.0);
    }
    if input.just_pressed(KeyCode::NumpadSubtract) {
        if view.0 > 3 {
            view.0 -= 1;
        }
        println!("view: {}", view.0);
    }
}
