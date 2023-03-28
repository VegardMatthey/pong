use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

use rand::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const BALL_COLOR: Color = Color::rgb(0.5, 1., 0.5);
const BALL_SPEED: f32 = 600.0;

const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const LEFT_WALL: f32 = -500.; // x coordinates
const RIGHT_WALL: f32 = 500.;
const BOTTOM_WALL: f32 = -300.; // y coordinates
const TOP_WALL: f32 = 300.;

const SCOREBOARD_FONT_SIZE: f32 = 128.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const SCOREBOARD_TEXT_CENTER: Val = Val::Px(960.0);

const PADDLE_SPEED: f32 = 800.;
const PADDLE_SIZE: Vec3 = Vec3::new(128.0, 32.0, 0.0);
const PADDLE_PADDING: f32 = 10.0;

const PLAYER1_COLOR: Color = Color::rgb(0.5, 0.5, 1.);
const PLAYER2_COLOR: Color = Color::rgb(1., 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard {
            player_1_score: 0,
            player_2_score: 0,
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_systems(
            (
                collision,
                move_player1,
                move_player2,
                velocity.after(collision),
                reset.after(update_scoreboard),
                update_scoreboard,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(bevy::window::close_on_esc)
        .run();
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Player1;

#[derive(Component)]
struct Player2;

#[derive(Resource)]
struct Scoreboard {
    player_1_score: usize,
    player_2_score: usize,
}

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Top));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(1.0, 1.0).normalize() * BALL_SPEED),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: PLAYER1_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec2::new(0.0, 250.0).extend(0.0),
                scale: Vec3::new(PADDLE_SIZE.x, PADDLE_SIZE.y, 1.0),
                ..default()
            },
            ..default()
        },
        Player1,
        Paddle,
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: PLAYER2_COLOR,
                ..default()
            },
            transform: Transform {
                translation: Vec2::new(0.0, -250.0).extend(0.0),
                scale: Vec3::new(PADDLE_SIZE.x, PADDLE_SIZE.y, 1.0),
                ..default()
            },
            ..default()
        },
        Player2,
        Paddle,
    ));

    commands.spawn(
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: SCOREBOARD_FONT_SIZE,
            color: PLAYER1_COLOR,
        })])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_CENTER,
                ..default()
            },
            ..default()
        }),
    );
    commands.spawn((
        TextBundle::from_sections([TextSection::from_style(TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: SCOREBOARD_FONT_SIZE,
            color: PLAYER2_COLOR,
        })])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_CENTER,
                ..default()
            },
            ..default()
        }),
    ));
}

fn move_player1(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player1>>,
) {
    let mut player_direction = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        player_direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::D) {
        player_direction += 1.0;
    }

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    let mut player_transform = player_query.single_mut();
    let new_player_position =
        player_transform.translation.x + player_direction * PADDLE_SPEED * TIME_STEP;
    player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
}

fn move_player2(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player2>>,
) {
    let mut player_direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        player_direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        player_direction += 1.0;
    }

    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
    let mut player_transform = player_query.single_mut();

    let new_player_position =
        player_transform.translation.x + player_direction * PADDLE_SPEED * TIME_STEP;

    player_transform.translation.x = new_player_position.clamp(left_bound, right_bound);
}

fn velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn collision(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    paddle_query: Query<&Transform, With<Paddle>>,
    collider_query: Query<&Transform, With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    for transform in &collider_query {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send_default();

            let mut reflect_x = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => {
                    scoreboard.player_1_score += 1;
                    ball_velocity.x = 0.0;
                    ball_velocity.y = 0.0;
                }
                Collision::Bottom => {
                    scoreboard.player_2_score += 1;
                    ball_velocity.x = 0.0;
                    ball_velocity.y = 0.0;
                }
                Collision::Inside => ()
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }
        }
    }
    for paddle_transform in &paddle_query {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            paddle_transform.translation,
            paddle_transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send_default();
            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => ()
            }

            let mut rng = rand::thread_rng();
            let x: f32 = rng.gen_range(-10.0..10.0);
            
            if reflect_x {
                ball_velocity.x = -ball_velocity.x + x;
            }
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn reset(
    mut commands: Commands,
    query: Query<(Entity, &Velocity), With<Ball>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (entity, velocity) = query.single();
    if velocity.x != 0.0 {
        return;
    }
    commands.entity(entity).despawn();
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen_range(-1.0..1.0);
    let y: f32 = rng.gen_range(-1.0..1.0);
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(x, y).normalize() * BALL_SPEED),
    ));
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut x = true;
    for mut text in &mut query {
        if scoreboard.player_1_score > 9 {
            text.sections[0].value = "Player 1 Wins!".to_string();
        } else if scoreboard.player_2_score > 9 {
            text.sections[0].value = "Player 2 Wins!".to_string();
        } else if x {
            text.sections[0].value = (scoreboard.player_1_score / 2).to_string();
        } else {
            text.sections[0].value = (scoreboard.player_2_score / 2).to_string();
        }
        x = false;
    }
}
