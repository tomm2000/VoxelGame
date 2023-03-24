use bevy::prelude::{system_adapter::new, *};
use bevy::render::mesh::{self, PrimitiveTopology};
use std::{
  cmp::max,
  ops::{AddAssign, Mul},
};

pub enum VoxelFaceDirection {
  Front,
  Back,
  Left,
  Right,
  Top,
  Bottom,
}

impl VoxelFaceDirection {
  pub fn get_normal(&self) -> Vec3 {
    match self {
      VoxelFaceDirection::Front => Vec3::new(0.0, 0.0, 1.0),
      VoxelFaceDirection::Back => Vec3::new(0.0, 0.0, -1.0),
      VoxelFaceDirection::Left => Vec3::new(-1.0, 0.0, 0.0),
      VoxelFaceDirection::Right => Vec3::new(1.0, 0.0, 0.0),
      VoxelFaceDirection::Top => Vec3::new(0.0, 1.0, 0.0),
      VoxelFaceDirection::Bottom => Vec3::new(0.0, -1.0, 0.0),
    }
  }

  pub fn get_iter() -> Vec<VoxelFaceDirection> {
    vec![
      VoxelFaceDirection::Front,
      VoxelFaceDirection::Back,
      VoxelFaceDirection::Left,
      VoxelFaceDirection::Right,
      VoxelFaceDirection::Top,
      VoxelFaceDirection::Bottom,
    ]
  }
}

#[derive(Component)]
pub struct Voxel {
  position: Vec3,
  is_solid: bool,
}

impl Voxel {
  pub fn new(position: Vec3, is_solid: bool) -> Self {
    Self { position, is_solid }
  }

  pub fn get_face(&self, direction: VoxelFaceDirection) -> MeshFace {
    let mut face = match direction {
      VoxelFaceDirection::Front => MeshFace::new([
        self.position + Vec3::new(0.0, 0.0, 1.0),
        self.position + Vec3::new(1.0, 0.0, 1.0),
        self.position + Vec3::new(1.0, 1.0, 1.0),
        self.position + Vec3::new(0.0, 1.0, 1.0),
      ]),
      VoxelFaceDirection::Back => MeshFace::new([
        self.position + Vec3::new(0.0, 0.0, 0.0),
        self.position + Vec3::new(1.0, 0.0, 0.0),
        self.position + Vec3::new(1.0, 1.0, 0.0),
        self.position + Vec3::new(0.0, 1.0, 0.0),
      ]),
      VoxelFaceDirection::Left => MeshFace::new([
        self.position + Vec3::new(0.0, 0.0, 0.0),
        self.position + Vec3::new(0.0, 0.0, 1.0),
        self.position + Vec3::new(0.0, 1.0, 1.0),
        self.position + Vec3::new(0.0, 1.0, 0.0),
      ]),
      VoxelFaceDirection::Right => MeshFace::new([
        self.position + Vec3::new(1.0, 0.0, 0.0),
        self.position + Vec3::new(1.0, 0.0, 1.0),
        self.position + Vec3::new(1.0, 1.0, 1.0),
        self.position + Vec3::new(1.0, 1.0, 0.0),
      ]),
      VoxelFaceDirection::Top => MeshFace::new([
        self.position + Vec3::new(0.0, 1.0, 0.0),
        self.position + Vec3::new(1.0, 1.0, 0.0),
        self.position + Vec3::new(1.0, 1.0, 1.0),
        self.position + Vec3::new(0.0, 1.0, 1.0),
      ]),
      VoxelFaceDirection::Bottom => MeshFace::new([
        self.position + Vec3::new(0.0, 0.0, 0.0),
        self.position + Vec3::new(1.0, 0.0, 0.0),
        self.position + Vec3::new(1.0, 0.0, 1.0),
        self.position + Vec3::new(0.0, 0.0, 1.0),
      ]),
    };

    // adjust the face to the voxel's position
    for vertex in face.vertices.iter_mut() {
      vertex.add_assign(self.position);
    }

    return face;
  }
}

pub struct VoxelChunk {
  pub size: Vec3,
  pub voxels: Vec<Vec<Vec<Voxel>>>,
}

#[derive(Debug)]
pub struct MeshFace {
  pub vertices: [Vec3; 4],
}

impl MeshFace {
  pub fn new(vertices: [Vec3; 4]) -> Self {
    Self { vertices }
  }

  pub fn get_center(&self) -> Vec3 {
    let mut center = Vec3::new(0.0, 0.0, 0.0);
    for vertex in self.vertices.iter() {
      center.add_assign(*vertex)
    }
    center / self.vertices.len() as f32
  }

  pub fn get_edge_normal(&self, edge: (Vec3, Vec3)) -> Vec3 {
    let center = self.get_center();

    let (v1, v2) = edge;
    let mid_point = v1.lerp(v2, 0.5);
    let center_to_midpoint = mid_point - center;

    center_to_midpoint.normalize()
  }

  /**
   * Expands the face in the given direction by the given amount.
   * note: this function always grows the face, if you want to shrink use the shrink function
   * note: passing a negative amount will cause the opposite direction to be expanded
   */
  pub fn expand(&mut self, direction: Vec3, amount: i32) {
    let edge = find_face_edge(self, |(a, b)| {
      let normal = self.get_edge_normal((a, b));
      if normal == (direction * amount as f32).normalize() {
        return true;
      }

      return false;
    });

    if edge.is_none() {
      return;
    }
    let (a, b) = edge.unwrap();

    let mut new_vertices = Vec::new();

    for vertex in self.vertices.iter() {
      if *vertex == a || *vertex == b {
        // println!("expanding vertex: {:?} | {:?}", vertex, *vertex + (direction * amount as f32));
        new_vertices.push(*vertex + (direction * amount as f32));
      } else {
        new_vertices.push(*vertex);
      }
    }

    self.vertices = [
      new_vertices[0],
      new_vertices[1],
      new_vertices[2],
      new_vertices[3],
    ];
  }

  pub fn size(&self) -> Vec3 {
    Vec3::new(
      (self.vertices[0].x - self.vertices[1].x).abs(),
      (self.vertices[0].y - self.vertices[3].y).abs(),
      (self.vertices[0].z - self.vertices[1].z).abs(),
    )
  }

  // pub fn merge(face1: &mut MeshFace, face2: &mut MeshFace) {

  // }
}

pub fn generate_chunk_mesh(chunk: &VoxelChunk) -> Vec<MeshFace> {
  let mut faces = Vec::new();

  // cycle trough each normal direction
  for direction in VoxelFaceDirection::get_iter() {
    let normal = direction.get_normal();

    // cycle trough each voxel
    // for z in 0..chunk.size.z as usize {
    let x = 0;
    let y = 0;

    let origin_x = 0;
    let origin_y = 0;

    let face_building = false;

    // cycle trough each voxel
    while y < chunk.size.y as usize {
      while x < chunk.size.x as usize {
        // if the voxel is not solid
        if !chunk.voxels[x][y][0].is_solid && face_building {

        }
      }
    }
    // }
  }

  return faces;
}

pub fn find_face_edge<F>(face: &MeshFace, func: F) -> Option<(Vec3, Vec3)>
where
  F: Fn((Vec3, Vec3)) -> bool,
{
  for i in 0..face.vertices.len() {
    let a = Vec3::new(
      face.vertices[i][0],
      face.vertices[i][1],
      face.vertices[i][2],
    );
    let b = Vec3::new(
      face.vertices[(i + 1) % face.vertices.len()][0],
      face.vertices[(i + 1) % face.vertices.len()][1],
      face.vertices[(i + 1) % face.vertices.len()][2],
    );

    if func((a, b)) {
      return Some((a, b));
    }
  }

  return None;
}

pub fn iterate_face_edges<F>(face: &MeshFace, func: F)
where
  F: Fn((Vec3, Vec3)),
{
  for i in 0..face.vertices.len() {
    let a = Vec3::new(
      face.vertices[i][0],
      face.vertices[i][1],
      face.vertices[i][2],
    );
    let b = Vec3::new(
      face.vertices[(i + 1) % face.vertices.len()][0],
      face.vertices[(i + 1) % face.vertices.len()][1],
      face.vertices[(i + 1) % face.vertices.len()][2],
    );
    func((a, b));
  }
}

fn generate_cube_face() {}

fn generate_cube_mesh(position: &Vec3) -> Mesh {
  let vertices = vec![
    // Front
    [position.x - 0.5, position.y - 0.5, position.z + 0.5],
    [position.x + 0.5, position.y - 0.5, position.z + 0.5],
    [position.x + 0.5, position.y + 0.5, position.z + 0.5],
    [position.x - 0.5, position.y + 0.5, position.z + 0.5],
    // Back
    [position.x - 0.5, position.y - 0.5, position.z - 0.5],
    [position.x + 0.5, position.y - 0.5, position.z - 0.5],
    [position.x + 0.5, position.y + 0.5, position.z - 0.5],
    [position.x - 0.5, position.y + 0.5, position.z - 0.5],
    // Left
    [position.x - 0.5, position.y - 0.5, position.z - 0.5],
    [position.x - 0.5, position.y - 0.5, position.z + 0.5],
    [position.x - 0.5, position.y + 0.5, position.z + 0.5],
    [position.x - 0.5, position.y + 0.5, position.z - 0.5],
    // Right
    [position.x + 0.5, position.y - 0.5, position.z - 0.5],
    [position.x + 0.5, position.y - 0.5, position.z + 0.5],
    [position.x + 0.5, position.y + 0.5, position.z + 0.5],
    [position.x + 0.5, position.y + 0.5, position.z - 0.5],
    // Top
    [position.x - 0.5, position.y + 0.5, position.z + 0.5],
    [position.x + 0.5, position.y + 0.5, position.z + 0.5],
    [position.x + 0.5, position.y + 0.5, position.z - 0.5],
    [position.x - 0.5, position.y + 0.5, position.z - 0.5],
    // Bottom
    [position.x - 0.5, position.y - 0.5, position.z - 0.5],
    [position.x + 0.5, position.y - 0.5, position.z - 0.5],
    [position.x + 0.5, position.y - 0.5, position.z + 0.5],
    [position.x - 0.5, position.y - 0.5, position.z + 0.5],
  ];

  let normals = vec![
    // Front
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, 1.0],
    // Back
    [0.0, 0.0, -1.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, -1.0],
    [0.0, 0.0, -1.0],
    // Left
    [-1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    // Right
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    // Top
    [0.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    // Bottom
    [0.0, -1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, -1.0, 0.0],
    [0.0, -1.0, 0.0],
  ];

  let indices = vec![
    // Front
    0, 1, 2, 2, 3, 0, // Back
    4, 5, 6, 6, 7, 4, // Left
    8, 9, 10, 10, 11, 8, // Right
    12, 13, 14, 14, 15, 12, // Top
    16, 17, 18, 18, 19, 16, // Bottom
    20, 21, 22, 22, 23, 20,
  ];

  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

  return mesh;
}

pub fn spawn_voxels(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  query: Query<&Voxel>,
) {
  let mesh = generate_cube_mesh(&Vec3::new(0.0, 2.0, 0.0));

  commands.spawn(PbrBundle {
    mesh: meshes.add(mesh),
    material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    ..default()
  });
}
