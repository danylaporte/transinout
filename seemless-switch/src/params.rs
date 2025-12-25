use nih_plug::midi::control_change::{DAMPER_PEDAL, EXPRESSION_CONTROLLER_MSB, MODULATION_MSB};
use nih_plug::prelude::*;

#[derive(Params)]
pub struct SeemlessSwitchParams {
    #[id = "active"]
    pub active: BoolParam,

    #[id = "apb"]
    pub allow_pitch_bend: BoolParam,

    #[id = "expr"]
    pub expr: IntParam,

    #[id = "mw"]
    pub mw: IntParam,
}

impl Default for SeemlessSwitchParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("Active", true),
            allow_pitch_bend: BoolParam::new("Allow Pitch Bend", true),
            expr: IntParam::new("Expression", 127, IntRange::Linear { min: 0, max: 127 }),
            mw: IntParam::new("Mod Wheel", 0, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl SeemlessSwitchParams {
    pub fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            expr: self.expr.value().clamp(0, 127) as u8,
            mw: self.mw.value().clamp(0, 127) as u8,
        }
    }
}

pub struct ParamsSnapshot {
    pub expr: u8,
    pub mw: u8,
}

impl ParamsSnapshot {
    fn create_cc(&self, timing: u32, cc: u8, value: u8) -> NoteEvent<()> {
        NoteEvent::MidiCC {
            timing,
            channel: 0,
            cc,
            value: value as f32 / 127.0,
        }
    }

    pub fn send(
        &self,
        old: Option<&ParamsSnapshot>,
        context: &mut impl ProcessContext<crate::SeemlessSwitch>,
    ) {
        if old.map_or(true, |old| old.expr != self.expr) {
            context.send_event(self.create_cc(2, EXPRESSION_CONTROLLER_MSB, self.expr));
        }

        if old.map_or(true, |old| old.mw != self.mw) {
            context.send_event(self.create_cc(2, MODULATION_MSB, self.mw));
        }

        if old.is_none() {
            context.send_event(self.create_cc(2, DAMPER_PEDAL, 0));
        }
    }
}
