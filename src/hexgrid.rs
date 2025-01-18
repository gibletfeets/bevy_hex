use bevy::asset::RenderAssetUsages;
use bevy::color::Color;
use bevy::math::{UVec3, Vec2, Vec3};
use bevy::prelude::{Resource, Mesh};
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::ShaderType;
use rand::random;

//TODO: REPLACE WITH ENUM, SET UP THINGS TO INDEX WITH THIS
//NE: 0
// W: 1
//SE: 2
//SW: 3
// E: 4
//NW: 5
static NE: usize = 0;
static  W: usize = 1;
static SE: usize = 2;
static SW: usize = 3;
static  E: usize = 4;
static NW: usize = 5;

static COLOR1: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
static COLOR2: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
static COLOR3: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

static OUTER_RADIUS: f32 = 10.0;
static INNER_RADIUS: f32 = OUTER_RADIUS * 0.866025404;

static SOLID_FACTOR: f32 = 0.8;
static BLEND_FACTOR: f32 = 1.0 - SOLID_FACTOR;
static HEX_CORNERS: [Vec3; 7] = [
    Vec3::new(0.0, 0.0, -OUTER_RADIUS),
    Vec3::new(-INNER_RADIUS, 0.0, -0.5 * OUTER_RADIUS),
    Vec3::new(-INNER_RADIUS, 0.0,  0.5 * OUTER_RADIUS),
    Vec3::new(0.0, 0.0, OUTER_RADIUS),
    Vec3::new( INNER_RADIUS, 0.0,  0.5 * OUTER_RADIUS),
    Vec3::new( INNER_RADIUS, 0.0, -0.5 * OUTER_RADIUS),
    Vec3::new(0.0, 0.0, -OUTER_RADIUS)
];

static HEX_NORMALS: [Vec2; 7] = [
    Vec2::new( 0.5, -0.866025404),
    Vec2::new(-0.5, -0.866025404),
    Vec2::new(-1.0, 0.0),
    Vec2::new(-0.5,  0.866025404),
    Vec2::new( 0.5,  0.866025404),
    Vec2::new( 1.0,0.0),
    Vec2::new( 0.5, -0.866025404)
];

//TODO: IMPLEMENT HEXAGONAL COORDINATE STYLES - AXIAL AND OFFSET. IMPLEMENT INDEXING WITH THIS

#[derive(Clone, Copy, Debug)]
pub struct OffsetCoordinate {
    pub x: usize,
    pub z: usize
}

impl OffsetCoordinate {
    pub fn from_hex(hex: HexCoordinate) -> OffsetCoordinate {
        let x = hex.x + (hex.z - (hex.z%2)) / 2;
        let z = hex.z;
        OffsetCoordinate{x: x as usize, z: z as usize}
    }

    pub fn from_position(position: Vec3) -> OffsetCoordinate {
        Self::from_hex(HexCoordinate::from_position(position))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HexCoordinate {
    x: i32,
    y: i32,
    z: i32
}

impl HexCoordinate {
    fn new(x: i32, z: i32) -> HexCoordinate {
        HexCoordinate{
            x,
            y: -x - z,
            z
        }
    }
    fn from_position(position: Vec3) -> HexCoordinate {
        let mut x = position.x / (INNER_RADIUS * 2.0);
        let mut y = -x;
        let offset = -position.z / (OUTER_RADIUS * 3.0);
        x -= offset;
        y -= offset;
        let mut ix = x.round() as i32;
        let iy = y.round() as i32;
        let mut iz = (-x - y).round() as i32;
        if ix + iy + iz != 0 {
            let dx = (x - ix as f32).abs();
            let dy = (y - iy as f32).abs();
            let dz = (-x - y - iz as f32).abs();

            if dx > dy && dx > dz {
                ix = -iy - iz;
            } else if dz > dy {
                iz = -ix - iy;
            }
        }

        HexCoordinate::new(ix, iz)
    }
}

struct EdgeVertices {
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    v4: Vec3,
}

impl EdgeVertices {
    fn new(v1: Vec3, v4: Vec3) -> EdgeVertices {
        let v2 = v1.lerp(v4, 1.0/3.0);
        let v3 = v1.lerp(v4, 2.0/3.0);
        EdgeVertices{ v1, v2, v3, v4 }
    }
}

pub struct HexCell {
    neighbor_cell_refs: [Option<(usize, usize)>; 6],
    pub height_refs: [(usize, usize); 6],
    terrain: u32,
    position: Vec3,
}

struct HexMeshData{
    vertices: Vec<Vec3>,
    colors: Vec<[f32; 4]>,
    vert_terrain: Vec<UVec3>,
    triangles: Vec<u32>
}

#[derive(Resource)]
pub struct HexGrid {
    pub cells: Vec<Vec<HexCell>>,
    pub heights: Vec<Vec<i32>>,
}
impl HexGrid {
    pub(crate) fn new(cell_count_x: usize, cell_count_z: usize) -> HexGrid {
        let mut heights = vec![vec![2; cell_count_z+1]; 2*cell_count_x];
        // for z in 0..(cell_count_z+1) {
        //     for x in 0..(2*cell_count_x) {
        //         heights[x][z] = (x+z) as i8;
        //     }
        // }


        let mut cells = (0..cell_count_x)
            .map(|x| (0..cell_count_z)
                .map(|z| {
                HexGrid::create_cell(x, z, cell_count_x, cell_count_z)
        }).collect::<Vec<HexCell>>()).collect::<Vec<Vec<HexCell>>>();
        // for x in 0..cell_count_x {
        //     for z in 0..cell_count_z {
        //         if random::<f32>() > 0.5 {
        //             for (ref_x, ref_z) in cells[x][z].height_refs {
        //                 heights[ref_x][ref_z] += 1;
        //             }
        //         }
        //     }
        // }
        // for (ref_x, ref_z) in cells[1][1].height_refs {
        //     heights[ref_x][ref_z] = 1;
        // }
        for x in 0..cell_count_x {
            for z in 0..cell_count_z {
                if x > 0 {
                    cells[x][z].neighbor_cell_refs[W] = Some((x-1, z));
                    cells[x-1][z].neighbor_cell_refs[E] = Some((x, z));
                }
                if z > 0 {
                    if z%2 == 0 {
                        cells[x][z  ].neighbor_cell_refs[SW] = Some((x, z-1));
                        cells[x][z-1].neighbor_cell_refs[NE] = Some((x, z));
                        if x > 0 {
                            cells[x  ][z  ].neighbor_cell_refs[SE] = Some((x-1, z-1));
                            cells[x-1][z-1].neighbor_cell_refs[NW] = Some((x  , z  ));
                        }
                    } else {
                        cells[x][z  ].neighbor_cell_refs[SE] = Some((x, z-1));
                        cells[x][z-1].neighbor_cell_refs[NW] = Some((x, z  ));
                        if x < cell_count_x-1 {
                            cells[x  ][z  ].neighbor_cell_refs[SW] = Some((x+1, z-1));
                            cells[x+1][z-1].neighbor_cell_refs[NE] = Some((x  , z  ));
                        }
                    }
                }
            }
        }
        HexGrid {
            cells,
            heights
        }
    }

    fn get_height_refs(_x: usize, z: usize, cell_count_x: usize, _cell_count_z: usize) -> [(usize, usize); 6] {
        let x = 2*(_x%cell_count_x) + z%2;
        [
            ((x+1)%(cell_count_x*2), z+1),
            (x, z + 1),
            (x, z),
            ((x+1)%(cell_count_x*2), z),
            ((x+2)%(cell_count_x*2), z),
            ((x+2)%(cell_count_x*2), z+1),
        ]
    }

    fn create_cell(_x: usize, _z: usize, cell_count_x: usize, cell_count_z: usize) -> HexCell {
        let (x, z) = (_x as f32, _z as f32);
        let position = Vec3::new(
            (x+z*0.5 - (_z/2) as f32)*INNER_RADIUS*2.0,
            0.0,
            -z*OUTER_RADIUS*1.5
        );
        let height_refs = HexGrid::get_height_refs(_x, _z, cell_count_x, cell_count_z);
        let neighbor_cell_refs = [None; 6];
        let terrain = 0;
        HexCell{
            position,
            height_refs,
            terrain,
            neighbor_cell_refs
        }
    }


    //http://www.geometry.caltech.edu/pubs/WSHD05.pdf
    
    fn curve(x: f32) -> f32{
        //let offset = x.floor();
        //let x = x.fract();
        //x*x*x*(x*(x*6.0-15.0)+10.0)+offset
        x
    }
    fn calc_weight(v: Vec2, n: Vec2, m: Vec2, x: Vec2) -> f32 {
        let x = x;
        static K: f32 = 0.86602540378;
        K/(n.dot(v-x)*m.dot(v-x))
    }

    fn calc_height(&self, _x: Vec3, cell: &HexCell) -> f32 {
        let x = Vec2::new(_x.x - cell.position.x, _x.z - cell.position.z);
        let mut sum = 0.0;
        let mut height = 0.0;
        for i in 0..6 {
            let p = HEX_CORNERS[i];
            let v = Vec2::new(p.x, p.z);
            let n = HEX_NORMALS[i];
            let m = HEX_NORMALS[i+1];
            let weight = Self::calc_weight(v, n, m, x);

            sum += weight;

            let (height_x, height_z) = cell.height_refs[i];
            //println!("x: {height_x}, z: {height_z}");
            //println!("{weight}");
            height += self.heights[height_x][height_z] as f32 * weight;
        }
        Self::curve((height/sum))*(OUTER_RADIUS/4.0)
    }

    pub fn triangulate_grid(&self) -> Mesh {
        let mut data = HexMeshData {
            vertices: vec![],
            colors: vec![],
            vert_terrain: vec![],
            triangles: vec![]
        };
        for cell in self.cells.iter().flatten() {
            self.triangulate_cell(
                cell,
                &mut data
            );
        }
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                data.vertices.clone()
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_UV_0,
                data.vertices.iter().map(|v| Vec2::new(v.x/INNER_RADIUS, v.z/INNER_RADIUS)).collect::<Vec<Vec2>>()
            )
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_COLOR,
                data.colors
            )
            // .with_inserted_attribute(
            //     ATTRIBUTE_TEXTURE_INDEX,
            //     data.vert_terrain
            // )
            .with_inserted_indices(
                //TODO: fix backwards winding order
                Indices::U32(data.triangles.into_iter().rev().collect::<Vec<u32>>())
            )
            .with_computed_smooth_normals()
    }

    fn triangulate_cell(
        &self,
        cell: &HexCell,
        data: &mut HexMeshData
    ) {
        for dir in NE..=NW {
            //this whole process is creating redundant verts and should probably be changed.
            let vert_idx_pre_tri = data.vertices.len();
            self.subdivide_triangle(
                cell.position,
                cell.position + HEX_CORNERS[dir]*SOLID_FACTOR,
                 cell.position + HEX_CORNERS[dir+1]*SOLID_FACTOR,
                cell.terrain,
                data
            );
            for vertex in &mut data.vertices[vert_idx_pre_tri..] {
                vertex.y = self.calc_height(vertex.clone(), cell);
            }
            if dir <= SE {
                let e = EdgeVertices::new(
                    cell.position + HEX_CORNERS[dir]*SOLID_FACTOR,
                    cell.position + HEX_CORNERS[dir+1]*SOLID_FACTOR,
                );
                self.triangulate_connection(
                    dir,
                    cell,
                    e,
                    data
                );
            }
        }
    }

    fn subdivide_triangle(
        &self,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        terrain: u32,
        data: &mut HexMeshData
    ) {
        let va1 = v1.lerp(v2, 1.0/3.0);
        let va2 = v1.lerp(v2, 2.0/3.0);

        let vb1 = v2.lerp(v3, 1.0/3.0);
        let vb2 = v2.lerp(v3, 2.0/3.0);

        let vc1 = v3.lerp(v1, 1.0/3.0);
        let vc2 = v3.lerp(v1, 2.0/3.0);

        let c = (v1 + v2 + v3)/3.0;

        let vert_idx = data.vertices.len() as u32;

        data.vertices.push(c  ); //+0
        data.vertices.push(v1 ); //+1
        data.vertices.push(v2 ); //+2
        data.vertices.push(v3 ); //+3
        data.vertices.push(va1); //+4
        data.vertices.push(va2); //+5
        data.vertices.push(vb1); //+6
        data.vertices.push(vb2); //+7
        data.vertices.push(vc1); //+8
        data.vertices.push(vc2); //+9

        data.colors.append(&mut vec![COLOR1; 10]);
        data.vert_terrain.append(&mut vec![UVec3::new(terrain, terrain, terrain); 10]);

        data.triangles.append(&mut vec![vert_idx+1, vert_idx+9, vert_idx+4]);

        data.triangles.append(&mut vec![vert_idx+5, vert_idx+4, vert_idx ]);

        data.triangles.append(&mut vec![vert_idx+5, vert_idx+6, vert_idx+2]);

        data.triangles.append(&mut vec![vert_idx  , vert_idx+7, vert_idx+6]);

        data.triangles.append(&mut vec![vert_idx+8, vert_idx+3, vert_idx+7]);

        data.triangles.append(&mut vec![vert_idx+9, vert_idx+8, vert_idx  ]);

        data.triangles.append(&mut vec![vert_idx  , vert_idx+4, vert_idx+9]);

        data.triangles.append(&mut vec![vert_idx+5, vert_idx  , vert_idx+6]);

        data.triangles.append(&mut vec![vert_idx+8, vert_idx+7, vert_idx  ]);
    }

    fn triangulate_connection(
        &self,
        dir: usize,
        cell: &HexCell,
        e1: EdgeVertices,
        data: &mut HexMeshData
    ) {
        match cell.neighbor_cell_refs[dir] {
            Some((x, z)) => {
                let neighbor = &self.cells[x][z];
                //let bridge = (HEX_CORNERS[dir] + HEX_CORNERS[dir+1])*BLEND_FACTOR;
                let mut bridge = (HEX_CORNERS[dir] + HEX_CORNERS[dir+1])*BLEND_FACTOR;
                bridge.y = neighbor.position.y - cell.position.y;
                let e2 = EdgeVertices::new(
                    e1.v1 + bridge,
                    e1.v4 + bridge
                );
                self.triangulate_edge_strip(
                    &e1,
                    cell.terrain,
                    &e2,
                    neighbor.terrain,
                    data
                );
                let vert_idx = data.vertices.len();
                for (idx, vertex) in &mut data.vertices[(vert_idx-12)..].iter_mut().enumerate() {
                    if idx%4 < 2 {
                        vertex.y = self.calc_height(vertex.clone(), cell);
                    } else {
                        vertex.y = self.calc_height(vertex.clone(), neighbor);
                    }
                }
                //TODO - this triangle is getting made more times than it needs to be. Investigate.
                if dir <= E {
                    if let Some((x, z)) = cell.neighbor_cell_refs[(dir+1)%6] {
                        let next_dir = (dir+1)%6;
                        let next_neighbor = &self.cells[x][z];
                        let bridge = (HEX_CORNERS[next_dir] + HEX_CORNERS[next_dir+1])*BLEND_FACTOR;
                        let vert_idx = data.vertices.len();
                        data.vertices.append(&mut vec![
                            e1.v4, e2.v4, e1.v4.clone() + bridge
                        ]);
                        data.vertices[vert_idx].y = self.calc_height(data.vertices[vert_idx], cell);
                        data.vertices[vert_idx+1].y = self.calc_height(data.vertices[vert_idx+1], neighbor);
                        data.vertices[vert_idx+2].y = self.calc_height(data.vertices[vert_idx+2], next_neighbor);

                        let vert_idx = vert_idx as u32;
                        data.triangles.append(&mut vec![vert_idx, vert_idx + 2, vert_idx + 1]);
                        data.colors.append(&mut vec![
                            COLOR2,
                            COLOR1,
                            COLOR3
                        ]);
                        let types = UVec3::new(
                            neighbor.terrain,
                            cell.terrain,
                            next_neighbor.terrain
                        );
                        data.vert_terrain.append(&mut vec![types; 3]);

                    }
                }
            },
            None => {return}
        }
    }

    fn triangulate_edge_strip(
        &self,
        e1: &EdgeVertices,
        terrain1: u32,
        e2: &EdgeVertices,
        terrain2: u32,
        data: &mut HexMeshData
    ) {
        //TODO: remove redundant verts
        let terrain = UVec3::new(terrain2, terrain1, terrain2);
        Self::add_quad(e1.v1, e1.v2, e2.v1, e2.v2, data);
        data.colors.append(&mut vec![COLOR2, COLOR2, COLOR1, COLOR1]);
        Self::add_quad(e1.v2, e1.v3, e2.v2, e2.v3, data);
        data.colors.append(&mut vec![COLOR2, COLOR2, COLOR1, COLOR1]);
        Self::add_quad(e1.v3, e1.v4, e2.v3, e2.v4, data);
        data.colors.append(&mut vec![COLOR2, COLOR2, COLOR1, COLOR1]);
        data.vert_terrain.append(&mut vec![terrain; 12]);

    }

    fn add_quad(
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        v4: Vec3,
        data: &mut HexMeshData
    ) {
        let vert_idx = data.vertices.len() as u32;
        data.vertices.append(&mut vec![v1, v2, v3, v4]);
        data.triangles.append(&mut vec![
            vert_idx,
            vert_idx + 1,
            vert_idx + 2,
            vert_idx + 1,
            vert_idx + 3,
            vert_idx + 2
        ])
    }
}