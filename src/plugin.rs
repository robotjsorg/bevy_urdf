use bevy::prelude::*;
use bevy_rapier3d::prelude::RapierRigidBodySet;
use rapier3d::prelude::Collider;
use urdf_rs::{Geometry, Pose};

use crate::{
    events::{
        handle_load_robot, handle_spawn_robot, handle_wait_robot_loaded, LoadRobot, RobotLoaded,
        SpawnRobot, UrdfRobotRigidBodyHandle, WaitRobotLoaded,
    },
    urdf_asset_loader::{self, UrdfAsset},
};
pub struct UrdfPlugin;

impl Plugin for UrdfPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<urdf_asset_loader::RpyAssetLoader>()
            .add_event::<SpawnRobot>()
            .add_event::<WaitRobotLoaded>()
            .add_event::<LoadRobot>()
            .add_event::<RobotLoaded>()
            .add_systems(Update, sync_robot_geometry)
            .add_systems(
                Update,
                (
                    handle_spawn_robot,
                    handle_load_robot,
                    handle_wait_robot_loaded,
                ),
            )
            .init_asset::<urdf_asset_loader::UrdfAsset>();
    }
}

pub fn extract_robot_geometry(
    robot: &UrdfAsset,
) -> Vec<(usize, Option<Geometry>, Pose, Option<Collider>)> {
    let mut result: Vec<(usize, Option<Geometry>, Pose, Option<Collider>)> = Vec::new();
    for (i, link) in robot.robot.links.iter().enumerate() {
        let colliders = robot.urdf_robot.links[i].colliders.clone();
        let collider = if colliders.len() == 1 {
            Some(colliders[0].clone())
        } else {
            None
        };

        let geometry = if !link.visual.is_empty() {
            Some(link.visual[0].geometry.clone())
        } else {
            None
        };
        let inertia_origin = link.inertial.origin.clone();

        result.push((i, geometry.clone(), inertia_origin.clone(), collider));
    }

    result
}

fn sync_robot_geometry(
    mut q_rapier_robot_bodies: Query<(Entity, &mut Transform, &mut UrdfRobotRigidBodyHandle)>,
    q_rapier_rigid_body_set: Query<(&RapierRigidBodySet,)>,
) {
    for rapier_rigid_body_set in q_rapier_rigid_body_set.iter() {
        for (_, mut transform, body_handle) in q_rapier_robot_bodies.iter_mut() {
            if let Some(robot_body) = rapier_rigid_body_set.0.bodies.get(body_handle.0) {
                let rapier_pos = robot_body.position();

                let rapier_rot = rapier_pos.rotation;

                let quat_fix = Quat::from_rotation_z(std::f32::consts::PI);
                let bevy_quat = quat_fix
                    * Quat::from_array([rapier_rot.i, rapier_rot.j, rapier_rot.k, rapier_rot.w]);

                let rapier_vec = Vec3::new(
                    rapier_pos.translation.x,
                    rapier_pos.translation.y,
                    rapier_pos.translation.z,
                );
                let bevy_vec = quat_fix.mul_vec3(rapier_vec);
                // bevy_vec.y *= -1.0;

                *transform = Transform::from_translation(bevy_vec).with_rotation(bevy_quat);
            }
        }
    }
}
