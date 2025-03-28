use bevy::{
    input::{mouse::MouseButtonInput, ButtonState}, prelude::*, window::PrimaryWindow, audio::AudioSource,
    asset::Handle,
};


const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

const BUBBLES_NUMBER: u32 = 115;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_mouse,
            spawn_bubbles.run_if(run_if_no_bubbles),
        ))
        .run();
}

#[derive(Component)]
struct Bubble {
    radius: f32,
}

#[derive(Resource, Deref)]
struct ClickSound(Handle<AudioSource>);

fn setup(
    
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    
    // Sound
    let pop_sound = asset_server.load("sounds/pop.ogg");
    commands.insert_resource(ClickSound(pop_sound));

    spawn_bubbles(commands, q_windows, meshes, materials);
        
}
fn spawn_bubbles(
    mut commands: Commands,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Bubbles:
    for _ in 0..BUBBLES_NUMBER {
        let color = Color::srgb(
            rand::random_range(0.0 .. 1.0), 
            rand::random_range(0.0 .. 1.0), 
            rand::random_range(0.0 .. 1.0)
        );
        let position = Vec3::new(
            rand::random_range(-q_windows.single().width()/2.0 .. q_windows.single().width()/2.0),
            rand::random_range(-q_windows.single().height()/2.0 .. q_windows.single().height()/2.0),
            // hack: to not overlap colors:
            rand::random_range(0.0 .. 1.0),
        );
        let diameter = rand::random_range(100.0 .. 300.0);
        commands.spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(position)
            .with_scale(Vec2::splat(diameter).extend(1.)),
            Bubble {
                radius: diameter / 2.,
            },
        ));
    }
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
    q_bubbles: Query<&Transform, With<Bubble>>,
) -> bool {
    q_bubbles.iter().count() == 0
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