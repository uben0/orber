use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub sprint: KeyCode,
    pub jump: KeyCode,
    pub croutch: KeyCode,
}

impl KeyBindings {
    pub fn classic_wasd() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            sprint: KeyCode::ShiftLeft,
            jump: KeyCode::Space,
            croutch: KeyCode::ControlLeft,
        }
    }
    pub fn typist_esdf() -> Self {
        Self {
            move_forward: KeyCode::KeyE,
            move_backward: KeyCode::KeyD,
            move_left: KeyCode::KeyS,
            move_right: KeyCode::KeyF,
            sprint: KeyCode::KeyA,
            jump: KeyCode::Space,
            croutch: KeyCode::KeyZ,
        }
    }
}
