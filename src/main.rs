use std::f32::consts::PI;

use bevy::{
    window::{WindowMode, PrimaryWindow},
    input::{mouse::MouseButtonInput, ButtonState, common_conditions::*},
    audio::AudioSource,
    asset::Handle,
    prelude::*,
};


const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

const BUBBLES_NUMBER: i32 = 10;
const BUBBLE_MAX_VELOCITY: f32 = 100.0;
const BUBBLE_MIN_VELOCITY: f32 = -100.0;
//const BUBBLE_MASS_DIVIDER: f32 = 10.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Fullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_mouse,
            spawn_bubble.run_if(run_if_more_bubbles_possible),
            move_bubbles,
            check_collision,
            reset_game.run_if(run_if_no_bubbles),
            exit.run_if(input_just_pressed(KeyCode::Escape))
        ))
        .run();
}

#[derive(Component)]
struct Bubble {
    radius: f32,
    velocity: Vec3,
    mass: f32,
}
#[derive(Component)]
struct BubblesToSpawnCounter {
    count: i32,
}

#[derive(Resource, Deref)]
struct ClickSound(Handle<AudioSource>);

fn setup(
    
    mut commands: Commands,
    asset_server: Res<AssetServer>,
 ) {
    commands.spawn(Camera2d);
    
    // Sound
    let pop_sound = asset_server.load("sounds/pop.ogg");
    commands.insert_resource(ClickSound(pop_sound));
    
    commands.spawn(BubblesToSpawnCounter {
        count: BUBBLES_NUMBER,
    });
}
fn spawn_bubble(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_bubbles: Query<&Transform, With<Bubble>>,
    mut q_bubbles_to_spawn_counter: Query<&mut BubblesToSpawnCounter>,
) {
    let color = Color::srgb(
        rand::random_range(0.0 .. 1.0), 
        rand::random_range(0.0 .. 1.0), 
        rand::random_range(0.0 .. 1.0)
    );
    let position = Vec3::new(
        rand::random_range(-q_windows.single().width()/2.0 .. q_windows.single().width()/2.0),
        rand::random_range(-q_windows.single().height()/2.0 .. q_windows.single().height()/2.0),
        0.0,
    );
    let diameter = rand::random_range(100.0 .. 300.0);
    let mass = diameter * diameter * PI / 4.0;
    let velocity = Vec3::new(
        rand::random_range(BUBBLE_MIN_VELOCITY .. BUBBLE_MAX_VELOCITY),
        rand::random_range(BUBBLE_MIN_VELOCITY .. BUBBLE_MAX_VELOCITY),
        0.0,
    );
    
    let mut intersections = 0;
    // Check if the new bubble intersects with any existing bubbles
    // If it does, skip spawning it
    for bubble in q_bubbles.iter() {
        if bubble.translation.distance(position) < diameter/2.0 + bubble.scale.x/2.0 {
            intersections += 1;
            break;
        }
    }
    if intersections == 0 {
        commands.spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position)
            .with_scale(Vec2::splat(diameter).extend(1.)),
            Bubble {
                radius: diameter / 2.,
                velocity,
                mass,
            },
        ));
        // Update the bubble counter
        for mut bubbles_to_spawn_counter in q_bubbles_to_spawn_counter.iter_mut() {
            bubbles_to_spawn_counter.count -= 1;
            if bubbles_to_spawn_counter.count <= 0 {
                bubbles_to_spawn_counter.count = 0;
            }
        }

    }
}

fn move_bubbles(
    time: Res<Time>,
    mut q_bubbles: Query<(&mut Bubble, &mut Transform)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    for (mut bubble, mut transform) in q_bubbles.iter_mut() {
        transform.translation += bubble.velocity * time.delta_secs();
        // Check if the bubble is outside the window
        if transform.translation.x < -q_windows.single().width()/2.0 || transform.translation.x > q_windows.single().width()/2.0 {
            bubble.velocity.x = -bubble.velocity.x;
        }
        if transform.translation.y < -q_windows.single().height()/2.0 || transform.translation.y > q_windows.single().height()/2.0 {
            bubble.velocity.y = -bubble.velocity.y;
        }
    }
}

fn check_collision(
    mut q_bubbles: Query<(&mut Bubble, &mut Transform)>,
) {
    let mut combinations = q_bubbles.iter_combinations_mut();
    while let Some([(mut bubble, mut transform), (mut other_bubble, mut other_transform)]) = combinations.fetch_next() {
        if transform.translation.distance(other_transform.translation) < bubble.radius + other_bubble.radius {
            // Handle collision
            // Update velocities based on mass
            let total_mass = bubble.mass + other_bubble.mass;
            let velocity1 = bubble.velocity * ( bubble.mass - other_bubble.mass) / total_mass +
                other_bubble.velocity * (2.0 * other_bubble.mass) / total_mass;
            let velocity2 = other_bubble.velocity * (other_bubble.mass - bubble.mass) / total_mass +    
                bubble.velocity * (2.0 * bubble.mass) / total_mass;
            bubble.velocity = velocity1;
            other_bubble.velocity = velocity2;
            // Update positions to avoid sticking
            let overlap = bubble.radius + other_bubble.radius - transform.translation.distance(other_transform.translation);
            let direction = (other_transform.translation - transform.translation).normalize();
            transform.translation -= direction * overlap / 2.0;
            other_transform.translation += direction * overlap / 2.0;
        }
    }
}

fn reset_game (
    mut q_bubbles_to_spawn_counter: Query<&mut BubblesToSpawnCounter>
){

    q_bubbles_to_spawn_counter.single_mut().count = BUBBLES_NUMBER;

}
fn handle_mouse(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_t_bubbles: Query<(&Transform, Entity, Option<&Bubble>)>,
    mut commands: Commands,
    camera: Query<&Transform, With<Camera>>,
    sound: Res<ClickSound>,
) {
    for event in mouse_button_input_events.read() {
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state: ButtonState::Pressed,
                window: _,
            } => {
                // Check if the cursor is inside the game window
                if let Some(position) = q_windows.single().cursor_position() {
                    let cursor_position_world = window_to_world(q_windows.single(), camera.single(), &position);
                                       
                    for (transform, entity_id, bubble) in q_t_bubbles.iter() {
                        if let Some(bubble) = bubble {
                            // When cursor is inside bubble coordinates:
                            if cursor_position_world.distance(transform.translation) < bubble.radius {
                                // Delete bubble:
                                commands.entity(entity_id).despawn();
                                // Play pop sound:
                                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
                                
                                
                            }
                        }
                    }
                } else {
                    println!("Cursor is not in the game window.");
                }
                
            }
            _ => {}
        }
    }
}
fn run_if_no_bubbles(
    q_bubbles: Query<&Bubble>,
) -> bool {
    q_bubbles.iter().count() == 0
}
fn run_if_more_bubbles_possible(
    q_bubbles_to_spawn_counter: Query<&BubblesToSpawnCounter>
) -> bool {
    let bubbles_to_spawn_counter = q_bubbles_to_spawn_counter.single();
    bubbles_to_spawn_counter.count as usize > 0
}

fn window_to_world(
    window: &Window,
    camera: &Transform,
    position: &Vec2,
) -> Vec3 {
    let center = camera.translation.truncate();
    let half_width = (window.width() / 2.0) * camera.scale.x;
    let half_height = (window.height() / 2.0) * camera.scale.y;
    let left = center.x - half_width;
    let bottom = center.y - half_height;
    Vec3::new(
        left + position.x * camera.scale.x,
        -(bottom + position.y * camera.scale.y),
        0.0,  // I'm working in 2D
    )
}
fn exit() {
    std::process::exit(0);
}