use geometry::greedymesh::{Chunk, greedy_mesh};

mod geometry;

use bevy::{prelude::*, pbr::wireframe::{WireframePlugin, Wireframe}};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother, controllers::orbit::{OrbitCameraPlugin, OrbitCameraBundle, OrbitCameraController}};

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    // Enables the system that synchronizes your `Transform`s and `LookTransform`s.
    .add_plugin(LookTransformPlugin)
    .add_plugin(WireframePlugin)
    .add_plugin(OrbitCameraPlugin::default())
    .add_startup_system(setup)
    .run();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  commands.spawn(PointLightBundle {
    point_light: PointLight {
      intensity: 15000.0,
      shadows_enabled: true,
      range: 150.0,
      ..default()
    },
    transform: Transform::from_xyz(55.0, 55.0, 55.0),
    ..default()
  });

  commands
    .spawn(Camera3dBundle::default())
    .insert(OrbitCameraBundle::new(
        OrbitCameraController::default(),
        Vec3::new(45.0, 45.0, 45.0),
        Vec3::new(16., 16., 16.),
        Vec3::Y,
    ));

  let test_chunk = Chunk::new(32);
  // test_chunk.print();

  // let start_time = std::time::Instant::now();

  let mesh = greedy_mesh(&test_chunk);

  // println!("Time taken: {}ms", start_time.elapsed().as_millis());

  commands.spawn((
    PbrBundle {
      mesh: meshes.add(mesh),
      material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
      transform: Transform::from_xyz(0.0, 0.0, 0.0),
      ..Default::default()
    },
    // This enables wireframe drawing on this entity
    // Wireframe,
));
}