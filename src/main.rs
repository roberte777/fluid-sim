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
}

const STARTING_RADIUS: f32 = 0.5;
const STARTING_WIDTH: f32 = 15.;
const STARTING_HEIGHT: f32 = 15.;
const STARTING_DAMPING: f32 = 1.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
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
    // println!("{}-{}", window.width(), window.height());
    commands.spawn(Camera2dBundle {
        transform: Transform::from_scale(Vec3::new(0.022, 0.022, 0.022)),
        ..default()
    });

    // Circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Circle::new(STARTING_RADIUS).into())
                .into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            transform: Transform::from_translation(Vec3::new(-0., 0., 0.)),
            ..default()
        },
        Velocity(Vec2::default()),
        Ball {
            radius: STARTING_RADIUS,
            damping: STARTING_DAMPING,
        },
    ));

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
        let mut ball = ball_query.single_mut();
        ui.label("Ball radius:");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut ball.radius).speed(0.1));
        });
        ui.label("Damping:");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut ball.damping).speed(0.1));
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
    time: Res<Time>,
) {
    let gravity = Vec2::new(0.0, -9.8);
    let mut force = Vec2::new(0., 0.);
    force += gravity;
    for (mut transform, mut velocity, ball) in query.iter_mut() {
        let bounding_box = bounding_box_query.single();
        velocity.0 += force * time.delta_seconds();
        let position = Vec3::new(
            transform.translation.x + (velocity.0.x * time.delta_seconds()),
            transform.translation.y + (velocity.0.y * time.delta_seconds()),
            0.,
        );
        transform.translation = position;

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
    }
}
