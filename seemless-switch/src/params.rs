use nih_plug::midi::control_change::{EXPRESSION_CONTROLLER_MSB, MODULATION_MSB};
use nih_plug::prelude::*;

use crate::SeemlessSwitch;

#[derive(Params)]
pub struct SeemlessSwitchParams {
    #[id = "active"]
    pub active: BoolParam,

    #[id = "amw"]
    pub allow_mod_wheel: BoolParam,

    #[id = "apb"]
    pub allow_pitch_bend: BoolParam,

    #[id = "adp"]
    pub allow_sustain: BoolParam,

    #[id = "expr"]
    pub expr: IntParam,

    #[id = "mw"]
    pub mw: IntParam,

    #[id = "slider1"]
    pub slider1: IntParam,

    #[id = "slider2"]
    pub slider2: IntParam,

    #[id = "slider3"]
    pub slider3: IntParam,

    #[id = "slider4"]
    pub slider4: IntParam,

    #[id = "slider5"]
    pub slider5: IntParam,

    #[id = "slider6"]
    pub slider6: IntParam,

    #[id = "slider7"]
    pub slider7: IntParam,

    #[id = "slider8"]
    pub slider8: IntParam,

    #[id = "knob1"]
    pub knob1: IntParam,

    #[id = "knob2"]
    pub knob2: IntParam,

    #[id = "knob3"]
    pub knob3: IntParam,

    #[id = "knob4"]
    pub knob4: IntParam,

    #[id = "knob5"]
    pub knob5: IntParam,

    #[id = "knob6"]
    pub knob6: IntParam,

    #[id = "knob7"]
    pub knob7: IntParam,

    #[id = "knob8"]
    pub knob8: IntParam,
}

impl Default for SeemlessSwitchParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("Active", true),
            allow_mod_wheel: BoolParam::new("Allow Mod Wheel", true),
            allow_pitch_bend: BoolParam::new("Allow Pitch Bend", true),
            allow_sustain: BoolParam::new("Allow Sustain", true),
            expr: IntParam::new("Expression", -1, IntRange::Linear { min: -1, max: 127 }),
            mw: IntParam::new("Mod Wheel", -1, IntRange::Linear { min: -1, max: 127 }),
            slider1: IntParam::new("Slider 1 (82)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider2: IntParam::new("Slider 2 (83)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider3: IntParam::new("Slider 3 (84)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider4: IntParam::new("Slider 4 (85)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider5: IntParam::new("Slider 5 (86)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider6: IntParam::new("Slider 6 (87)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider7: IntParam::new("Slider 7 (88)", -1, IntRange::Linear { min: -1, max: 127 }),
            slider8: IntParam::new("Slider 8 (89)", -1, IntRange::Linear { min: -1, max: 127 }),

            knob1: IntParam::new("Knob 1 (16)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob2: IntParam::new("Knob 2 (17)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob3: IntParam::new("Knob 3 (18)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob4: IntParam::new("Knob 4 (19)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob5: IntParam::new("Knob 5 (20)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob6: IntParam::new("Knob 6 (21)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob7: IntParam::new("Knob 7 (22)", -1, IntRange::Linear { min: -1, max: 127 }),
            knob8: IntParam::new("Knob 8 (23)", -1, IntRange::Linear { min: -1, max: 127 }),
        }
    }
}

impl SeemlessSwitchParams {
    pub fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            expr: set_value_range_opt(&self.expr),
            mw: set_value_range_opt(&self.mw),

            knob1: set_value_range_opt(&self.knob1),
            knob2: set_value_range_opt(&self.knob2),
            knob3: set_value_range_opt(&self.knob3),
            knob4: set_value_range_opt(&self.knob4),
            knob5: set_value_range_opt(&self.knob5),
            knob6: set_value_range_opt(&self.knob6),
            knob7: set_value_range_opt(&self.knob7),
            knob8: set_value_range_opt(&self.knob8),

            slider1: set_value_range_opt(&self.slider1),
            slider2: set_value_range_opt(&self.slider2),
            slider3: set_value_range_opt(&self.slider3),
            slider4: set_value_range_opt(&self.slider4),
            slider5: set_value_range_opt(&self.slider5),
            slider6: set_value_range_opt(&self.slider6),
            slider7: set_value_range_opt(&self.slider7),
            slider8: set_value_range_opt(&self.slider8),
        }
    }
}

fn set_value_range_opt(v: &IntParam) -> Option<u8> {
    let v = v.value();

    if v == -1 {
        None
    } else {
        Some(v.clamp(0, 127) as u8)
    }
}

pub struct ParamsSnapshot {
    pub expr: Option<u8>,
    pub mw: Option<u8>,

    pub knob1: Option<u8>,
    pub knob2: Option<u8>,
    pub knob3: Option<u8>,
    pub knob4: Option<u8>,
    pub knob5: Option<u8>,
    pub knob6: Option<u8>,
    pub knob7: Option<u8>,
    pub knob8: Option<u8>,

    pub slider1: Option<u8>,
    pub slider2: Option<u8>,
    pub slider3: Option<u8>,
    pub slider4: Option<u8>,
    pub slider5: Option<u8>,
    pub slider6: Option<u8>,
    pub slider7: Option<u8>,
    pub slider8: Option<u8>,
}

impl ParamsSnapshot {
    pub fn send(
        &self,
        old: Option<&ParamsSnapshot>,
        context: &mut impl ProcessContext<crate::SeemlessSwitch>,
    ) {
        send_cc(old, self, |p| p.expr, EXPRESSION_CONTROLLER_MSB, context);
        send_cc(old, self, |p| p.mw, MODULATION_MSB, context);
        send_cc(old, self, |p| p.knob1, 16, context);
        send_cc(old, self, |p| p.knob2, 17, context);
        send_cc(old, self, |p| p.knob3, 18, context);
        send_cc(old, self, |p| p.knob4, 19, context);
        send_cc(old, self, |p| p.knob5, 20, context);
        send_cc(old, self, |p| p.knob6, 21, context);
        send_cc(old, self, |p| p.knob7, 22, context);
        send_cc(old, self, |p| p.knob8, 23, context);
        send_cc(old, self, |p| p.slider1, 82, context);
        send_cc(old, self, |p| p.slider2, 83, context);
        send_cc(old, self, |p| p.slider3, 84, context);
        send_cc(old, self, |p| p.slider4, 85, context);
        send_cc(old, self, |p| p.slider5, 86, context);
        send_cc(old, self, |p| p.slider6, 87, context);
        send_cc(old, self, |p| p.slider7, 88, context);
        send_cc(old, self, |p| p.slider8, 89, context);
    }
}

fn send_cc(
    old: Option<&ParamsSnapshot>,
    new: &ParamsSnapshot,
    map: impl Fn(&ParamsSnapshot) -> Option<u8> + Copy,
    controller: u8,
    context: &mut impl ProcessContext<SeemlessSwitch>,
) {
    if let Some(new) = map(new)
        && old.and_then(map).is_none_or(|old| old != new)
    {
        context.send_event(NoteEvent::MidiCC {
            timing: 0,
            cc: controller,
            channel: 0,
            value: new as f32 / 127.0,
        });
    }
}
