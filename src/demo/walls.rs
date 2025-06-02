use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_int_cell::<WallBundle>(1);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Bundle, LdtkIntCell)]
pub struct WallBundle {
    pub wall: Wall,
    pub collider: Collider,
    pub rigid_body: RigidBody,
}

impl Default for WallBundle {
    fn default() -> Self {
        Self {
            wall: Wall,
            collider: Collider::rectangle(16., 16.), // Default size for wall collision),
            rigid_body: RigidBody::Static,
        }
    }
}
