use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

// ============================================================
// RESOURCES
// ============================================================

/// Stocke toutes les positions et types de blocs du monde
#[derive(Resource)]
struct WorldMap {
    blocks: HashMap<(isize, isize, isize), BlockType>,
}

/// Graine de génération du monde
#[derive(Resource)]
struct WorldSeed(u64);

/// Flag indiquant si le timestep fixe a tourné ce frame
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct DidFixedTimestepRunThisFrame(bool);

// ============================================================
// COMPOSANTS
// ============================================================

/// Type de bloc avec sa couleur associée
#[derive(Clone, Copy)]
enum BlockType {
    Dirt,
    Wood,
    Stone,
}

impl BlockType {
    fn color(&self) -> Color {
        match self {
            BlockType::Dirt  => Color::srgb(0.5, 0.25, 0.1),
            BlockType::Wood  => Color::srgb(0.6, 0.4,  0.2),
            BlockType::Stone => Color::srgb(0.5, 0.5,  0.5),
        }
    }
}

#[derive(Component)] struct Ground;
#[derive(Component)] struct WorldBlock;
#[derive(Component)] struct MenuUI;
#[derive(Component)] struct InGameUI;
#[derive(Component)] struct SeedText;

#[derive(Component)]
enum MenuButton { Play, NewSeed, Quit }

/// Input de déplacement accumulé entre deux timesteps
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct AccumulatedInput { movement: Vec2 }

/// Vélocité du joueur dans la simulation physique
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec3);

/// Position physique réelle du joueur (séparée du Transform visuel)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PhysicalTranslation(Vec3);

/// Position physique du frame précédent (pour l'interpolation)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PreviousPhysicalTranslation(Vec3);

/// Sensibilité de la caméra (x = horizontal, y = vertical)
#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(Vec2::new(0.003, 0.002))
    }
}

// ============================================================
// ÉTATS
// ============================================================

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    InGame,
}

// ============================================================
// GÉNÉRATION DU MONDE
// ============================================================

fn generate_blocks(mut world: ResMut<WorldMap>, seed: Res<WorldSeed>) {
    world.blocks.clear();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.0);
    let size: isize = 20;

    for x in -size..size {
        for z in -size..size {
            let noise1 = ((x as f32 * 0.3).sin() * (z as f32 * 0.2).cos()) * 4.0;
            let noise2 = ((x as f32 * 0.7).cos() * (z as f32 * 0.5).sin()) * 2.0;
            let offset = rng.gen_range(-1.0..1.0_f32);
            let height = (noise1 + noise2 + offset).max(0.0) as i32 + 1;

            for y in 0..=height {
                let block = match y {
                    y if y == height => match rng.gen_range(0..3) {
                        0 => BlockType::Stone,
                        _ => BlockType::Wood,
                    },
                    y if y > height - 3 => {
                        if rng.gen_bool(0.7) { BlockType::Dirt } else { BlockType::Stone }
                    }
                    _ => {
                        if rng.gen_bool(0.1) { BlockType::Dirt } else { BlockType::Stone }
                    }
                };
                world.blocks.insert((x, y as isize, z), block);
            }
        }
    }
}

fn display_blocks(
    mut commands: Commands,
    world: Res<WorldMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // On précalcule le mesh une seule fois et on le réutilise
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for (pos, block) in &world.blocks {
        let (x, y, z) = *pos;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(materials.add(block.color())),
            Transform::from_xyz(x as f32, y as f32, z as f32),
            WorldBlock,
        ));
    }
}

// ============================================================
// SETUP SCÈNE
// ============================================================

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sol
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    // Lumière directionnelle
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// ============================================================
// JOUEUR ET CAMÉRA
// ============================================================

fn spawn_player(mut commands: Commands) {
    // Caméra FPS
    commands.spawn((Camera3d::default(), CameraSensitivity::default()));
    // Entité joueur (invisible, gère la physique)
    commands.spawn((
        Name::new("Player"),
        Transform::from_scale(Vec3::splat(0.3)),
        AccumulatedInput::default(),
        Velocity::default(),
        PhysicalTranslation::default(),
        PreviousPhysicalTranslation::default(),
    ));
}

/// Rotation de la caméra avec la souris
fn rotate_camera(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    player: Single<(&mut Transform, &CameraSensitivity), With<Camera>>,
) {
    let (mut transform, sensitivity) = player.into_inner();
    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO { return; }

    let delta_yaw   = -delta.x * sensitivity.x;
    let delta_pitch = -delta.y * sensitivity.y;
    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);

    const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
    let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw + delta_yaw, pitch, roll);
}

/// Lecture des touches WASD et calcul de la vélocité
fn accumulate_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut AccumulatedInput, &mut Velocity)>,
    camera: Single<&Transform, With<Camera>>,
) {
    const SPEED: f32 = 4.0;
    let (mut input, mut velocity) = player.into_inner();

    input.movement = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) { input.movement.y += 1.0; }
    if keyboard_input.pressed(KeyCode::KeyS) { input.movement.y -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyA) { input.movement.x -= 1.0; }
    if keyboard_input.pressed(KeyCode::KeyD) { input.movement.x += 1.0; }

    // Alignement avec la direction de la caméra
    let input_3d = Vec3::new(input.movement.x, 0.0, -input.movement.y);
    velocity.0 = (camera.rotation * input_3d).clamp_length_max(1.0) * SPEED;
}

/// Avance la simulation physique d'un timestep fixe
fn advance_physics(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut PhysicalTranslation, &mut PreviousPhysicalTranslation, &Velocity)>,
) {
    for (mut current, mut previous, velocity) in query.iter_mut() {
        previous.0 = current.0;
        current.0 += velocity.0 * fixed_time.delta_secs();
    }
}

/// Interpolation du Transform visuel entre deux timesteps physiques
fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &PhysicalTranslation, &PreviousPhysicalTranslation)>,
) {
    let alpha = fixed_time.overstep_fraction();
    for (mut transform, current, previous) in query.iter_mut() {
        transform.translation = previous.0.lerp(current.0, alpha);
    }
}

/// Synchronise la caméra sur la position interpolée du joueur
fn translate_camera(
    mut camera: Single<&mut Transform, With<Camera>>,
    player: Single<&Transform, (With<AccumulatedInput>, Without<Camera>)>,
) {
    camera.translation = player.translation;
}

fn clear_fixed_timestep_flag(mut flag: ResMut<DidFixedTimestepRunThisFrame>) {
    flag.0 = false;
}
fn set_fixed_time_step_flag(mut flag: ResMut<DidFixedTimestepRunThisFrame>) {
    flag.0 = true;
}
fn did_fixed_timestep_run_this_frame(flag: Res<DidFixedTimestepRunThisFrame>) -> bool {
    flag.0
}
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

// ============================================================
// CURSEUR
// ============================================================

fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let Some(point) = ray.plane_intersection_point(
        ground.translation(),
        InfinitePlane3d::new(ground.up()),
    ) else { return };

    gizmos.circle(
        Isometry3d::new(
            point + ground.up() * 0.01,
            Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
        ),
        0.2,
        Color::WHITE,
    );
}

// ============================================================
// UI — MENU
// ============================================================

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, action: MenuButton) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(200.0),
            height: Val::Px(60.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
        action,
        MenuUI,
    ))
    .with_children(|p: &mut ChildSpawnerCommands| {
        p.spawn((
            Text::new(label),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}

fn setup_menu(mut commands: Commands, seed: Res<WorldSeed>) {
    commands.spawn((Camera2d, MenuUI));
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(15.0),
            ..default()
        },
        MenuUI,
    ))
    .with_children(|parent: &mut ChildSpawnerCommands| {
        parent.spawn((
            Text::new(format!("Seed: {}", seed.0)),
            TextFont { font_size: 30.0, ..default() },
            TextColor(Color::WHITE),
            SeedText,
            MenuUI,
        ));
        spawn_button(parent, "Play",     MenuButton::Play);
        spawn_button(parent, "New Seed", MenuButton::NewSeed);
        spawn_button(parent, "Quit",     MenuButton::Quit);
    });
}

fn menu_interactions(
    mut interaction_query: Query
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut seed: ResMut<WorldSeed>,
    mut text_query: Query<&mut Text, With<SeedText>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => match button {
                MenuButton::Play    => { next_state.set(GameState::InGame); }
                MenuButton::NewSeed => {
                    seed.0 = rand::random::<u64>();
                    if let Ok(mut text) = text_query.single_mut() {
                        *text = Text::new(format!("Seed: {}", seed.0));
                    }
                }
                MenuButton::Quit => { next_state.set(GameState::Menu); }
            },
            Interaction::Hovered => { *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)); }
            Interaction::None    => { *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2)); }
        }
    }
}

// ============================================================
// UI — EN JEU
// ============================================================

fn setup_ingame_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InGameUI,
    ))
    .with_children(|parent: &mut ChildSpawnerCommands| {
        parent.spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
            MenuButton::Quit,
            InGameUI,
        ))
        .with_children(|p: &mut ChildSpawnerCommands| {
            p.spawn((
                Text::new("Quit"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn ingame_interactions(
    mut interaction_query: Query
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match interaction {
            Interaction::Pressed => match button {
                MenuButton::Quit => { next_state.set(GameState::Menu); }
                _ => {}
            },
            Interaction::Hovered => { *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)); }
            Interaction::None    => { *color = BackgroundColor(Color::srgb(0.8, 0.2, 0.2)); }
        }
    }
}

// ============================================================
// CLEANUP
// ============================================================

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    for entity in &query { commands.entity(entity).despawn(); }
}
fn cleanup_world(mut commands: Commands, query: Query<Entity, With<WorldBlock>>) {
    for entity in &query { commands.entity(entity).despawn(); }
}
fn cleanup_ingame_ui(mut commands: Commands, query: Query<Entity, With<InGameUI>>) {
    for entity in &query { commands.entity(entity).despawn(); }
}

// ============================================================
// MAIN
// ============================================================

/* 
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Resources
        .insert_resource(WorldMap { blocks: HashMap::new() })
        .insert_resource(WorldSeed(0))
        .init_resource::<DidFixedTimestepRunThisFrame>()
        .init_state::<GameState>()
        // Systèmes globaux
        .add_systems(Startup, spawn_player)
        .add_systems(Update, draw_cursor)
        .add_systems(PreUpdate, clear_fixed_timestep_flag)
        .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
        .add_systems(FixedUpdate, advance_physics)
        .add_systems(RunFixedMainLoop, (
            (rotate_camera, accumulate_input)
                .chain()
                .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            (
                clear_input.run_if(did_fixed_timestep_run_this_frame),
                interpolate_rendered_transform,
                translate_camera,
            )
                .chain()
                .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
        ))
        // Menu
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, menu_interactions.run_if(in_state(GameState::Menu)))
        .add_systems(OnExit(GameState::Menu), cleanup_menu)
        // En jeu
        .add_systems(OnEnter(GameState::InGame), (setup, generate_blocks, display_blocks, setup_ingame_ui).chain())
        .add_systems(Update, ingame_interactions.run_if(in_state(GameState::InGame)))
        .add_systems(OnExit(GameState::InGame), (cleanup_world, cleanup_ingame_ui))
        .run();
}
*/