use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

/// TODO
// Clean up main game state
// create game over screen and present winner
// create gap for score

const SCREEN_HEIGHT: f32 = 300.;
const SCREEN_WIDTH: f32 = 500.;
// const LEFT_BOUND: f32 = -(SCREEN_WIDTH / 2.);
// const RIGHT_BOUND: f32 = SCREEN_WIDTH / 2.;
const TOP_BOUND: f32 = SCREEN_HEIGHT / 2.;
const BOTTOM_BOUND: f32 = -(SCREEN_HEIGHT / 2.);
const PADDLE_PADDING: f32 = 15.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_SPEED: f32 = 400.;
const HUMAN_PADDLE_START: Vec3 = Vec3::new(-(SCREEN_WIDTH / 2.) + PADDLE_PADDING, 0., 0.);
const CPU_PADDLE_START: Vec3 = Vec3::new((SCREEN_WIDTH / 2.) - PADDLE_PADDING, 0., 0.);
const BALL_RADIUS: f32 = 6.;
const BALL_SPEED: f32 = 65.;
const INIT_BALL_DIRECTION: Vec2 = Vec2::new(-5. * BALL_SPEED, 5. * BALL_SPEED);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Menu,
    InGame,
    GameOver,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Pong".to_string(),
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_event::<ScoreEvent>()
        .add_state(AppState::Menu)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_system_set(SystemSet::on_enter(AppState::Menu).with_system(setup))
        .add_system_set(SystemSet::on_update(AppState::Menu).with_system(enter_game))
        .add_system_set(SystemSet::on_exit(AppState::Menu).with_system(cleanup_menu))
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(ingame_setup))
        // System set update for in game state
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(check_collisions)
                .with_system(reset_pos.after(check_collisions))
                .with_system(check_winner.after(check_collisions))
                .with_system(move_paddle.before(check_collisions))
                .with_system(move_ball.before(check_collisions))
                .with_system(cpu_ai.before(check_collisions)),
        )
        .add_system(bevy::window::close_on_esc)
        .run();
}

/// COMPONENTS
#[derive(Component)]
struct MenuText;

// For human paddle
#[derive(Component)]
struct Paddle;

// CPU Impl
#[derive(Component)]
struct CPUPaddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Debug)]
struct ScoreEvent(String);

#[derive(Component)]
struct HumanScore;

#[derive(Component)]
struct CPUScore;

// Setup Game
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn_bundle(Camera2dBundle::default());

    // test
    commands
        .spawn()
        .insert(MenuText)
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: UiColor(Color::BLACK),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn()
                .insert(MenuText)
                .insert_bundle(TextBundle::from_section(
                    "Pong",
                    TextStyle {
                        font: asset_server.load("OpenSans-Regular.ttf"),
                        font_size: 60.,
                        color: Color::WHITE,
                    },
                ));
            parent
                .spawn()
                .insert(MenuText)
                .insert_bundle(TextBundle::from_section(
                    "Press 'Space' to Play",
                    TextStyle {
                        font: asset_server.load("OpenSans-Regular.ttf"),
                        font_size: 24.,
                        color: Color::WHITE,
                    },
                ));
        });
    // Sound
    // TODO
}

// Despawn menu items
fn cleanup_menu(mut commands: Commands, mut query: Query<Entity, With<MenuText>>) {
    for text in query.iter_mut() {
        commands.entity(text).despawn();
    }
}

// InGame setup
fn ingame_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn Paddle
    let human_paddle = shapes::Rectangle {
        origin: RectangleOrigin::Center,
        extents: Vec2::new(PADDLE_WIDTH, SCREEN_HEIGHT / 3.),
    };

    commands
        .spawn()
        .insert(Paddle)
        .insert_bundle(GeometryBuilder::build_as(
            &human_paddle,
            DrawMode::Fill(FillMode {
                color: Color::WHITE,
                options: FillOptions::default(),
            }),
            Transform {
                translation: HUMAN_PADDLE_START,
                ..default()
            },
        ));

    // CPU Paddle
    let cpu_paddle = shapes::Rectangle {
        origin: RectangleOrigin::Center,
        extents: Vec2::new(PADDLE_WIDTH, SCREEN_HEIGHT / 3.),
    };

    commands
        .spawn()
        .insert(CPUPaddle)
        .insert_bundle(GeometryBuilder::build_as(
            &cpu_paddle,
            DrawMode::Fill(FillMode {
                color: Color::WHITE,
                options: FillOptions::default(),
            }),
            Transform {
                translation: CPU_PADDLE_START,
                ..default()
            },
        ));

    let ball = shapes::Circle {
        radius: BALL_RADIUS,
        center: Vec2::new(0., 0.),
    };

    commands
        .spawn()
        .insert(Ball)
        .insert_bundle(GeometryBuilder::build_as(
            &ball,
            DrawMode::Fill(FillMode {
                color: Color::WHITE,
                options: FillOptions::default(),
            }),
            Transform::default(),
        ))
        .insert(Velocity(INIT_BALL_DIRECTION));

    // Show Score
    commands
        .spawn()
        .insert_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            // Player score
            parent
                .spawn()
                .insert(HumanScore)
                .insert_bundle(TextBundle::from_section(
                    "0",
                    TextStyle {
                        font: asset_server.load("OpenSans-Regular.ttf"),
                        font_size: 80.,
                        color: Color::WHITE,
                    },
                ));

            // CPU score
            parent
                .spawn()
                .insert(CPUScore)
                .insert_bundle(TextBundle::from_section(
                    "0",
                    TextStyle {
                        font: asset_server.load("OpenSans-Regular.ttf"),
                        font_size: 80.,
                        color: Color::WHITE,
                    },
                ));
        });
}

// Player input
fn move_paddle(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if input.pressed(KeyCode::W) {
        direction += 1.0;
    }
    if input.pressed(KeyCode::S) {
        direction -= 1.0;
    }

    let new_paddle_position =
        paddle_transform.translation.y + (direction * time.delta_seconds() * PADDLE_SPEED);

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
fn check_collisions(
    mut ball_query: Query<(&Transform, &mut Velocity), With<Ball>>,
    mut paddle_query: Query<&Transform, With<Paddle>>,
    cpu_query: Query<&Transform, With<CPUPaddle>>,
    mut score_event: EventWriter<ScoreEvent>,
) {
    let (ball_transform, mut velocity) = ball_query.single_mut();
    let paddle_transform = paddle_query.single_mut();
    let cpu_transform = cpu_query.single();

    let paddle_top = paddle_transform.translation.y + (SCREEN_HEIGHT / 3.) / 2.;
    let paddle_bottom = paddle_transform.translation.y - (SCREEN_HEIGHT / 3.) / 2.;
    let cpu_paddle_top = cpu_transform.translation.y + (SCREEN_HEIGHT / 3.) / 2.;
    let cpu_paddle_bottom = cpu_transform.translation.y - (SCREEN_HEIGHT / 3.) / 2.;

    // collision logic and if no collision emit score event
    if (ball_transform.translation.x <= paddle_transform.translation.x + (PADDLE_WIDTH / 2.)
        && (ball_transform.translation.y <= paddle_top
            && ball_transform.translation.y >= paddle_bottom))
        || (ball_transform.translation.x >= cpu_transform.translation.x - (PADDLE_WIDTH / 2.)
            && (ball_transform.translation.y <= cpu_paddle_top)
            && ball_transform.translation.y >= cpu_paddle_bottom)
    {
        velocity.0.x = -velocity.0.x;
    } else if ball_transform.translation.x < paddle_transform.translation.x + (PADDLE_WIDTH / 2.) {
        score_event.send(ScoreEvent(String::from("c")));
    } else if ball_transform.translation.x > cpu_transform.translation.x - (PADDLE_WIDTH / 2.) {
        score_event.send(ScoreEvent(String::from("h")));
    }
}

fn reset_pos(
    mut score_event: EventReader<ScoreEvent>,
    mut set: ParamSet<(
        Query<&mut Transform, With<Paddle>>,
        Query<&mut Transform, With<CPUPaddle>>,
        Query<&mut Transform, With<Ball>>,
        Query<&mut Text, With<HumanScore>>,
        Query<&mut Text, With<CPUScore>>,
    )>,
) {
    for event in score_event.iter() {
        info!("EVENT: Reset Positions, TYPE: {:?}", event);
        for mut trans in set.p0().iter_mut() {
            trans.translation = HUMAN_PADDLE_START;
        }

        for mut trans in set.p1().iter_mut() {
            trans.translation = CPU_PADDLE_START;
        }

        for mut trans in set.p2().iter_mut() {
            trans.translation = Vec3::new(0., 0., 0.)
        }

        // Update score, convert strings to int and then back
        if event.0 == "c" {
            for mut text in &mut set.p4().iter_mut() {
                let num = &text.sections[0].value;
                text.sections[0].value = (num.parse::<u8>().unwrap() + 1).to_string();
            }
        }
    }
}

// AI for cpu paddle
fn cpu_ai(
    mut set: ParamSet<(
        Query<&Transform, With<Ball>>,
        Query<&mut Transform, With<CPUPaddle>>,
    )>,
) {
    let ball_y = set.p0().single().translation.y;

    // create bounds
    let top_bound = 150. - ((SCREEN_HEIGHT / 3.) / 2.);
    let bottom_bound = -150. + ((SCREEN_HEIGHT / 3.) / 2.);

    // Move paddle to current ball position
    for mut trans in set.p1().iter_mut() {
        trans.translation.y = ball_y.clamp(bottom_bound, top_bound);
    }
}

fn enter_game(mut keys: ResMut<Input<KeyCode>>, mut app_state: ResMut<State<AppState>>) {
    if keys.just_pressed(KeyCode::Space) {
        app_state.set(AppState::InGame).unwrap();
        keys.reset(KeyCode::Space);
    }
}

fn check_winner(
    mut set: ParamSet<(Query<&Text, With<HumanScore>>, Query<&Text, With<CPUScore>>)>,
    mut app_state: ResMut<State<AppState>>,
) {
    for text in set.p0().iter() {
        if text.sections[0].value == "8" {
            app_state.set(AppState::GameOver).unwrap();
        }
    }
    for text in set.p1().iter() {
        if text.sections[0].value == "8" {
            app_state.set(AppState::GameOver).unwrap();
        }
    }
}
