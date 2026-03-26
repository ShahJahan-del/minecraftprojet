use bevy::prelude::*;
use std::collections::HashMap;
use bevy::camera::Camera3d;
use bevy::light::AmbientLight;

/*

1) Créer la Resource WorldMap = Stocke les positions et données des blocs
-> HashMap = dictionnaire (clé = coordonnées bloc ; valeur = texture bloc)

2) Créer Block Component + BlockType + propriétés = Enum BlockType → différentes textures et comportements

3) Système de “spawn des blocs” au Startup = Remplit la Resource et/ou crée les Entities

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

// 3) Fonction spawn blocs

fn generate_blocks(mut world:ResMut<WorldMap>) { // fonction(mut instance de la structure <WorldMap>)
    world.blocks.insert((0, 0, 0), BlockType::Dirt); // accès au champ "Blocks" de "world" et spawn avec des paramètres précis 
    world.blocks.insert((1, 0, 0), BlockType::Wood);
    world.blocks.insert((2, 0, 0), BlockType::Stone);
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
fn setup(
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

// 5) Fonctions caméra, lumière, meshes et matériaux

fn main() {
    println!("Hello World !");
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(WorldMap {
            blocks: HashMap::new(),
        })
    .add_systems(Startup, (generate_blocks, setup, display_blocks).chain())
    .run();
}



