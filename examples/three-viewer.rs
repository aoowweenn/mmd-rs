extern crate cgmath;
extern crate mint;
extern crate three;
extern crate mmd;

use cgmath::prelude::*;
use three::Object;

use mmd::io::pmx::PmxFile;

fn main() {
    let mut win = three::Window::new("Three-rs shapes example");
    let cam = win.factory.perspective_camera(75.0, 1.0 .. 50.0);
    cam.set_position([0.0, 0.0, 20.0]);

    let mmd_geometry = {
        let f = PmxFile::from_file("asset/江風ver1.05.pmx").unwrap();
        println!("{:?}", f.model_name);
        let vertices = f.model.vertices.0.iter().map(|x| {
            let v3 = x.position.0;
            mint::Point3 { x: v3.x, y: v3.y, z: v3.z }
        }).collect::<Vec<_>>();
        let faces = f.model.face_indices.0.iter().map(|x| {
            if x.0 < 0 || x.0 >= vertices.len() as i32 {
                println!("wrong? {} -> {}", x.0, x.0 as u32);
            }
            x.0 as u32
        }).collect::<Vec<u32>>().chunks(3).map(|x| [x[0], x[1], x[2]]).collect::<Vec<_>>();
        println!("{}, {}", vertices.len(), faces.len());
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

    mmd_mesh.set_position([0.0, -10.0, 0.0]);

    let mut angle = cgmath::Rad::zero();
    while win.update() && !win.input.hit(three::KEY_ESCAPE) {
        if let Some(diff) = win.input.timed(three::AXIS_LEFT_RIGHT) {
            angle += cgmath::Rad(1.5 * diff);
            let q = cgmath::Quaternion::from_angle_y(angle);
            mmd_mesh.set_orientation(q);
        }
        win.render(&cam);
    }
}
