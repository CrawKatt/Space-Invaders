use bevy::prelude::Component;
use rand::{Rng, thread_rng};
use crate::{BASE_SPEED, FORMATION_MEMBERS_MAX, WinSize};

/// Component - Formación de enemigos (por enemigo)
#[derive(Clone, Component)]
pub struct Formation {
    pub start: (f32, f32),
    pub radius: (f32, f32),
    pub pivot: (f32, f32),
    pub speed: f32,
    pub angle: f32, // cambia por tiempo
}

/// Resource - Creación de formaciones
#[derive(Default)]
pub struct FormationMaker {
    current_template: Option<Formation>,
    current_members: u32,
}

/// Implementación de creación de formaciones
impl FormationMaker {
    pub fn make(&mut self, win_size: &WinSize) -> Formation {
        match (&self.current_template, self.current_members >= FORMATION_MEMBERS_MAX) {
            // si no hay plantilla, se crea una nueva
            (Some(tmpl), false) => {
                self.current_members += 1;
                tmpl.clone()
            }

            // si la primera formación o anterior esta llena, se crea una nueva
            (None, _) | (_, true) => {
                let mut rng = thread_rng();

                // computar el inicio x/y
                let w_span = win_size.w / 2. + 100.;
                let h_span = win_size.h / 2. + 100.;
                let x = if rng.gen_bool(0.5) { w_span } else { -w_span };
                let y = rng.gen_range(-h_span..h_span) as f32;
                let start = (x, y);

                // computar el pivot x/y
                let w_span = win_size.w / 4.;
                let h_span = win_size.h / 3. + 50.0;
                let pivot = (rng.gen_range(-w_span..w_span), rng.gen_range(0.0..h_span));

                // computar el radio x/y
                let radius = (rng.gen_range(80.0..150.), 100.);

                // computar el ángulo inicial
                let angle = (y - pivot.1).atan2(x - pivot.0);

                // computar la velocidad
                let speed = BASE_SPEED;

                // crear la formación
                let formation = Formation {
                    start,
                    radius,
                    pivot,
                    speed,
                    angle,
                };

                // almacenar como plantilla
                self.current_template = Some(formation.clone());
                // reiniciar el contador de miembros a 1
                self.current_members = 1;

                formation
            }
        }
    }
}