use bevy::prelude::*;
use rand::prelude::*;
use bevy::render::mesh::{self, PrimitiveTopology};

// NOTE: the algorithm can be sped up EXTREMELY by using stack allocated arrays instead of vectors
//       but this requires increasing the size of the stack on the main thread.

pub struct Chunk {
  pub size: usize,
  pub voxels: Vec<Vec<Vec<u8>>>,
}

impl Chunk {
  pub fn new(size: usize) -> Self {
    // let mut voxels = [[[1; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    let mut voxels = vec![vec![vec![1; size]; size]; size];

    for x in 0..size {
      for y in 0..size {
        for z in 0..size {
          voxels[x][y][z] = random::<u8>() % 2;
        }
      }
    }

    Self { size, voxels }
  }

  pub fn print(&self) {
    for z in 0..self.size {
      print!("y|x ");
      for y in 0..self.size {
        print!("{} ", y);
      }
      println!("");

      print!(" +-");
      for _ in 0..self.size {
        print!("--");
      }
      println!("   z = {}", z);

      for y in 0..self.size {
        print!("{}|  ", y);
        for x in 0..self.size {
          print!("{} ", self.voxels[x][y][z]);
        }
        println!();
      }
      println!();
    }
  }
}

// type FaceQueue = [[[[u8; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]; 6];
type FaceQueue = Vec<Vec<Vec<Vec<u8>>>>;

/**
 * Returns wether a face is NOT hidden by another voxel
 * `face`: the face to check
 * `direction`: the direction of the face (0 = right, 1 = left, 2 = up, 3 = down, 4 = front, 5 = back)
 * `chunk`: the chunk to check
 */
fn face_visible(face: &(usize, usize, usize), direction: usize, chunk: &Chunk) -> bool {
  if direction == 0 {
    // right
    if face.0 == chunk.size - 1 {
      return true;
    }
    chunk.voxels[face.0 + 1][face.1][face.2] == 0
  } else if direction == 1 {
    // left
    if face.0 == 0 {
      return true;
    }
    chunk.voxels[face.0 - 1][face.1][face.2] == 0
  } else if direction == 2 {
    // up
    if face.1 == chunk.size - 1 {
      return true;
    }
    chunk.voxels[face.0][face.1 + 1][face.2] == 0
  } else if direction == 3 {
    // down
    if face.1 == 0 {
      return true;
    }
    chunk.voxels[face.0][face.1 - 1][face.2] == 0
  } else if direction == 4 {
    // front
    if face.2 == chunk.size - 1 {
      return true;
    }
    chunk.voxels[face.0][face.1][face.2 + 1] == 0
  } else if direction == 5 {
    // back
    if face.2 == 0 {
      return true;
    }
    chunk.voxels[face.0][face.1][face.2 - 1] == 0
  } else {
    panic!("invalid direction");
  }
}

/**
 * Returns wether the face exists in the set
 * `face`: the face to check
 * `direction`: the direction of the face (0 = right, 1 = left, 2 = up, 3 = down, 4 = front, 5 = back)
 * `face_set`: the set of faces to check
 */
fn face_exists(
  face: &(usize, usize, usize),
  direction: usize,
  face_set: &FaceQueue,
  chunk: &Chunk,
) -> bool {
  // println!("face set: {:?}", face_set[direction]);
  if face.0 >= chunk.size || face.1 >= chunk.size || face.2 >= chunk.size {
    return false;
  }

  let exists = face_set[direction][face.0][face.1][face.2] == 1;
  // println!("-- face exists? {:?} {:?}", face, exists);

  exists
}

/**
 * Checks if a row of faces can be extended vertically
 * `height`: the height of the row
 * `width`: the width of the row
 * `direction`: the direction the row is facing (0 = right, 1 = left, 2 = up, 3 = down, 4 = front, 5 = back)
 * `face`: the face that the row is based on
 * `face_set`: the set of faces to check
 * `chunk`: the chunk to check
 */
fn can_extend_row(
  height: usize,
  width: usize,
  direction: usize,
  face: &(usize, usize, usize),
  face_set: &FaceQueue,
  chunk: &Chunk,
) -> bool {
  let expand = expand_direction(direction);

  // println!("- can extend row? {:?}", face);

  (0..width).all(|w| {
    let next_face = (
      face.0 + expand.width[0] * w + expand.height[0] * height,
      face.1 + expand.width[1] * w + expand.height[1] * height,
      face.2 + expand.width[2] * w + expand.height[2] * height,
    );

    // println!("-- looking at face {:?}, w: {}, h: {}", next_face, w, height);

    face_exists(&next_face, direction, face_set, chunk)
      && face_visible(&next_face, direction, chunk)
  })
}

/**
 * Utils struct that keeps track of the next face to check
 * `next_face`: the next face to check
 */
struct FaceFinder {
  next_face: (usize, usize, usize),
}

impl FaceFinder {
  fn new() -> Self {
    Self {
      next_face: (0, 0, 0),
    }
  }

  /**
   * Finds the next face in the chunk to check
   * `face_set`: the set of faces to check
   * `chunk`: the chunk to check
   * `direction`: the direction of the face (0 = right, 1 = left, 2 = up, 3 = down, 4 = front, 5 = back)
   */
  fn next_face(
    &mut self,
    face_set: &FaceQueue,
    chunk: &Chunk,
    direction: usize,
  ) -> Option<(usize, usize, usize)> {
    // println!("-- looking for next face starting with {:?}", to_test);
    while self.next_face.0 < chunk.size {
      while self.next_face.1 < chunk.size {
        while self.next_face.2 < chunk.size {
          // println!("-- looking at face {:?}", self.next_face);
          // if face_set[direction].contains(&(self.next_face.0, self.next_face.1, self.next_face.2)) {
          if face_set[direction][self.next_face.0][self.next_face.1][self.next_face.2] == 1 {
            // println!("found next face! {:?}", self.next_face);
            return Some(self.next_face);
          }
          self.next_face.2 += 1;
        }
        self.next_face.2 = 0;
        self.next_face.1 += 1;
      }
      self.next_face.1 = 0;
      self.next_face.0 += 1;
    }

    return None;

    // panic!("No next face found! (to_test = {:?})", &self.next_face); // should not happen in the algorithm
  }
}

struct ExpandDirection {
  width: [usize; 3],
  height: [usize; 3],
}

/**
 * Returns the expansion direction for a given direction
 * `direction`: the direction to expand (0 = right, 1 = left, 2 = up, 3 = down, 4 = front, 5 = back)
 * returns an `ExpandDirection` struct, containing the expansion directions for the width and height
 * an `ExpandDirection` struct contains 3 values, one for each axis (x, y, z), a face should be expanded in the axis with a value of 1, and not expanded in the axis with a value of 0
 */
fn expand_direction(direction: usize) -> ExpandDirection {
  match direction {
    // left - right
    0 => ExpandDirection {
      width: [0, 0, 1],
      height: [0, 1, 0],
    },
    1 => ExpandDirection {
      width: [0, 0, 1],
      height: [0, 1, 0],
    },
    // up - down
    2 => ExpandDirection {
      width: [1, 0, 0],
      height: [0, 0, 1],
    },
    3 => ExpandDirection {
      width: [1, 0, 0],
      height: [0, 0, 1],
    },
    // front - back
    4 => ExpandDirection {
      width: [1, 0, 0],
      height: [0, 1, 0],
    },
    5 => ExpandDirection {
      width: [1, 0, 0],
      height: [0, 1, 0],
    },

    _ => panic!("Invalid direction!"),
  }
}

/**
 * Runs the greedy meshing algorithm on a chunk
 * `chunk`: the chunk to run the algorithm on
 */
pub fn greedy_mesh(chunk: &Chunk) -> Mesh {
  // the queue of faces that still need to be meshed
  let mut face_queue: FaceQueue = vec![vec![vec![vec![0; chunk.size]; chunk.size]; chunk.size]; 6];

  // for each voxel in the chunk...
  for x in 0..chunk.size {
    for y in 0..chunk.size {
      for z in 0..chunk.size {
        // ...if the voxel is not empty...
        if chunk.voxels[x][y][z] == 0 {
          continue;
        }

        // ...for each direction...
        for d in 0..6 {
          let face = (x, y, z);

          // ...if the face is visible we add it to the queue
          if face_visible(&face, d, chunk) {
            // println!("face {:?} visible", face);
            // face_set[d].insert((x, y, z));
            face_queue[d][x][y][z] = 1;
          }
        }
      }
    }
  }
  
  let mut vertices: Vec<[f32; 3]> = Vec::new();
  let mut indices: Vec<u32> = Vec::new();
  let mut normals: Vec<[f32; 3]> = Vec::new();

  // for each direction...
  for d in [0, 1, 2, 3, 4, 5].iter() {
    let d = *d;
    let expand = expand_direction(d);
    let mut face_finder = FaceFinder::new();

    // println!("{:?} \n", face_set[d]);

    // ...we loop until there are no more faces to check
    loop {
      // find the next face to test
      let face = face_finder.next_face(&face_queue, chunk, d);

      // if there are no more faces to test, we break the loop
      if face.is_none() {
        break;
      }
      let face = face.unwrap();

      // println!("starting face at {:?}", face);

      // remove the face from the queue
      face_queue[d][face.0][face.1][face.2] = 0;

      // check if the face is visible, if not, we skip it (note: the face is still removed from the queue)
      if !face_visible(&face, d, chunk) {
        // println!("face {:?} not visible", face);
        continue;
      }

      // println!("testing face {:?}", face);

      // while the adjacent face is visible, we expand the face in the width direction
      let mut width = 1;
      let mut next_face = (
        face.0 + expand.width[0] * width,
        face.1 + expand.width[1] * width,
        face.2 + expand.width[2] * width,
      );

      // println!("next face: {:?}", next_face);

      while face_exists(&next_face, d, &face_queue, chunk) && face_visible(&next_face, d, chunk) {
        // face_set[d].remove(&(next_face.0, next_face.1, next_face.2));
        face_queue[d][next_face.0][next_face.1][next_face.2] = 0;
        // println!("extended to face {:?}", next_face);
        width += 1;
        next_face = (
          face.0 + expand.width[0] * width,
          face.1 + expand.width[1] * width,
          face.2 + expand.width[2] * width,
        );
      }

      // println!("width: {}", width);

      // if possible, we expand the whole row of faces in the height direction
      let mut height = 1;
      while can_extend_row(height, width, d, &face, &face_queue, chunk) {
        // println!("can extend row {}!", height);
        // remove the faces from the queue
        for w in 0..width {
          face_queue[d][face.0 + expand.width[0] * w + expand.height[0] * height]
            [face.1 + expand.width[1] * w + expand.height[1] * height]
            [face.2 + expand.width[2] * w + expand.height[2] * height] = 0;
        }

        height += 1;
      }

      // TODO: generate mesh for the face OR add the face to the list of faces to generate meshes for
      // println!("Face: {:?}, Width: {}, Height: {}, direction {} \n", face, width, height, d);

      let extra_x = if d == 0 { 1.0 } else { 0.0 };
      let extra_y = if d == 2 { 1.0 } else { 0.0 };
      let extra_z = if d == 4 { 1.0 } else { 0.0 };

      vertices.push([
        face.0 as f32 + extra_x,
        face.1 as f32 + extra_y,
        face.2 as f32 + extra_z
      ]);
      vertices.push([
        face.0 as f32 + expand.width[0] as f32 * width as f32 + extra_x,
        face.1 as f32 + expand.width[1] as f32 * width as f32 + extra_y,
        face.2 as f32 + expand.width[2] as f32 * width as f32 + extra_z,
      ]);
      vertices.push([
        face.0 as f32 + expand.width[0] as f32 * width as f32 + expand.height[0] as f32 * height as f32 + extra_x,
        face.1 as f32 + expand.width[1] as f32 * width as f32 + expand.height[1] as f32 * height as f32 + extra_y,
        face.2 as f32 + expand.width[2] as f32 * width as f32 + expand.height[2] as f32 * height as f32 + extra_z,
      ]);
      vertices.push([
        face.0 as f32 + expand.height[0] as f32 * height as f32 + extra_x,
        face.1 as f32 + expand.height[1] as f32 * height as f32 + extra_y,
        face.2 as f32 + expand.height[2] as f32 * height as f32 + extra_z,
      ]);

      let mut idx = vec![0, 1, 2, 2, 3, 0];

      if d == 0 || d == 2 || d == 5 {
        idx.reverse();
      }

      for i in 0..idx.len() { idx[i] += (vertices.len() - 4) as u32; }
      indices.append(&mut idx);

      for _ in 0..4 {
        normals.push( match d {
          0 => [1.0, 0.0, 0.0],
          1 => [-1.0, 0.0, 0.0],
          2 => [0.0, 1.0, 0.0],
          3 => [0.0, -1.0, 0.0],
          4 => [0.0, 0.0, 1.0],
          5 => [0.0, 0.0, -1.0],
          _ => panic!("Invalid direction!"),
        });
      }
    }
  }

  let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

  // println!("Vertices: {:?} {:?} ", vertices.len(), vertices);
  // println!("Indices: {:?} {:?}", indices.len(), indices);
  // println!("Normals: {:?} {:?}", normals.len(), normals);

  mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
  mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
  mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
  

  return mesh;
}
