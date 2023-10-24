use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct BoundingBox {
    width: f32,
    height: f32,
}

#[derive(Component)]
struct Ball {
    radius: f32,
    damping: f32,
    density: f32,
    pressure: f32,
    predicted_position: Vec2,
}

const STARTING_RADIUS: f32 = 0.35;
const STARTING_WIDTH: f32 = 150.;
const STARTING_HEIGHT: f32 = 90.;
const STARTING_DAMPING: f32 = 0.95;
const NUM_PARTICLES: usize = 1500;
const PARTICLE_SPACING: f32 = 1.;
const RADIUS_OF_INFLUENCE: f32 = 3.;
const COLUMNS: usize = 50; // Specify the number of columns

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, sph_system)
        .add_systems(Update, gravity)
        .add_systems(Update, ui_example_system)
        .add_systems(Update, update_box_mesh_system)
        .add_systems(Update, update_ball_mesh_system)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // window: Window,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.15, 0.15, 0.15)),
        ..default()
    });

    // Circle
    let rows = (NUM_PARTICLES as f32 / COLUMNS as f32).ceil() as usize;

    let total_width = (COLUMNS - 1) as f32 * PARTICLE_SPACING; // Total width particles will occupy
    let total_height = (rows - 1) as f32 * PARTICLE_SPACING; // Total height particles will occupy

    let start_x = -total_width / 2.0; // Starting x-coordinate to center the particles
    let start_y = -total_height / 2.0; // Starting y-coordinate to center the particles

    for i in 0..COLUMNS {
        for j in 0..rows {
            // Skip creating particles if we have already created the specified number
            if (i * rows + j) >= NUM_PARTICLES {
                break;
            }

            let x_position = start_x + (i as f32 * PARTICLE_SPACING) + STARTING_RADIUS;
            let y_position = start_y + (j as f32 * PARTICLE_SPACING) + STARTING_RADIUS;

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(shape::Circle::new(STARTING_RADIUS).into())
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::PURPLE)),
                    transform: Transform::from_translation(Vec3::new(x_position, y_position, 0.)),
                    ..default()
                },
                Velocity(Vec2::default()),
                Ball {
                    radius: STARTING_RADIUS,
                    damping: STARTING_DAMPING,
                    density: 0.,
                    pressure: 0.,
                    predicted_position: Vec2::default(),
                },
            ));
        }
    }

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Box::new(STARTING_WIDTH, STARTING_HEIGHT, 0.).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE.with_a(0.2))),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        },
        BoundingBox {
            width: STARTING_WIDTH,
            height: STARTING_HEIGHT,
        },
    ));
}
fn ui_example_system(
    mut ball_query: Query<&mut Ball>,
    mut box_query: Query<&mut BoundingBox>,
    mut contexts: EguiContexts,
) {
    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        let mut first_radius;
        let mut first_damping;
        {
            let ball = ball_query.iter().next().unwrap();
            first_radius = ball.radius;
            first_damping = ball.damping;
        }
        ui.label("Ball radius:");
        ui.horizontal(|ui| {
            if ui
                .add(egui::DragValue::new(&mut first_radius).speed(0.1))
                .changed()
            {
                for mut ball in ball_query.iter_mut() {
                    ball.radius = first_radius;
                }
            }
        });
        ui.label("Damping:");
        ui.horizontal(|ui| {
            if ui
                .add(egui::DragValue::new(&mut first_damping).speed(0.1))
                .changed()
            {
                for mut ball in ball_query.iter_mut() {
                    ball.damping = first_damping;
                }
            }
        });
        let mut box_data = box_query.single_mut();
        ui.label("Box width:");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut box_data.width).speed(1.0));
        });
        ui.label("Box height:");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut box_data.height).speed(1.0));
        });
    });
}
fn update_ball_mesh_system(mut ball_query: Query<(&Ball, &mut Transform), Changed<Ball>>) {
    for (ball, mut transform) in ball_query.iter_mut() {
        transform.scale = Vec3::new(
            ball.radius / STARTING_RADIUS,
            ball.radius / STARTING_RADIUS,
            1.0,
        );
    }
}

fn update_box_mesh_system(
    mut box_query: Query<(&BoundingBox, &mut Transform), Changed<BoundingBox>>,
) {
    for (box_data, mut transform) in box_query.iter_mut() {
        transform.scale = Vec3::new(
            box_data.width / STARTING_WIDTH,
            box_data.height / STARTING_HEIGHT,
            1.0,
        );
    }
}
fn gravity(
    mut query: Query<(&mut Transform, &mut Velocity, &Ball)>,
    bounding_box_query: Query<&BoundingBox>,
) {
    for (mut transform, mut velocity, ball) in query.iter_mut() {
        let bounding_box = bounding_box_query.single();

        let half_bound_size: Vec2 =
            Vec2::new(bounding_box.width, bounding_box.height) / 2. - Vec2::ONE * ball.radius;

        if transform.translation.x.abs() > half_bound_size.x {
            transform.translation.x = half_bound_size.x * transform.translation.x.signum();
            velocity.0.x *= -1. * ball.damping;
        }
        if transform.translation.y.abs() > half_bound_size.y {
            // Calculate how much the ball has penetrated the boundary
            let penetration = transform.translation.y.abs() - half_bound_size.y;
            // Adjust the ball's position to ensure it doesn't penetrate the boundary
            transform.translation.y =
                (half_bound_size.y - penetration) * transform.translation.y.signum();
            velocity.0.y *= -1. * ball.damping;
        }
        // if transform.translation.y.abs() > half_bound_size.y {
        //     transform.translation.y = half_bound_size.y * transform.translation.y.signum();
        //     velocity.0.y *= -1. * ball.damping;
        // }
    }
}

fn sph_system(mut ball_query: Query<(&mut Ball, &mut Velocity, &mut Transform)>, time: Res<Time>) {
    const GAS_CONSTANT: f32 = 500.0;
    const REST_DENSITY: f32 = 3.;
    let gravity = Vec2::new(0.0, -9.8);
    // let time_step = time.delta_seconds();
    let time_step = 1. / 60.;
    // predict next positions
    for (mut ball, mut velocity, mut transform) in ball_query.iter_mut() {
        velocity.0 += gravity * time_step;
        let position = Vec3::new(
            transform.translation.x + (velocity.0.x * time_step),
            transform.translation.y + (velocity.0.y * time_step),
            0.,
        );
        ball.predicted_position = position.truncate();
    }
    // Density computation for each ball
    let mut ball_query_vec = ball_query.iter_mut().collect::<Vec<_>>();
    // Assuming ball_query can be converted to Vec
    let len = ball_query_vec.len();

    for i in 0..len {
        let mut density = 0.0;

        for j in 0..len {
            let r = ball_query_vec[i]
                .0
                .predicted_position
                .distance(ball_query_vec[j].0.predicted_position);
            // summation of mass * smoothing kernel
            // assuming mass is 1
            density += 1. * spiky(r, RADIUS_OF_INFLUENCE);
        }

        ball_query_vec[i].0.density = density;
        // Pressure computation
        ball_query_vec[i].0.pressure = GAS_CONSTANT * (density - REST_DENSITY);
    }

    // Pressure force computation
    // - summation of mass * (pressure_a + pressure_b) / 2(density_b) * spiky_gradient(smoothing_kernel)
    // mew * summation of mass * (vj - vi) / (density_j) * viscosity_laplacian(smoothing_kernel)
    for i in 0..len {
        let mut force = Vec2::new(0., 0.);
        let mut pressure_force = Vec2::ZERO;

        for j in 0..len {
            // Skip computation for the same ball

            let r = ball_query_vec[i].0.predicted_position - ball_query_vec[j].0.predicted_position;

            pressure_force += compute_pressure_force(
                &ball_query_vec[i].0,
                &ball_query_vec[j].0,
                r,
                RADIUS_OF_INFLUENCE,
            );
        }

        let density = ball_query_vec[i].0.density;
        if density == 0. {
            continue;
        }
        force += pressure_force / density;
        // update positions
        ball_query_vec[i].1 .0 += force * time_step;
        // ball_query_vec[i].1 .0 = ball_query_vec[i].1 .0;
        let position = Vec3::new(
            ball_query_vec[i].2.translation.x + (ball_query_vec[i].1 .0.x * time_step),
            ball_query_vec[i].2.translation.y + (ball_query_vec[i].1 .0.y * time_step),
            0.,
        );
        // panic of position is nan
        if position.x.is_nan() || position.y.is_nan() {
            panic!("position is nan");
        }
        ball_query_vec[i].2.translation = position;
    }
}
fn compute_pressure_force(ball_a: &Ball, ball_b: &Ball, r: Vec2, h: f32) -> Vec2 {
    if r.length() == 0. {
        return Vec2::ZERO;
    }
    let dir = r.normalize();
    let dw = spiky_der(r.length(), h);
    if ball_b.density == 0. {
        // random vec
        // return Vec2::new(rand::random::<f32>(), rand::random::<f32>());
        return Vec2::ZERO;
    }
    let shared_pressure = (ball_a.pressure + ball_b.pressure) / 2.;
    // ball_b.pressure * dir * dw * 1. / ball_b.density
    shared_pressure * dir * dw * 1. / ball_b.density
}

fn spiky(dst: f32, radius: f32) -> f32 {
    let SPIKY_POW2_SCALING_FACTOR: f32 = 6. / (std::f32::consts::PI * RADIUS_OF_INFLUENCE.powi(4));

    if dst < radius {
        let v = radius - dst;
        return v * v * SPIKY_POW2_SCALING_FACTOR;
    }
    0.0
}

fn spiky_der(dst: f32, radius: f32) -> f32 {
    let SPIKY_POW2_SCALING_FACTOR: f32 = 12. / (std::f32::consts::PI * RADIUS_OF_INFLUENCE.powi(4));

    if dst < radius {
        return (dst - radius) * SPIKY_POW2_SCALING_FACTOR;
    }
    0.0
}
