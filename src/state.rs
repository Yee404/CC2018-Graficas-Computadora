// state.rs
// Estado general: la ubicación activa se mantiene separada de la etapa
// narrativa. En la Fase 1 solo se usan la ubicación, los tiempos y el
// cambio temporal de ubicación; el resto queda preparado para fases futuras.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    Stronghold,
    End,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceState {
    ExploringStronghold,
    TeleportingToEnd,
    DragonDeath,
    EndPortalOpen,
    TeleportingToCredits,
    Credits,
}

#[allow(dead_code)]
pub struct GameState {
    pub location: Location,
    pub sequence: SequenceState,
    pub stage_time: f32,     // segundos desde que empezó la etapa actual
    pub global_time: f32,    // segundos desde el inicio (animaciones)
    pub fade: f32,           // 0 = sin fundido, 1 = negro total
    pub credits_offset: f32, // desplazamiento vertical de los créditos
    pub controls_locked: bool,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            location: Location::Stronghold,
            sequence: SequenceState::ExploringStronghold,
            stage_time: 0.0,
            global_time: 0.0,
            fade: 0.0,
            credits_offset: 0.0,
            controls_locked: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.stage_time += dt;
        self.global_time += dt;
    }

    pub fn set_stage(&mut self, sequence: SequenceState) {
        self.sequence = sequence;
        self.stage_time = 0.0;
    }

    /// Cambio temporal de ubicación (Fase 1). Será reemplazado por los
    /// triggers físicos de los portales en la Fase 2.
    pub fn debug_switch_location(&mut self, location: Location) {
        self.location = location;
        self.set_stage(match location {
            Location::Stronghold => SequenceState::ExploringStronghold,
            // En la Fase 1 el dragón permanece estático en esta etapa.
            Location::End => SequenceState::DragonDeath,
        });
    }

    pub fn dragon_visible(&self) -> bool {
        !matches!(
            self.sequence,
            SequenceState::EndPortalOpen
                | SequenceState::TeleportingToCredits
                | SequenceState::Credits
        )
    }

    pub fn exit_portal_active(&self) -> bool {
        matches!(
            self.sequence,
            SequenceState::EndPortalOpen
                | SequenceState::TeleportingToCredits
                | SequenceState::Credits
        )
    }
}
