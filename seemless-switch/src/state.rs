use crate::params::ParamsSnapshot;

#[derive(Debug, Default, Clone, Copy)]
pub struct NotesState(pub u128);

impl NotesState {
    pub fn is_all_off(&self) -> bool {
        self.0 == 0
    }

    pub fn set_off(&mut self, note: u8) {
        self.0 &= !note_mask(note);
    }

    pub fn set_on(&mut self, note: u8) {
        self.0 |= note_mask(note);
    }
}

#[inline]
fn note_mask(note: u8) -> u128 {
    1 << note
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DamperState(bool);

impl DamperState {
    pub fn is_off(&self) -> bool {
        !self.0
    }

    pub fn set_off(&mut self) {
        self.0 = false;
    }

    pub fn set_on(&mut self) {
        self.0 = true;
    }
}

pub enum InternalState {
    On {
        damper: DamperState,
        notes: NotesState,
        snapshot: ParamsSnapshot,
    },
    SeamlessSwitch {
        damper: DamperState,
        notes: NotesState,
        snapshot: ParamsSnapshot,
    },
    Off,
}

impl Default for InternalState {
    fn default() -> Self {
        Self::Off
    }
}
