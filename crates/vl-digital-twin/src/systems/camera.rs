//! Scene and camera setup systems.

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

/// Startup: spawn Camera3d with PanOrbitCamera controls.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 12.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera {
            radius: Some(25.0),
            ..default()
        },
    ));
}

/// Startup: add directional light + ambient light.
pub fn setup_lights(mut commands: Commands) {
    // Directional "sun" light
    commands.spawn((
        DirectionalLight {
            illuminance:                       10_000.0,
            shadows_enabled:                   true,
            affects_lightmapped_mesh_diffuse:  true,
            ..default()
        },
        Transform::from_xyz(5.0, 15.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Soft ambient fill
    commands.insert_resource(GlobalAmbientLight {
        color:                      Color::WHITE,
        brightness:                 300.0,
        affects_lightmapped_meshes: true,
    });
}

/// Startup: create a ground plane and some placeholder device meshes.
pub fn setup_scene(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.25, 0.25),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
