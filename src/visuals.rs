use three_d::*;

pub fn colormaterial(context: &Context, col: Color) -> PhysicalMaterial {
    PhysicalMaterial::new(
        context,
        &CpuMaterial {
            albedo: col,
            ..Default::default()
        },
    )
}

pub fn make_window() -> Window {
    Window::new(WindowSettings {
        title: "Shapes!".to_string(),
        max_size: Some((900, 500)),
        vsync: true,
        multisamples: 4,
        ..Default::default()
    })
    .expect("Couldn't create window")
}

pub struct Scene {
    pub control: OrbitControl,
    pub camera: Camera,
    pub lights: Vec<Box<dyn Light>>,
}

impl Scene {
    pub fn new(viewport: Viewport, context: &Context) -> Self {
        let camera = Camera::new_perspective(
            viewport,
            vec3(5.0, 2.0, 2.5),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            1000.0,
        );
        let control = OrbitControl::new(*camera.target(), 1.0, 100.0);

        let lights: Vec<Box<dyn Light>> = vec![
            Box::new(AmbientLight::new(context, 0.02, Color::WHITE)),
            Box::new(DirectionalLight::new(
                context,
                0.15,
                Color::WHITE,
                &vec3(-0.2, -1., 0.),
            )),
            Box::new(PointLight::new(
                context,
                1.0,
                Color::WHITE,
                &vec3(2., 2., 2.),
                Attenuation {
                    constant: 1.,
                    linear: 0.01,
                    quadratic: 0.,
                },
            )),
        ];

        Self {
            control,
            camera,
            lights,
        }
    }

}
