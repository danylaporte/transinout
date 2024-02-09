use nih_plug::{
    midi::control_change::{
        BANK_SELECT_LSB, BANK_SELECT_MSB, DAMPER_PEDAL, GENERAL_PURPOSE_CONTROLLER_5_MSB,
        MAIN_VOLUME_MSB, SOUND_CONTROLLER_3, SOUND_CONTROLLER_4,
    },
    prelude::*,
};
use std::sync::Arc;

struct ProgramChange {
    /// damper pedal active
    damper: bool,
    last_active_ch: u8,
    notes_on: u128,
    params: Arc<ProgramChangeParams>,
    snapshot: Option<ParamsSnapshot>,
}

impl Default for ProgramChange {
    fn default() -> Self {
        Self {
            damper: false,
            last_active_ch: 0,
            notes_on: 0,
            params: Arc::new(ProgramChangeParams::default()),
            snapshot: None,
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
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.params.active.value() {
            self.process_active(context);
        } else {
            self.process_inactive(context);
        }

        ProcessStatus::Normal
    }
}

impl ProgramChange {
    fn process_active(&mut self, context: &mut impl ProcessContext<Self>) {
        let new_snapshot = self.params.snapshot();
        let channel = new_snapshot.ch;

        self.last_active_ch = channel;

        new_snapshot.send(self.snapshot.as_ref(), context);

        self.snapshot = Some(new_snapshot);

        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::Choke {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                } => {
                    self.make_note_off(note);

                    context.send_event(NoteEvent::Choke {
                        timing,
                        voice_id,
                        channel,
                        note,
                    });
                }

                NoteEvent::MidiCC {
                    timing,
                    channel: _,
                    cc,
                    value,
                } => match cc {
                    BANK_SELECT_LSB | BANK_SELECT_MSB => {}
                    _ => {
                        context.send_event(NoteEvent::MidiCC {
                            timing,
                            channel,
                            cc,
                            value,
                        });

                        if cc == DAMPER_PEDAL {
                            self.damper = value >= 0.5;
                        }
                    }
                },

                NoteEvent::MidiChannelPressure {
                    timing,
                    channel: _,
                    pressure,
                } => context.send_event(NoteEvent::MidiChannelPressure {
                    timing,
                    channel,
                    pressure,
                }),

                NoteEvent::MidiPitchBend {
                    timing,
                    channel: _,
                    value,
                } => context.send_event(NoteEvent::MidiPitchBend {
                    timing,
                    channel,
                    value,
                }),

                NoteEvent::MidiProgramChange { .. } => {}

                NoteEvent::MonoAutomation {
                    timing,
                    poly_modulation_id,
                    normalized_value,
                } => context.send_event(NoteEvent::MonoAutomation {
                    timing,
                    poly_modulation_id,
                    normalized_value,
                }),

                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    velocity,
                } => {
                    self.make_note_off(note);
                    context.send_event(NoteEvent::NoteOff {
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
                    channel: _,
                    note,
                    velocity,
                } => {
                    self.make_note_on(note);
                    context.send_event(NoteEvent::NoteOn {
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
                    channel: _,
                    note,
                    brightness,
                } => context.send_event(NoteEvent::PolyBrightness {
                    timing,
                    voice_id,
                    channel,
                    note,
                    brightness,
                }),

                NoteEvent::PolyExpression {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    expression,
                } => context.send_event(NoteEvent::PolyExpression {
                    timing,
                    voice_id,
                    channel,
                    note,
                    expression,
                }),

                NoteEvent::PolyModulation {
                    timing,
                    voice_id,
                    poly_modulation_id,
                    normalized_offset,
                } => context.send_event(NoteEvent::PolyModulation {
                    timing,
                    voice_id,
                    poly_modulation_id,
                    normalized_offset,
                }),

                NoteEvent::PolyPan {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    pan,
                } => context.send_event(NoteEvent::PolyPan {
                    timing,
                    voice_id,
                    channel,
                    note,
                    pan,
                }),

                NoteEvent::PolyPressure {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    pressure,
                } => context.send_event(NoteEvent::PolyPressure {
                    timing,
                    voice_id,
                    channel,
                    note,
                    pressure,
                }),

                NoteEvent::PolyTuning {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    tuning,
                } => context.send_event(NoteEvent::PolyTuning {
                    timing,
                    voice_id,
                    channel,
                    note,
                    tuning,
                }),

                NoteEvent::PolyVibrato {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    vibrato,
                } => context.send_event(NoteEvent::PolyVibrato {
                    timing,
                    voice_id,
                    channel,
                    note,
                    vibrato,
                }),

                NoteEvent::PolyVolume {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                    gain,
                } => context.send_event(NoteEvent::PolyVolume {
                    timing,
                    voice_id,
                    channel,
                    note,
                    gain,
                }),

                NoteEvent::VoiceTerminated {
                    timing,
                    voice_id,
                    channel: _,
                    note,
                } => {
                    self.make_note_off(note);
                    context.send_event(NoteEvent::VoiceTerminated {
                        timing,
                        voice_id,
                        channel,
                        note,
                    });
                }

                e => context.send_event(e),
            }
        }
    }

    fn process_inactive(&mut self, context: &mut impl ProcessContext<Self>) {
        self.snapshot = None;

        if self.damper || self.notes_on != 0 {
            let channel = self.last_active_ch;

            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::Choke {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                    } => {
                        self.make_note_off(note);

                        context.send_event(NoteEvent::Choke {
                            timing,
                            voice_id,
                            channel,
                            note,
                        });
                    }

                    NoteEvent::MidiCC {
                        timing,
                        channel: _,
                        cc,
                        value,
                    } => match cc {
                        BANK_SELECT_LSB | BANK_SELECT_MSB => {}
                        _ => {
                            if cc == DAMPER_PEDAL && value < 0.5 && self.damper {
                                self.damper = false;

                                context.send_event(NoteEvent::MidiCC {
                                    timing,
                                    channel,
                                    cc,
                                    value,
                                });
                            }
                        }
                    },

                    NoteEvent::NoteOff {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        velocity,
                    } => {
                        if self.is_note_on(note) {
                            self.make_note_off(note);
                            context.send_event(NoteEvent::NoteOff {
                                timing,
                                voice_id,
                                channel,
                                note,
                                velocity,
                            });
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    fn is_note_on(&mut self, note: u8) -> bool {
        (self.notes_on & (1 << note)) != 0
    }

    fn make_note_on(&mut self, note: u8) {
        let mask = 1 << note;
        self.notes_on |= mask;
    }

    fn make_note_off(&mut self, note: u8) {
        let mask = 1 << note;
        self.notes_on &= !mask;
    }
}

#[derive(Params)]
struct ProgramChangeParams {
    /// when active, the plugin will transmit midi notes, if inactive, only note ends will be transmited and then terminated.
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
            active: BoolParam::new("active", true),
            attack: IntParam::new("attack", 64, IntRange::Linear { min: 0, max: 127 }),
            ch: IntParam::new("channel", 1, IntRange::Linear { min: 1, max: 16 }),
            decay: IntParam::new("decay", 64, IntRange::Linear { min: 0, max: 127 }),
            msb: IntParam::new("msb", 0, IntRange::Linear { min: 0, max: 127 }),
            lsb: IntParam::new("lsb", 0, IntRange::Linear { min: 0, max: 127 }),
            pc: IntParam::new("pc", 0, IntRange::Linear { min: 0, max: 127 }),
            release: IntParam::new("release", 64, IntRange::Linear { min: 0, max: 127 }),
            vol: IntParam::new("vol", 100, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl ProgramChangeParams {
    fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            attack: self.attack.value().clamp(0, 127) as u8,
            ch: (self.ch.value().clamp(1, 16) - 1) as u8,
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
