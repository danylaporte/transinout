use nih_plug::{
    midi::control_change::{DAMPER_PEDAL, EXPRESSION_CONTROLLER_MSB, MODULATION_MSB},
    prelude::*,
};
use std::{mem::take, sync::Arc};

struct SeemlessSwitch {
    state: InternalState,
    params: Arc<SeemlessSwitchParams>,
}

impl Default for SeemlessSwitch {
    fn default() -> Self {
        Self {
            params: Arc::new(SeemlessSwitchParams::default()),
            state: InternalState::Off,
        }
    }
}

impl Plugin for SeemlessSwitch {
    const NAME: &'static str = "Seemless Switch";
    const VENDOR: &'static str = "Dany Laporte";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        None
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        _buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        const OFF: bool = false;
        const ON: bool = true;

        self.state = match (take(&mut self.state), self.params.active.value()) {
            (InternalState::Off, OFF) => InternalState::Off,

            (InternalState::Off, ON) => {
                let snapshot = self.params.snapshot();

                snapshot.send(None, ctx);

                InternalState::On {
                    damper: Default::default(),
                    notes: Default::default(),
                    snapshot,
                }
            }

            (
                InternalState::On {
                    damper,
                    notes,
                    snapshot,
                }
                | InternalState::SeamlessSwitch {
                    damper,
                    notes,
                    snapshot,
                },
                ON,
            ) => {
                let new = self.params.snapshot();

                new.send(Some(&snapshot), ctx);

                InternalState::On {
                    damper,
                    notes,
                    snapshot: new,
                }
            }

            (
                InternalState::On {
                    damper,
                    notes,
                    snapshot,
                }
                | InternalState::SeamlessSwitch {
                    damper,
                    notes,
                    snapshot,
                },
                OFF,
            ) => {
                if notes.is_all_off() && damper.is_off() {
                    InternalState::Off
                } else {
                    InternalState::SeamlessSwitch {
                        damper,
                        notes,
                        snapshot,
                    }
                }
            }
        };

        match &mut self.state {
            InternalState::Off => {}
            InternalState::On { damper, notes, .. } => {
                while let Some(event) = ctx.next_event() {
                    match event {
                        NoteEvent::Choke {
                            timing,
                            voice_id,
                            note,
                            ..
                        } => {
                            notes.set_off(note);
                            ctx.send_event(NoteEvent::Choke {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                            });
                        }
                        NoteEvent::MidiCC {
                            timing, cc, value, ..
                        } => {
                            if cc == DAMPER_PEDAL {
                                if value >= 0.5 {
                                    damper.set_on();
                                } else {
                                    damper.set_off();
                                }
                            }
                            ctx.send_event(NoteEvent::MidiCC {
                                timing,
                                cc,
                                channel: 0,
                                value,
                            });
                        }
                        NoteEvent::MidiChannelPressure {
                            timing, pressure, ..
                        } => {
                            ctx.send_event(NoteEvent::MidiChannelPressure {
                                timing,
                                channel: 0,
                                pressure,
                            });
                        }
                        NoteEvent::MidiPitchBend { timing, value, .. } => {
                            if self.params.allow_pitch_bend.value() {
                                ctx.send_event(NoteEvent::MidiPitchBend {
                                    timing,
                                    channel: 0,
                                    value,
                                });
                            }
                        }
                        NoteEvent::MidiProgramChange { .. } => {}
                        NoteEvent::NoteOff {
                            timing,
                            voice_id,
                            note,
                            velocity,
                            ..
                        } => {
                            notes.set_off(note);
                            ctx.send_event(NoteEvent::NoteOff {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                velocity,
                            });
                        }
                        NoteEvent::NoteOn {
                            timing,
                            voice_id,
                            note,
                            velocity,
                            ..
                        } => {
                            notes.set_on(note);
                            ctx.send_event(NoteEvent::NoteOn {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                velocity,
                            });
                        }
                        NoteEvent::PolyBrightness {
                            timing,
                            voice_id,
                            note,
                            brightness,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyBrightness {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                brightness,
                            });
                        }
                        NoteEvent::PolyExpression {
                            timing,
                            voice_id,
                            note,
                            expression,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyExpression {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                expression,
                            });
                        }
                        NoteEvent::PolyPan {
                            timing,
                            voice_id,
                            note,
                            pan,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyPan {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                pan,
                            });
                        }
                        NoteEvent::PolyPressure {
                            timing,
                            voice_id,
                            note,
                            pressure,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyPressure {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                pressure,
                            });
                        }
                        NoteEvent::PolyTuning {
                            timing,
                            voice_id,
                            note,
                            tuning,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyTuning {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                tuning,
                            });
                        }
                        NoteEvent::PolyVibrato {
                            timing,
                            voice_id,
                            note,
                            vibrato,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyVibrato {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                vibrato,
                            });
                        }
                        NoteEvent::PolyVolume {
                            timing,
                            voice_id,
                            note,
                            gain,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::PolyVolume {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                gain,
                            });
                        }
                        NoteEvent::VoiceTerminated {
                            timing,
                            voice_id,
                            note,
                            ..
                        } => {
                            ctx.send_event(NoteEvent::VoiceTerminated {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                            });
                        }
                        event => ctx.send_event(event),
                    }
                }
            }
            InternalState::SeamlessSwitch { damper, notes, .. } => {
                while let Some(event) = ctx.next_event() {
                    match event {
                        NoteEvent::Choke {
                            timing,
                            voice_id,
                            note,
                            ..
                        } => {
                            notes.set_off(note);
                            damper.set_off();

                            ctx.send_event(NoteEvent::Choke {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                            });
                        }
                        NoteEvent::MidiCC {
                            timing, cc, value, ..
                        } => {
                            if cc == DAMPER_PEDAL && value < 0.5 {
                                damper.set_off();

                                ctx.send_event(NoteEvent::MidiCC {
                                    timing,
                                    channel: 0,
                                    cc,
                                    value: 0.0,
                                });
                            }
                        }
                        NoteEvent::NoteOff {
                            timing,
                            voice_id,
                            note,
                            velocity,
                            ..
                        } => {
                            notes.set_off(note);
                            ctx.send_event(NoteEvent::NoteOff {
                                timing,
                                voice_id,
                                channel: 0,
                                note,
                                velocity,
                            });
                        }
                        _ => {}
                    }
                }

                if damper.is_off() && notes.is_all_off() {
                    self.state = InternalState::Off;
                }
            }
        }

        ProcessStatus::Normal
    }
}

#[derive(Default)]
struct NotesState(u128);

impl NotesState {
    fn is_all_off(&self) -> bool {
        self.0 == 0
    }

    fn set_off(&mut self, note: u8) {
        self.0 &= !note_mask(note);
    }

    fn set_on(&mut self, note: u8) {
        self.0 |= note_mask(note);
    }
}

#[inline]
fn note_mask(note: u8) -> u128 {
    1 << note
}

enum InternalState {
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

#[derive(Default)]
struct DamperState(bool);

impl DamperState {
    fn is_off(&self) -> bool {
        !self.0
    }

    fn set_off(&mut self) {
        self.0 = false;
    }

    fn set_on(&mut self) {
        self.0 = true;
    }
}

#[derive(Params)]
struct SeemlessSwitchParams {
    #[id = "active"]
    active: BoolParam,

    #[id = "apb"]
    allow_pitch_bend: BoolParam,

    #[id = "expr"]
    expr: IntParam,

    #[id = "mw"]
    mw: IntParam,
}

impl Default for SeemlessSwitchParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("Active", true),
            allow_pitch_bend: BoolParam::new("Allow Pitch Bend", true),
            expr: IntParam::new("Expresion", 127, IntRange::Linear { min: 0, max: 127 }),
            mw: IntParam::new("Mod Wheel", 0, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl SeemlessSwitchParams {
    fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            expr: self.expr.value().clamp(0, 127) as u8,
            mw: self.mw.value().clamp(0, 127) as u8,
        }
    }
}

struct ParamsSnapshot {
    expr: u8,
    mw: u8,
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

    fn send(
        &self,
        old: Option<&ParamsSnapshot>,
        context: &mut impl ProcessContext<SeemlessSwitch>,
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

impl ClapPlugin for SeemlessSwitch {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.seemless-switch";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Hold notes until release even if inactive, keep sound until note off.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for SeemlessSwitch {
    const VST3_CLASS_ID: [u8; 16] = *b"SeemlessSw202312";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(SeemlessSwitch);
nih_export_vst3!(SeemlessSwitch);
