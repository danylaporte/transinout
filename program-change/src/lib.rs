use nih_plug::{
    midi::control_change::{
        BANK_SELECT_LSB, BANK_SELECT_MSB, DAMPER_PEDAL, GENERAL_PURPOSE_CONTROLLER_5_MSB,
        MAIN_VOLUME_MSB, SOUND_CONTROLLER_3, SOUND_CONTROLLER_4,
    },
    prelude::*,
};
use std::{mem::take, sync::Arc};

struct ProgramChange {
    state: InternalState,
    params: Arc<ProgramChangeParams>,
}

impl Default for ProgramChange {
    fn default() -> Self {
        Self {
            params: Arc::new(ProgramChangeParams::default()),
            state: InternalState::Off,
        }
    }
}

impl Plugin for ProgramChange {
    const NAME: &'static str = "Program Change";
    const VENDOR: &'static str = "Dany Laporte";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // This plugin doesn't have any audio IO
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

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
                    notes: Default::default(),
                    snapshot,
                }
            }

            (
                InternalState::On {
                    mut notes,
                    snapshot,
                }
                | InternalState::SeamlessSwitch {
                    mut notes,
                    snapshot,
                },
                ON,
            ) => {
                let new = self.params.snapshot();

                if new.ch == snapshot.ch {
                    new.send(Some(&snapshot), ctx);
                } else {
                    notes.send_all_note_off(snapshot.ch, ctx);
                    new.send(None, ctx);

                    notes = Default::default();
                }

                InternalState::On {
                    notes,
                    snapshot: new,
                }
            }

            (
                InternalState::On { notes, snapshot }
                | InternalState::SeamlessSwitch { notes, snapshot },
                OFF,
            ) => {
                damper_off(ctx, snapshot.ch);

                if notes.is_all_off() {    
                    InternalState::Off
                } else if self.params.channel() != snapshot.ch {
                    notes.send_all_note_off(snapshot.ch, ctx);
                    InternalState::Off
                } else {
                    InternalState::SeamlessSwitch { notes, snapshot }
                }
            }
        };

        match &mut self.state {
            InternalState::Off => {}
            InternalState::On { notes, snapshot } => {
                let channel = snapshot.ch;

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
                                channel,
                                note,
                            });
                        }
                        NoteEvent::MidiCC {
                            timing, cc, value, ..
                        } => {
                            ctx.send_event(NoteEvent::MidiCC {
                                timing,
                                cc,
                                channel,
                                value,
                            });
                        }
                        NoteEvent::MidiChannelPressure {
                            timing, pressure, ..
                        } => {
                            ctx.send_event(NoteEvent::MidiChannelPressure {
                                timing,
                                channel,
                                pressure,
                            });
                        }
                        NoteEvent::MidiPitchBend { timing, value, .. } => {
                            ctx.send_event(NoteEvent::MidiPitchBend {
                                timing,
                                channel,
                                value,
                            });
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
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
                                channel,
                                note,
                            });
                        }
                        event => ctx.send_event(event),
                    }
                }
            }
            InternalState::SeamlessSwitch { notes, snapshot } => {
                let channel = snapshot.ch;

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
                                channel,
                                note,
                            });
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
                                channel,
                                note,
                                velocity,
                            });
                        }
                        _ => {}
                    }
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

    fn is_on(&self, note: u8) -> bool {
        self.0 & note_mask(note) != 0
    }

    fn set_off(&mut self, note: u8) {
        self.0 &= !note_mask(note);
    }

    fn set_on(&mut self, note: u8) {
        self.0 |= note_mask(note);
    }

    fn send_all_note_off(&self, channel: u8, ctx: &mut impl ProcessContext<ProgramChange>) {
        if self.0 != 0 {
            for note in 0..127 {
                if self.is_on(note) {
                    // send midi message
                    ctx.send_event(NoteEvent::NoteOff {
                        timing: 0,
                        voice_id: None,
                        channel,
                        note,
                        velocity: 0.0,
                    });
                }
            }
        }
    }
}

#[inline]
fn note_mask(note: u8) -> u128 {
    1 << note
}

enum InternalState {
    On {
        notes: NotesState,
        snapshot: ParamsSnapshot,
    },
    SeamlessSwitch {
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

#[derive(Params)]
struct ProgramChangeParams {
    #[id = "active"]
    active: BoolParam,

    #[id = "attack"]
    attack: IntParam,

    #[id = "channel"]
    ch: IntParam,

    #[id = "decay"]
    decay: IntParam,

    #[id = "lsb"]
    lsb: IntParam,

    #[id = "msb"]
    msb: IntParam,

    #[id = "pc"]
    pc: IntParam,

    #[id = "release"]
    release: IntParam,

    #[id = "vol"]
    vol: IntParam,
}

impl Default for ProgramChangeParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("Active", true),
            attack: IntParam::new("Attack", 64, IntRange::Linear { min: 0, max: 127 }),
            ch: IntParam::new("Channel", 1, IntRange::Linear { min: 1, max: 16 }),
            decay: IntParam::new("Decay", 64, IntRange::Linear { min: 0, max: 127 }),
            msb: IntParam::new("Bank Select MSB", 0, IntRange::Linear { min: 0, max: 127 }),
            lsb: IntParam::new("Bank Select LSB", 0, IntRange::Linear { min: 0, max: 127 }),
            pc: IntParam::new("Program Change", 0, IntRange::Linear { min: 0, max: 127 }),
            release: IntParam::new("Release", 64, IntRange::Linear { min: 0, max: 127 }),
            vol: IntParam::new("Volume", 100, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl ProgramChangeParams {
    fn channel(&self) -> u8 {
        self.ch.value().clamp(1, 16) as u8 - 1
    }

    fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            attack: self.attack.value().clamp(0, 127) as u8,
            ch: self.channel(),
            decay: self.decay.value().clamp(0, 127) as u8,
            msb: self.msb.value().clamp(0, 127) as u8,
            lsb: self.lsb.value().clamp(0, 127) as u8,
            pc: self.pc.value().clamp(0, 127) as u8,
            release: self.release.value().clamp(0, 127) as u8,
            vol: self.vol.value().clamp(0, 127) as u8,
        }
    }
}

struct ParamsSnapshot {
    attack: u8,
    ch: u8,
    decay: u8,
    lsb: u8,
    msb: u8,
    pc: u8,
    release: u8,
    vol: u8,
}

impl ParamsSnapshot {
    fn create_cc(&self, timing: u32, cc: u8, value: u8) -> NoteEvent<()> {
        NoteEvent::MidiCC {
            timing,
            channel: self.ch,
            cc,
            value: value as f32 / 127.0,
        }
    }

    fn create_pc(&self) -> NoteEvent<()> {
        NoteEvent::MidiProgramChange {
            timing: 1,
            channel: self.ch,
            program: self.pc,
        }
    }

    fn send(&self, old: Option<&ParamsSnapshot>, context: &mut impl ProcessContext<ProgramChange>) {
        let old = old.filter(|old| old.ch == self.ch);

        // we must handle bank select with program change
        if old.map_or(true, |old| {
            old.msb != self.msb || old.lsb != self.lsb || old.pc != self.pc
        }) {
            context.send_event(self.create_cc(0, BANK_SELECT_MSB, self.msb));
            context.send_event(self.create_cc(0, BANK_SELECT_LSB, self.lsb));
            context.send_event(self.create_pc());
        }

        if old.map_or(true, |old| old.attack != self.attack) {
            context.send_event(self.create_cc(2, SOUND_CONTROLLER_4, self.attack));
        }

        if old.map_or(true, |old| old.decay != self.decay) {
            context.send_event(self.create_cc(2, GENERAL_PURPOSE_CONTROLLER_5_MSB, self.decay));
        }

        if old.map_or(true, |old| old.release != self.release) {
            context.send_event(self.create_cc(2, SOUND_CONTROLLER_3, self.release));
        }

        if old.map_or(true, |old| old.vol != self.vol) {
            context.send_event(self.create_cc(2, MAIN_VOLUME_MSB, self.vol));
        }

        if old.is_none() {
            context.send_event(self.create_cc(2, DAMPER_PEDAL, 0));
        }
    }
}

fn damper_off(ctx: &mut impl ProcessContext<ProgramChange>, channel: u8) {
    ctx.send_event(
    NoteEvent::MidiCC {
        timing: 0,
        channel,
        cc: DAMPER_PEDAL,
        value: 0.0,
    })
}

impl ClapPlugin for ProgramChange {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.program-change";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Send program change when toggle active, keep sound until note off.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for ProgramChange {
    const VST3_CLASS_ID: [u8; 16] = *b"ProgramChn202312";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(ProgramChange);
nih_export_vst3!(ProgramChange);
