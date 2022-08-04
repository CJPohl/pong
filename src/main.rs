use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

const SCREEN_HEIGHT :f32  = 300.;
const SCREEN_WIDTH: f32 = 500.;
// const LEFT_BOUND: f32 = -(SCREEN_WIDTH / 2.);
// const RIGHT_BOUND: f32 = SCREEN_WIDTH / 2.;
const TOP_BOUND: f32 = SCREEN_HEIGHT / 2.;
const BOTTOM_BOUND: f32 = -(SCREEN_HEIGHT / 2.);
const PADDLE_PADDING: f32 = 15.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_SPEED: f32 = 400.;
const BALL_RADIUS: f32 = 6.; 
const BALL_SPEED: f32 = 65.;
const INIT_BALL_DIRECTION: Vec2 = Vec2::new(-5. * BALL_SPEED, 5. * BALL_SPEED);

fn main() {
    App::new()
        .insert_resource(Msaa {samples: 4})
        .insert_resource(WindowDescriptor {
            title: "Pong".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(setup)
        .add_system_set(SystemSet::new()
                        .with_system(check_collisions)
                        .with_system(move_paddle.before(check_collisions))
                        .with_system(move_ball.before(check_collisions))
                        .with_system(cpu_ai.before(check_collisions))
                        )
        .add_system(bevy::window::close_on_esc) 
        .run(); 
}   

/// COMPONENTS

// For human paddle
#[derive(Component)]
struct Paddle;

// CPU Impl
#[derive(Component)]
struct CPUPaddle;

#[derive (Component)]
struct Ball;

#[derive (Component)]
struct Velocity(Vec2);

// Setup Game
fn setup(mut commands: Commands, ) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default()); 

    // Sound
    // TODO

    
    // Spawn Paddle
    let human_paddle = shapes::Rectangle {
        origin: RectangleOrigin::Center, extents: Vec2::new(PADDLE_WIDTH, SCREEN_HEIGHT / 3.,)
    }; 

    commands.spawn()
        .insert(Paddle)
        .insert_bundle(GeometryBuilder::build_as(
            &human_paddle,
            DrawMode::Fill(FillMode {color: Color::WHITE, options: FillOptions::default()}),
            Transform { translation: Vec3::new(-(SCREEN_WIDTH / 2.) + PADDLE_PADDING, 0., 0.,), ..default() },
            ));

    // CPU Paddle
    let cpu_paddle = shapes::Rectangle {
        origin: RectangleOrigin::Center, extents: Vec2::new(PADDLE_WIDTH, SCREEN_HEIGHT / 3.,)
    };

    commands.spawn()
        .insert(CPUPaddle)
        .insert_bundle(GeometryBuilder::build_as(
                &cpu_paddle, 
                DrawMode::Fill(FillMode {color: Color::WHITE, options: FillOptions::default()}),
                Transform { translation: Vec3::new((SCREEN_WIDTH / 2.) - PADDLE_PADDING, 0., 0.), ..default() }
               ));
 
    let ball = shapes::Circle {
        radius: BALL_RADIUS, center: Vec2::new(0., 0.,)
    };
    
    commands.spawn()
        .insert(Ball)
        .insert_bundle(GeometryBuilder::build_as(
            &ball,
            DrawMode::Fill(FillMode {color: Color::WHITE, options: FillOptions::default()}),
            Transform::default()
            ))
        .insert(Velocity(INIT_BALL_DIRECTION));
}

// Player input 
fn move_paddle(time: Res<Time>, input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Paddle>>) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if input.pressed(KeyCode::W) {
        direction += 1.0;
    }
    if input.pressed(KeyCode::S) {
        direction -= 1.0;
    }

    let new_paddle_position = paddle_transform.translation.y + (direction * time.delta_seconds() * PADDLE_SPEED);

    // create bounds
    let top_bound = 150. - ((SCREEN_HEIGHT / 3.) / 2.);
    let bottom_bound = -150. + ((SCREEN_HEIGHT / 3.) / 2.);
    
    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}


// Move ball
fn move_ball(time: Res<Time>, mut ball_query: Query<(&mut Transform, &mut Velocity), With<Ball>>) {
    let (mut ball_transform, mut velocity) = ball_query.single_mut();
    
    ball_transform.translation.x += time.delta_seconds() * velocity.0.x;
    ball_transform.translation.y += time.delta_seconds() * velocity.0.y; 
    
    if ball_transform.translation.y >= TOP_BOUND || ball_transform.translation.y <= BOTTOM_BOUND {
        velocity.0.y = -velocity.0.y;
    } 
}

// Check collision
fn check_collisions(mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>, mut paddle_query: Query<&Transform, With<Paddle>>, cpu_query: Query<&Transform, With<CPUPaddle>>) {
    let (ball_transform, mut velocity) = ball_query.single_mut();
    let paddle_transform = paddle_query.single_mut();
    let cpu_transform = cpu_query.single();

    let paddle_top = paddle_transform.translation.y + (SCREEN_HEIGHT / 3.) / 2.;
    let paddle_bottom = paddle_transform.translation.y - (SCREEN_HEIGHT / 3.) / 2.;
    let cpu_paddle_top = cpu_transform.translation.y + (SCREEN_HEIGHT / 3.) / 2.;
    let cpu_paddle_bottom = cpu_transform.translation.y - (SCREEN_HEIGHT / 3.) / 2.;

    if ball_transform.translation.x <= paddle_transform.translation.x + (PADDLE_WIDTH / 2.) && (ball_transform.translation.y <= paddle_top && ball_transform.translation.y >= paddle_bottom) { 
        velocity.0.x = -velocity.0.x;
    }

    if ball_transform.translation.x >= cpu_transform.translation.x - (PADDLE_WIDTH / 2.) && (ball_transform.translation.y <= cpu_paddle_top && ball_transform.translation.y >= cpu_paddle_bottom) {
        velocity.0.x = -velocity.0.x;
    }
    
}

// AI for cpu paddle
fn cpu_ai(mut set: ParamSet<(
        Query<&Transform, With<Ball>>,
        Query<&mut Transform, With<CPUPaddle>>
        )> ) {
    let ball_y = set.p0().single().translation.y;


    // create bounds
    let top_bound = 150. - ((SCREEN_HEIGHT / 3.) / 2.);
    let bottom_bound = -150. + ((SCREEN_HEIGHT / 3.) / 2.);

    // Move paddle to current ball position
    for mut trans in set.p1().iter_mut() {
        trans.translation.y = ball_y.clamp(bottom_bound, top_bound);
    }
}

