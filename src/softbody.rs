use std::iter;
use three_d::*;

mod bunny;
mod visuals;

type Vec3f = Vector3<f32>;

pub fn demo() {
    log::warn!("Logging works !");
    let window = visuals::make_window();
    let context = window.gl();
    let mut scene = visuals::Scene::new(window.viewport(), &context);
    let axes = Axes::new(&context, 0.03, 1.0);
    let main_material = visuals::colormaterial(&context, Color::RED);
    let ground_material = visuals::colormaterial(&context, Color::new(95, 90, 100, 255));

    let mut ground_mesh = CpuMesh::square();
    ground_mesh
        .transform(
            &(Matrix4::from_scale(5.) * Matrix4::from_angle_x(Rad(std::f32::consts::PI / 2.))),
        )
        .unwrap();
    ground_mesh.compute_normals();

    let mut bunny = RenderSoftBody::new_bunny();
    let mut gui = three_d::GUI::new(&context);
    let mut niter = 1;
    let mut edge_comp = 0.03;
    let mut vol_comp = 0.0;

    window.render_loop(move |mut frame_input: FrameInput| {
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    use three_d::egui::*;
                    ui.heading("Softbody simulation");
                    ui.add(Label::new("A simple softbody solver, using semi-implicit Euler integration and Gauss-Seidel constraint solving"));
                    ui.add(Slider::new(&mut niter, 1..=30).text("Number of physics iterations per frame"));
                    ui.add(Slider::new(&mut edge_comp, 0.0..=2.0).text("Edge \"stretch\" compliance (unit?)"));
                    ui.add(Slider::new(&mut vol_comp, 0.0..=0.01).text("Volume \"squash\" compliance (unit?)"));
                });
                panel_width = gui_context.used_rect().width() as f64;
            },
        );

        bunny.update_physics(|body| {
            for _ in 0..niter {
                update_softbody(body, 1./(30.*niter as f32), edge_comp, vol_comp);
            }
        });
        bunny.mesh.compute_normals();

        let bunny_gpu = Gm::new(Mesh::new(&context, &bunny.mesh), main_material.clone());
        let ground = Gm::new(Mesh::new(&context, &ground_mesh), ground_material.clone());

        let objects: Vec<&dyn Object> = vec![&bunny_gpu, &axes, &ground];

        let viewport = Viewport {
            x: (panel_width * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width
                - (panel_width * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        scene.camera.set_viewport(viewport);

        scene.control.handle_events(&mut scene.camera, &mut frame_input.events);
        let clearcol = ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0);
        let light_ref: Vec<_> = scene.lights.iter().map(|x| x.as_ref()).collect();
        frame_input
            .screen()
            .clear(clearcol)
            .render(&scene.camera, objects, &light_ref)
            .write(|| gui.render());
        FrameOutput::default()
    });
}

struct RenderSoftBody {
    pub softbody: SoftBody,
    pub mesh: CpuMesh,
}

impl RenderSoftBody {
    pub fn update_physics(&mut self, fun: impl Fn(&mut SoftBody)) {
        self.swap_buffers();
        fun(&mut self.softbody);
        self.swap_buffers();
        debug_assert!(self.softbody.positions.is_empty() && self.softbody.indices.is_empty(), "The softbody is supposed to be in rendering mode but it's simulation buffers are not empty");
    }

    pub fn new_bunny() -> Self {
        let positions: Vec<Vec3f> = bunny::VERTICES
            .chunks(3)
            .map(|v| vec3(v[0], v[1] + 2.0, v[2]))
            .collect();
        let indices = Indices::U32(bunny::TET_SURFACE_TRI_IDS.into());
        let edges: Vec<[u32; 2]> = bunny::TET_EDGE_IDS
            .chunks(2)
            .map(|e| -> [u32; 2] { e.try_into().expect("couldn't convert to owned array") })
            .collect();
        let edges_rest_len: Vec<f32> = edges
            .iter()
            .map(|e| (positions[e[1] as usize] - positions[e[0] as usize]).magnitude())
            .collect();
        let tetras: Vec<[u32; 4]> = bunny::TET_IDS
            .chunks(4)
            .map(|e| -> [u32; 4] { e.try_into().expect("failed to convert to [u32;4]") })
            .collect();
        let tetras_rest_vols: Vec<f32> = tetras
            .iter()
            .map(|t| {
                1.0 * tetra_vol([
                    positions[t[0] as usize],
                    positions[t[1] as usize],
                    positions[t[2] as usize],
                    positions[t[3] as usize],
                ])
            })
            .collect();
        let speeds = vec![Vec3f::new(0., 0., 0.); bunny::VERTICES.len() / 3];

        Self {
            softbody: SoftBody {
                positions: vec![],
                speeds,
                indices: vec![],
                edges,
                edges_rest_len,
                tetras,
                tetras_rest_vols,
            },
            mesh: CpuMesh {
                positions: Positions::F32(positions),
                indices,
                ..Default::default()
            },
        }
    }

    fn swap_buffers(&mut self) {
        // toggles the vectors between the rendering and the simulating
        std::mem::swap(
            &mut self.softbody.positions,
            Self::position_to_vec3f32(&mut self.mesh.positions),
        );
        std::mem::swap(
            &mut self.softbody.indices,
            Self::indices_to_u32(&mut self.mesh.indices),
        );
    }

    fn position_to_vec3f32(pos: &mut Positions) -> &mut Vec<Vec3f> {
        match pos {
            Positions::F32(v) => v,
            _ => panic!("Did not match Positions::F32"),
        }
    }
    fn indices_to_u32(idx: &mut Indices) -> &mut Vec<u32> {
        match idx {
            Indices::U32(v) => v,
            _ => panic!("Did not match Indices::U32"),
        }
    }
}

struct SoftBody {
    pub positions: Vec<Vec3f>,
    pub speeds: Vec<Vec3f>,
    pub indices: Vec<u32>,
    pub edges: Vec<[u32; 2]>,
    pub edges_rest_len: Vec<f32>,
    pub tetras: Vec<[u32; 4]>,
    pub tetras_rest_vols: Vec<f32>,
}

fn update_softbody(body: &mut SoftBody, dt: f32, e_comp: f32, v_comp: f32) {
    gravity_and_ground(body, dt, Vec3f::new(0., -9.81/8., 0.));
    solve_edges(body, dt, e_comp);
    solve_volumes(body, dt, v_comp);
    // Order :
    //- SemiImpl Euler
    //-- v += a(x)*dt
    //-- x += v*dt
    //- Constraints
    //-- x += dx
    //-- v += dx/dt
}

fn gravity_and_ground(body: &mut SoftBody, dt: f32, g: Vec3f) {
    for (x, v) in iter::zip(&mut body.positions, &mut body.speeds) {
        let g = vec3(0.,2., 0.) - *x;
        let g = g / 100.;
        *v += dt * g;
        let xprev = *x;
        *x += dt * *v;
        if x.y < 0.1 {
            *x = xprev;
            x.y = 0.1;
            v.y = 0.0; // no rebond ?
            *v *= 0.1; // MEH, friction artificielle
        }
    }
}

fn solve_edges(body: &mut SoftBody, dt: f32, comp: f32) {
    let a = comp / (dt * dt);

    let c_and_gradc = |v0: Vec3f, v1: Vec3f, restlen: f32| -> (f32, [Vec3f; 2]) {
        let v = v0 - v1;
        let d = v.magnitude();
        (d - restlen, [v / d, -v / d])
    };

    for (edge, restlen) in iter::zip(&body.edges, &body.edges_rest_len) {
        let v0 = body.positions[edge[0] as usize];
        let v1 = body.positions[edge[1] as usize];
        let (c, gradc) = c_and_gradc(v0, v1, *restlen);
        let w = [1., 1.]; // TODO!
        let tmp: f32 = iter::zip(w, gradc)
            .map(|(wi, gradi)| wi * gradi.magnitude2())
            .sum();
        let lambda = -c / (tmp + a * restlen);
        body.positions[edge[0] as usize] += lambda * w[0] * gradc[0];
        body.positions[edge[1] as usize] += lambda * w[1] * gradc[1];
        body.speeds[edge[0] as usize] += lambda * w[0] * gradc[0] / dt;
        body.speeds[edge[1] as usize] += lambda * w[1] * gradc[1] / dt;
    }
}

fn tetra_vol(vs: [Vec3f; 4]) -> f32 {
    let a = vs[1] - vs[0];
    let b = vs[2] - vs[0];
    let c = vs[3] - vs[0];
    a.cross(b).dot(c) / 6.
}

fn solve_volumes(body: &mut SoftBody, dt: f32, comp: f32) {
    let a = comp / (dt * dt);

    let c_and_gradc = |vs: [Vec3f; 4], restvol: f32| -> (f32, [Vec3f; 4]) {
        (
            tetra_vol(vs) - restvol,
            [
                /*
                -1. / 6. * (vs[1] - vs[3]).cross(vs[2] - vs[3]),
                -1. / 6. * (vs[0] - vs[2]).cross(vs[3] - vs[2]),
                -1. / 6. * (vs[3] - vs[1]).cross(vs[0] - vs[1]),
                -1. / 6. * (vs[2] - vs[0]).cross(vs[1] - vs[0]),
                */
                1. / 6. * (vs[3] - vs[1]).cross(vs[2] - vs[1]),
                1. / 6. * (vs[2] - vs[0]).cross(vs[3] - vs[0]),
                1. / 6. * (vs[3] - vs[0]).cross(vs[1] - vs[0]),
                1. / 6. * (vs[1] - vs[0]).cross(vs[2] - vs[0]),
            ],
        )
    };

    for (tetra, restvol) in iter::zip(&body.tetras, &body.tetras_rest_vols) {
        let vs = [
            body.positions[tetra[0] as usize],
            body.positions[tetra[1] as usize],
            body.positions[tetra[2] as usize],
            body.positions[tetra[3] as usize],
        ];
        let (c, gradc) = c_and_gradc(vs, *restvol);
        let w = [1., 1., 1., 1.]; // TODO!
        let tmp: f32 = iter::zip(w, gradc)
            .map(|(wi, gradi)| wi * gradi.magnitude2())
            .sum();
        let lambda = -c / (tmp + a * restvol);

        body.positions[tetra[0] as usize] += lambda * w[0] * gradc[0];
        body.positions[tetra[1] as usize] += lambda * w[1] * gradc[1];
        body.positions[tetra[2] as usize] += lambda * w[2] * gradc[2];
        body.positions[tetra[3] as usize] += lambda * w[3] * gradc[3];
        body.speeds[tetra[0] as usize] += lambda * w[0] * gradc[0] / dt;
        body.speeds[tetra[1] as usize] += lambda * w[1] * gradc[1] / dt;
        body.speeds[tetra[2] as usize] += lambda * w[2] * gradc[2] / dt;
        body.speeds[tetra[3] as usize] += lambda * w[3] * gradc[3] / dt;
    }
}
