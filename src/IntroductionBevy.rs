use bevy::prelude::*;

// Test System
/* 
1) Créer un System (= une fonction)
2) Ajouter la fonction à l'App avec .add_systems(Update, nom_fonction_sans_parenthèses)
*/   
fn hello_world() {
    println!("hello world !");
}

// Test Entity + Component

/* 
1) Créer une struct pour l'Entity (pas de champs)
2) Créer une struct pour le Component (avec des champs si on veut)
3) Faire une fonction pour "créer réellement" des Entities en leur assignant un ID et des Components
4) Ajouter la fonction à l'App avec .add_systems(Startup, nom_fonction_sans_parenthèses)
*/

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

// Test Query
/*
1) Créer fonction de requête (ici : "Prendre tous les Components Name des entités qui ont aussi le Component Person, et leur dire hello")
2) Ajouter la fonction à l'app (on peut utiliser un tuple pour mettre plusieurs systems)
*/

fn greet_people(query: Query<&Name, With<Person>>) {
    for name in &query {
        println!("hello {}!", name.0);
    }
}

// Test Mutable Query
/*
1) Créer fonction de requête (ici : "Rendre mutable tous les Components Name des entités qui ont aussi le Component Person, et modifier un nom en particulier")
2) Ajouter la fonction à l'app (on peut utiliser un tuple pour mettre plusieurs systems)
*/

fn update_people(mut query: Query<&mut Name, With<Person>>) {
    for mut name in &mut query {
        if name.0 == "Elaina Proctor" {
            name.0 = "Elaina Hume".to_string();
            break; // We don't need to change any other names.
        }
    }
}

// Test lugin personnalisé
/*
1) Créer la structure pour le plugin (HelloPlugin)
1) Implémenter l'interface Plugin dans HelloPlugin
2) Ecrire la fonction avec tout le code (en mettant les Systems dans l'App)
3) Ajouter le plugin dans l'App avec add_plugins(nom_plugin)
*/
pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (hello_world, (update_people, greet_people).chain()));
    }
}

// Test Resource
/*
1) Créer la structure pour le Timer
2) Créer et écrire la fonction, en mettant time et timer dans les arguments
3) Implémenter l'interface Plugin et écrire le code avec .insert_resource(...) en précisant les valeurs des arguments
4) Ajouter le plugin dans l'App avec add_plugins(nom_plugin)
*/

#[derive(Resource)]
struct GreetTimer(Timer);

fn greet_people_periodic(time: Res<Time>, mut timer: ResMut<GreetTimer>, query: Query<&Name, With<Person>>) {
    // update our timer with the time elapsed since the last update
    // if that caused the timer to finish, we say hello to everyone
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            println!("hello {}!", name.0);
        }
    }
}

pub struct HelloPeriodicPlugin;

impl Plugin for HelloPeriodicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, add_people);
        app.add_systems(Update, (update_people, greet_people_periodic).chain());
    }
}


// Test App
/*
fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(HelloPlugin)
    .add_plugins(HelloPeriodicPlugin)
    .run();
}
*/

