use self::formation::{Formation, FormationMaker};
use crate::components::{Enemy, FromEnemy, Laser, Movable, SpriteSize, Velocity};
use crate::{
    EnemyCount, GameTextures, WinSize, ENEMY_LASER_SIZE, ENEMY_MAX, ENEMY_SIZE, SPRITE_SCALE,
    TIME_STEP,
};

use bevy::prelude::*;
use rand::{thread_rng, Rng};
use std::f32::consts::PI;

mod formation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FormationMaker::default())
            .add_systems(Update, enemy_spawn_system)
            .add_systems(Update, enemy_fire_system.run_if(enemy_fire_criteria))
            .add_systems(Update, enemy_movement_system);
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    win_size: Res<WinSize>,
) {
    if enemy_count.0 < ENEMY_MAX {
        // obtener formación e iniciar x/y
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;

        commands
            .spawn(SpriteBundle {
                texture: game_textures.enemy.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(formation)
            .insert(SpriteSize::from(ENEMY_SIZE));

        enemy_count.0 += 1;
    }
}

fn enemy_fire_criteria() -> bool {
    thread_rng().gen_bool(1. / 60.)
}

fn enemy_fire_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for &tf in enemy_query.iter() {
        let (x, y) = (tf.translation.x, tf.translation.y);
        // añadir laser enemigo
        commands
            .spawn(SpriteBundle {
                texture: game_textures.enemy_laser.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y - 15., 1.), // eje z = profundidad, debe ser 1 para sobrepasar el background
                    rotation: Quat::from_rotation_x(PI),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                },
                ..Default::default()
            })
            .insert(Laser)
            .insert(SpriteSize::from(ENEMY_LASER_SIZE))
            .insert(FromEnemy)
            .insert(Movable { auto_despawn: true })
            .insert(Velocity { x: 0., y: -1. });
    }
}

fn enemy_movement_system(mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>) {
    for (mut transform, mut formation) in query.iter_mut() {
        // posicion inicial
        let (x_org, y_org) = (transform.translation.x, transform.translation.y);

        // distancia máxima
        let max_distance = TIME_STEP * formation.speed;

        // fixtures para que no se salgan de la pantalla
        let dir: f32 = if formation.start.0 < 0. { 1. } else { -1. };
        let (x_pivot, y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;

        // computar siguiente posición
        let angle = formation.angle
            + dir * formation.speed * TIME_STEP / (x_radius.min(y_radius) * PI / 2.);

        // computar objetivo x/y
        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        // computar distancia
        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance != 0. {
            max_distance / distance
        } else {
            0.
        };

        // computar final x/y
        let x = x_org - dx * distance_ratio;
        let x = if dx > 0. { x.max(x_dst) } else { x.min(x_dst) };
        let y = y_org - dy * distance_ratio;
        let y = if dy > 0. { y.max(y_dst) } else { y.min(y_dst) };

        // iniciar rotación del ángulo de la formación solo cuando el sprite esta cerca del borde
        if distance < max_distance * formation.speed / 20. {
            formation.angle = angle;
        }

        let translation = &mut transform.translation;
        (translation.x, translation.y) = (x, y);
    }
}
