extern crate cgmath;
extern crate mint;
extern crate three;
extern crate mmd;

use std::env;
use std::path::Path;

use cgmath::prelude::*;
use three::Object;

use mmd::io::pmx::PmxFile;

struct PmxModel {
    f: PmxFile,
    meshes: Vec<three::Mesh>,
    geometries: Vec<three::Geometry>,
    materials: Vec<three::Material>,
}

impl PmxModel {
    fn new<P: AsRef<Path>>(win: &mut three::Window, file_path: P) -> PmxModel { 
        let f = PmxFile::from_file(&file_path).unwrap();

        let vertices = f.model.vertices.0.iter().map(|x| {
            let v3 = x.position.0;
            mint::Point3 { x: v3.x, y: v3.y, z: v3.z }
        }).collect::<Vec<_>>();

        let tex_coords = f.model.vertices.0.iter().map(|x| {
            let v2 = x.uv.0;
            mint::Point2 { x: v2.x, y: 1.0 - v2.y }
        }).collect::<Vec<_>>();

        let faces = f.model.face_indices.0.iter().map(|x| {
            x.0 as u32
        }).collect::<Vec<u32>>().chunks(3).map(|x| [x[0], x[1], x[2]]).collect::<Vec<_>>();

        let num_meshes = f.model.materials.0.len();
        let mut materials = Vec::with_capacity(num_meshes);
        let mut geometries = Vec::with_capacity(num_meshes);
        let mut meshes = Vec::with_capacity(num_meshes);
        let mut face_start: usize = 0;
        for i in 0..num_meshes {
            let texture_id = f.model.materials.0[i].texture_id.0;
            let num_faces = f.model.materials.0[i].num_vertex_indices;
            assert!(num_faces>= 0);
            println!("{:?}: {} faces", f.model.materials.0[i].name.jp, num_faces);
            let face_end = face_start + (num_faces as usize / 3);

            //assert!(texture_id != -1);
            let texture = if texture_id == -1 {
                None
            } else {
                let texture_name = &(f.model.textures.0[texture_id as usize].0).0;
                Some(win.factory.load_texture(file_path.as_ref().with_file_name(texture_name)))
            };

            let material = three::material::Material::Basic(
                three::material::Basic {
                    map: texture,
                    .. three::material::basic::Basic::default()
                }
            );

            let geometry = three::Geometry {
                faces: faces[face_start..face_end].to_vec(),
                tex_coords: tex_coords.clone(),
                base: three::Shape {
                    vertices: vertices.clone(),
                    .. three::Shape::default()
                },
                .. three::Geometry::default()
            };
 
            materials.push(material.clone());
            geometries.push(geometry.clone());

            let mesh = win.factory.mesh(geometry, material);
            win.scene.add(&mesh);
            meshes.push(mesh);

            face_start = face_end;
        }

        PmxModel { f, meshes, geometries, materials }
    }
}

fn main() {

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./three-viewer yourfile.pmx");
        return;
    }
    let file_path = &args[1];

    let mut win = three::Window::new("Three-rs MMD example");
    let cam = win.factory.perspective_camera(75.0, 1.0 .. 50.0);
    cam.set_position([0.0, 0.0, 20.0]);

    let mmd_geometry = {
        let f = PmxFile::from_file(file_path).unwrap();
        let vertices = f.model.vertices.0.iter().map(|x| {
            let v3 = x.position.0;
            mint::Point3 { x: v3.x, y: v3.y, z: v3.z }
        }).collect::<Vec<_>>();
        let faces = f.model.face_indices.0.iter().map(|x| {
            x.0 as u32
        }).collect::<Vec<u32>>().chunks(3).map(|x| [x[0], x[1], x[2]]).collect::<Vec<_>>();
        three::Geometry {
            faces,
            base: three::Shape {
                vertices,
                .. three::Shape::default()
            },
            .. three::Geometry::default()
        }
    };
    let mmd_material = three::material::Wireframe { color: 0xFFFFFF };
    let mmd_mesh = win.factory.mesh(mmd_geometry, mmd_material);
    win.scene.add(&mmd_mesh);

    let mmd_with_texture = PmxModel::new(&mut win, file_path);

    mmd_mesh.set_position([-10.0, -10.0, 0.0]);

    for mesh in mmd_with_texture.meshes.iter() {
        mesh.set_position([10.0, -10.0, 0.0])
    }

    let mut angle = cgmath::Rad::zero();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        if let Some(diff) = win.input.timed(three::AXIS_LEFT_RIGHT) {
            angle += cgmath::Rad(1.5 * diff);
            let q = cgmath::Quaternion::from_angle_y(angle);
            mmd_mesh.set_orientation(q);
            for mesh in mmd_with_texture.meshes.iter() {
                mesh.set_orientation(q);
            }
        }
        win.render(&cam);
    }
}
