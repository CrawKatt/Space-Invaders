use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
    Player, SpriteSize, Velocity,
};
use wasm_bindgen::prelude::wasm_bindgen;

use enemy::EnemyPlugin;
use player::PlayerPlugin;
use std::collections::HashSet;
use bevy::render::camera::ScalingMode;
use bevy::window::PrimaryWindow;
use crate::components::PlayerInvincible;

mod components;
mod enemy;
mod player;

// region:      --- Asset Constants ---

const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;

const PLAYER_SHOOT_SOUND: &str = "player_shoot.ogg";
const PLAYER_EXPLOSION_SOUND : &str = "player_explosion.ogg";

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_LASER_SPRITE: &str = "player_laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "enemy_laser_a_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LEN: usize = 16;

const SPRITE_SCALE: f32 = 0.5;

const PLAYER_RESPAWN_DELAY: f64 = 2.;
const PLAYER_INVINCIBLE_TIME: f32 = 1.5;
const ENEMY_MAX: u32 = 2;
const FORMATION_MEMBERS_MAX: u32 = 2;
const SCOREBOARD_FONT_SIZE: f32 = 40.;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);
const BACKGROUND_IMAGE : &str = "background.png";

// endregion:   --- Game Constants ---

// region:     --- Resources ---

#[derive(Resource)]
struct ExplosionSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct PlayerShootSound(Handle<AudioSource>);

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Resource)]
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}

#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct EnemyCount(u32);

#[derive(Resource)]
struct PlayerState {
    on: bool,       // jugador activo
    last_shot: f64, // -1 si no ha disparado
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }

    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
    }
}
// endregion:   --- Resources ---

// endregion:   --- Asset Constants ---

fn main() {
    run()
}

#[wasm_bindgen]
pub fn run() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(Scoreboard { score: 0 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spade Invaders!".into(),
                resizable: true,
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(PreUpdate,update_camera_system)
        .add_systems(Update, movable_system)
        .add_systems(Update, player_laser_hit_enemy_system)
        .add_systems(Update, enemy_laser_hit_player_system)
        .add_systems(Update, explosion_to_spawn_system)
        .add_systems(Update, explosion_animation_system)
        .add_systems(Update, (update_scoreboard_system, bevy::window::close_on_esc))
        .run();
}

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera
    ));
}

/// Sistema para actualizar la cámara principal
fn update_camera_system(
    mut cameras: Query<&mut OrthographicProjection, With<MainCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>
) {

    let Ok(mut camera) = cameras.get_single_mut() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    camera.scaling_mode = ScalingMode::Fixed {
        width: window.width(),
        height: window.height(),
    };

}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    query: Query<&Window, With<PrimaryWindow>>,
) {

    // Cámara principal
    let Ok(primary) = query.get_single() else {
        return;
    };

    // insertar sonido de disparo para el jugador
    let player_shoot_sound = asset_server.load(PLAYER_SHOOT_SOUND);
    commands.insert_resource(PlayerShootSound(player_shoot_sound));

    // insertar sonido de explosión
    let explosion_sound = asset_server.load(PLAYER_EXPLOSION_SOUND);
    commands.insert_resource(ExplosionSound(explosion_sound));

    // cámara del juego
    let mut camera = Camera2dBundle::default();

    // ajustar la camara al tamaño de la ventana
    camera.projection.scaling_mode = ScalingMode::Fixed {
        width: 1920.,
        height: 1080.,
    };

    // insertar la camara
    commands.spawn((camera, MainCamera));

    // insertar fondo
    commands.spawn(SpriteBundle {
        texture: asset_server.load(BACKGROUND_IMAGE),
        ..default()
    });

    // Scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            }),
    );

    let (win_w, win_h) = (primary.width(), primary.height());

    // añadir recurso WinSize
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // añadir recursos de explosiones
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(texture_atlas);

    // añadir recursos de texturas
    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn {
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    mut scoreboard: ResMut<Scoreboard>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    // iterar sobre todos los lasers del jugador
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        let laser_scale = Vec2::from(laser_tf.scale.xy());

        // iterar sobre todos los enemigos
        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            if despawned_entities.contains(&enemy_entity)
                || despawned_entities.contains(&laser_entity)
            {
                continue;
            }

            let enemy_scale = Vec2::from(enemy_tf.scale.xy());

            // si el laser colisiona con el enemigo
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );

            // si colisiona, eliminar el laser y el enemigo
            if let Some(_) = collision {
                // remover el enemigo
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;

                // remover el laser
                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);

                // iniciar la animacion de explosion
                commands
                    .spawn(ExplosionToSpawn(enemy_tf.translation.clone()));

                // aumentar la puntuación en +1 punto
                scoreboard.score += 1;
            }
        }
    }
}

fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut player_invincible_query: Query<(Entity, &mut PlayerInvincible)>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {

    for(player_entity, mut player_invincible) in player_invincible_query.iter_mut() {
        player_invincible.time_left -= time.delta_seconds();

        if player_invincible.time_left <= 0. {
            commands.entity(player_entity).remove::<PlayerInvincible>();
        }
    }

    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = Vec2::from(player_tf.scale.xy());

        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = Vec2::from(laser_tf.scale.xy());

            // si el laser colisiona con el jugador
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );

            // si colisiona, eliminar el laser y el jugador
            if let Some(_) = collision {

                // verificar si el jugador es invencible antes de realizar acciones
                if player_invincible_query.get(player_entity).is_err() {
                    // si el jugador no es invencible, realizar acciones normales
                    // remover el jugador
                    commands.entity(player_entity).despawn();
                    player_state.shot(time.elapsed_seconds_f64());

                    // remover el laser
                    commands.entity(laser_entity).despawn();

                    // iniciar la animacion de explosion
                    commands
                        .spawn(ExplosionToSpawn(player_tf.translation.clone()));

                    break;

                } else {
                    // si el jugador es invencible, solo eliminar el laser
                    commands.entity(laser_entity).despawn();
                }
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_texture: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
    sound: Res<ExplosionSound>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        // crear la entidad de explosion
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: game_texture.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            // auto-despawn al terminar de reproducir el sonido
            settings: PlaybackSettings::DESPAWN,
        });

        // despawnear la entidad de explosion_to_spawn
        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; // mover al siguiente frame
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn();
            }
        }
    }
}

// sistema de puntuación
fn update_scoreboard_system(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}