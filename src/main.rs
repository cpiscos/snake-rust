// Snake
// Simple game of snake in Rust using Bevy
use bevy::prelude::*;
use rand::Rng;

const PIXEL_UNIT_SIZE: f32 = 24.0;
const TICKRATE: f64 = 0.08;
const PLAYFIELD: (i32, i32) = (33, 33); // must be odd as snake starts in the middle
const PLAYFIELD_MAX_INDEX: u32 = (PLAYFIELD.0 * PLAYFIELD.1) as u32;

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
    potential_direction: Direction,
    position: (i32, i32),
}

impl SnakeHead {
    fn new() -> Self {
        SnakeHead {
            direction: Direction::Right,
            potential_direction: Direction::Right,
            position: (0, 0),
        }
    }
}

#[derive(Component)]
struct SnakeBody {
    position: (i32, i32),
}

#[derive(Resource)]
struct LastPosition {
    value: (i32, i32),
}

#[derive(Component)]
struct Apple {
    position: (i32, i32),
}

#[derive(Event)]
struct AppleEaten;

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LastPosition { value: (0, 0) })
        .add_systems(Startup, (setup_ui, setup_snake))
        .add_systems(
            Update,
            (
                spawn_apple,
                player_input,
                border_collision,
                snake_body_collision.after(move_snake_head),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                move_snake_body,
                move_snake_head.after(move_snake_body),
                grow_snake_body.after(move_snake_head),
            ),
        )
        .add_event::<AppleEaten>()
        .insert_resource(Time::<Fixed>::from_seconds(TICKRATE))
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(NodeBundle {
        style: Style {
            border: UiRect::all(Val::Px(1.0)),
            width: Val::Px(PLAYFIELD.0 as f32 * PIXEL_UNIT_SIZE),
            height: Val::Px(PLAYFIELD.1 as f32 * PIXEL_UNIT_SIZE),
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
        },
        border_color: Color::BLACK.into(),
        ..default()
    });
    commands.spawn((SpriteBundle {
        sprite: Sprite {
            color: Color::GRAY.into(),
            custom_size: Some(Vec2::new(
                PLAYFIELD.0 as f32 * PIXEL_UNIT_SIZE,
                PLAYFIELD.1 as f32 * PIXEL_UNIT_SIZE,
            )),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
        ..default()
    },));
}

fn setup_snake(mut commands: Commands, mut last_position: ResMut<LastPosition>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::GREEN.into(),
                custom_size: Some(Vec2::new(PIXEL_UNIT_SIZE, PIXEL_UNIT_SIZE)),
                ..default()
            },
            ..default()
        },
        SnakeHead::new(),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE.into(),
                custom_size: Some(Vec2::new(PIXEL_UNIT_SIZE, PIXEL_UNIT_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-PIXEL_UNIT_SIZE, 0.0, 100.0)),
            ..default()
        },
        SnakeBody { position: (-1, 0) },
    ));

    last_position.value = (-2, 0);
}

fn get_valid_apple_spawn(used_positions: Vec<(i32, i32)>) -> (i32, i32) {
    let mut valid_spawn = rand::thread_rng().gen_range(0..PLAYFIELD_MAX_INDEX) as i32;
    while used_positions.contains(&(
        valid_spawn % PLAYFIELD.0 - PLAYFIELD.0 / 2,
        valid_spawn / PLAYFIELD.1 - PLAYFIELD.1 / 2,
    )) {
        valid_spawn = rand::thread_rng().gen_range(0..PLAYFIELD_MAX_INDEX) as i32;
    }

    (
        (valid_spawn % PLAYFIELD.0 - PLAYFIELD.0 / 2) as i32,
        (valid_spawn / PLAYFIELD.1 - PLAYFIELD.1 / 2) as i32,
    )
}

fn spawn_apple(
    mut commands: Commands,
    snake_head_query: Query<&SnakeHead>,
    snake_body_query: Query<&SnakeBody>,
    apple_query: Query<&Apple>,
) {
    if !apple_query.is_empty() {
        return;
    }

    let mut snake_positions = Vec::new();
    let snake_head = snake_head_query.single();
    for snake_body in &snake_body_query {
        snake_positions.push(snake_body.position);
    }
    snake_positions.push(snake_head.position);
    let valid_spawn = get_valid_apple_spawn(snake_positions);
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED.into(),
                custom_size: Some(Vec2::new(PIXEL_UNIT_SIZE, PIXEL_UNIT_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                valid_spawn.0 as f32 * PIXEL_UNIT_SIZE,
                valid_spawn.1 as f32 * PIXEL_UNIT_SIZE,
                0.0,
            )),
            ..default()
        },
        Apple {
            position: valid_spawn,
        },
    ));
}

fn move_snake_head(
    mut commands: Commands,
    mut snake_head_query: Query<(&mut SnakeHead, &mut Transform)>,
    apple_query: Query<(Entity, &mut Apple)>,
    mut apple_eaten_event: EventWriter<AppleEaten>,
) {
    let (mut snake_head, mut transform) = snake_head_query.single_mut();
    match snake_head.potential_direction {
        Direction::Up => {
            if snake_head.direction != Direction::Down {
                snake_head.direction = Direction::Up;
            }
        }
        Direction::Down => {
            if snake_head.direction != Direction::Up {
                snake_head.direction = Direction::Down;
            }
        }
        Direction::Left => {
            if snake_head.direction != Direction::Right {
                snake_head.direction = Direction::Left;
            }
        }
        Direction::Right => {
            if snake_head.direction != Direction::Left {
                snake_head.direction = Direction::Right;
            }
        }
    }

    match snake_head.direction {
        Direction::Up => transform.translation.y += PIXEL_UNIT_SIZE,
        Direction::Down => transform.translation.y -= PIXEL_UNIT_SIZE,
        Direction::Left => transform.translation.x -= PIXEL_UNIT_SIZE,
        Direction::Right => transform.translation.x += PIXEL_UNIT_SIZE,
    }
    snake_head.position = (
        (transform.translation.x / PIXEL_UNIT_SIZE) as i32,
        (transform.translation.y / PIXEL_UNIT_SIZE) as i32,
    );

    let (apple_entity, apple) = apple_query.single();
    if snake_head.position == apple.position {
        commands.entity(apple_entity).despawn();
        apple_eaten_event.send(AppleEaten);
    }
}

fn move_snake_body(
    mut snake_head_query: Query<&mut SnakeHead>,
    mut snake_body_query: Query<(&mut SnakeBody, &mut Transform)>,
    mut last_position: ResMut<LastPosition>,
) {
    let snake_head = snake_head_query.single_mut();
    let mut prev_position = snake_head.position;
    for (mut snake_body, mut transform) in &mut snake_body_query {
        let temp = snake_body.position;
        snake_body.position = prev_position;
        prev_position = temp;
        transform.translation = Vec3::new(
            snake_body.position.0 as f32 * PIXEL_UNIT_SIZE,
            snake_body.position.1 as f32 * PIXEL_UNIT_SIZE,
            0.0,
        );
    }
    last_position.value = prev_position;
}

fn grow_snake_body(
    mut commands: Commands,
    mut apple_eaten_event: EventReader<AppleEaten>,
    last_position: Res<LastPosition>,
) {
    if apple_eaten_event.is_empty() {
        return;
    }
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE.into(),
                custom_size: Some(Vec2::new(PIXEL_UNIT_SIZE, PIXEL_UNIT_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                last_position.value.0 as f32 * PIXEL_UNIT_SIZE,
                last_position.value.1 as f32 * PIXEL_UNIT_SIZE,
                0.0,
            )),
            ..default()
        },
        SnakeBody {
            position: last_position.value,
        },
    ));
    apple_eaten_event.clear();
}

fn border_collision(mut snake_head_query: Query<&mut SnakeHead>) {
    let snake_head = snake_head_query.single_mut();
    if snake_head.position.0.abs() > PLAYFIELD.0 / 2
        || snake_head.position.1.abs() > PLAYFIELD.1 / 2
    {
        println!("Game Over!");
        std::process::exit(0);
    }
}

fn snake_body_collision(snake_head_query: Query<&SnakeHead>, snake_body_query: Query<&SnakeBody>) {
    let snake_head = snake_head_query.single();
    for snake_body in &snake_body_query {
        if snake_head.position == snake_body.position {
            println!("Game Over!");
            std::process::exit(0);
        }
    }
}

fn player_input(keyboard_input: Res<Input<KeyCode>>, mut snake_head_query: Query<&mut SnakeHead>) {
    if let Ok(mut snake_head) = snake_head_query.get_single_mut() {
        if keyboard_input.any_just_pressed([KeyCode::Up, KeyCode::W, KeyCode::I]) {
            snake_head.potential_direction = Direction::Up;
        }
        if keyboard_input.any_just_pressed([KeyCode::Down, KeyCode::S, KeyCode::K]) {
            snake_head.potential_direction = Direction::Down;
        }
        if keyboard_input.any_just_pressed([KeyCode::Left, KeyCode::A, KeyCode::J]) {
            snake_head.potential_direction = Direction::Left;
        }
        if keyboard_input.any_just_pressed([KeyCode::Right, KeyCode::D, KeyCode::L]) {
            snake_head.potential_direction = Direction::Right;
        }
    }
}
