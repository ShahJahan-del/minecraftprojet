use bevy::prelude::*;
use std::collections::HashMap;
use bevy::camera::Camera3d;
use bevy::light::AmbientLight;
use rand::prelude::*;

/*

1) Créer la Resource WorldMap = Stocke les positions et données des blocs
-> HashMap = dictionnaire (clé = coordonnées bloc ; valeur = texture bloc)

2) Créer Block Component + BlockType + propriétés = Enum BlockType → différentes textures et comportements

3) Système de “spawn des blocs” au Startup = Remplit la Resource et/ou crée les Entities
-> la génération aléatoire se fait là

4) Système de rendu = Parcourt la Resource ou les Entities et affiche les blocs

5) Plugins Bevy = caméra, lumière, meshes (formes géométriques), matériaux (texture, couleur), 

6) Système d’interaction = Casser, marcher, collecter → pattern match sur Components

6) Plugins

- WorldPlugin pour gérer la carte et spawn
- RenderPlugin pour dessiner les blocs
- PlayerPlugin pour déplacer et interagir

*/ 

// 1) Resource WorldMap

#[derive(Resource)]
struct WorldMap {
    blocks:HashMap<(isize, isize, isize), BlockType> // nom de champ toujours en minuscule
}

// 2) Component Block

#[derive(Clone, Copy)]
enum BlockType {
    Dirt,
    Wood,
    Stone
}

// 3) Fonction spawn blocs (génération aléatoire)

// Version avec les coordonnées précises

/*fn generate_blocks(mut world:ResMut<WorldMap>) { // fonction(mut instance de la structure <WorldMap>)
    world.blocks.insert((0, 0, 0), BlockType::Dirt); // accès au champ "Blocks" de "world" et spawn avec des paramètres précis (coordonnées, texture)
    world.blocks.insert((1, 0, 0), BlockType::Wood);
    world.blocks.insert((2, 0, 0), BlockType::Stone);
}
*/

// Version avec génération aléatoire

// use rand::Rng;

fn generate_blocks(mut world: ResMut<WorldMap>) {
    let mut rng = rand::thread_rng();
    let size = 20;

    for x in -size..size {
        for z in -size..size {
            let height = rng.gen_range(0..3); // hauteur aléatoire

            for y in 0..=height {
                world.blocks.insert((x, y, z), BlockType::Dirt);
            }
        }
    }
}

// 4) Fonction d'affichage des blocs

fn display_blocks(
    // Arguments
    mut commands: Commands,
    world: Res<WorldMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Corps
    for (pos, block) in &world.blocks {
        let (x, y, z) = *pos;

        let color = match block {
            BlockType::Dirt => Color::srgb(0.5, 0.25, 0.1),
            BlockType::Wood => Color::srgb(0.6, 0.4, 0.2),
            BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
        };

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(color)),
            Transform::from_xyz(x as f32, y as f32, z as f32),
        ));
    }
}

/// Fonction Setup : set up a simple 3D scene (lumière et caméra), fonction trouvée dans les exemples Bevy

// Code pour un sol circulaire

/* fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
*/

// 5) Fonctions caméra, lumière, meshes et matériaux

// Code pour un sol plat

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// 6) Système d’interaction = Bouger la caméra, Casser, marcher, collecter → pattern match sur Components ?


fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        // Calculate a ray pointing from the camera into the world based on the cursor's position.
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        // Calculate if and where the ray is hitting the ground plane.
        && let Some(point) = ray.plane_intersection_point(ground.translation(), InfinitePlane3d::new(ground.up()))
    {
        // Draw a circle just above the ground plane at that position.
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );
    }
}


fn main() {
    println!("Hello World !");
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(WorldMap {
            blocks: HashMap::new(),
        })
    .add_systems(Startup, (generate_blocks, setup, display_blocks).chain())
    .add_systems(Update, draw_cursor)
    .run();
}
