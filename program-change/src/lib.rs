use nih_plug::prelude::*;
use std::sync::Arc;

struct ProgramChange {
    last_active_ch: u8,
    notes_on: u128,
    params: Arc<ProgramChangeParams>,
    snapshot: Option<ParamsSnapshot>,
}

impl Default for ProgramChange {
    fn default() -> Self {
        Self {
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

const MSB: u8 = 0;
const VOL: u8 = 7; // Volume Coarse
const LSB: u8 = 32;

impl ProgramChange {
    fn process_active(&mut self, context: &mut impl ProcessContext<Self>) {
        let new = self.params.snapshot();
        let channel = new.ch;

        self.last_active_ch = channel;

        if self.snapshot.map_or(true, |old| old != new) {
            let timing = 0;
            let channel = new.ch;

            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: VOL,
                value: new.vol as f32 / 127.0,
            });

            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: MSB,
                value: new.msb as f32 / 127.0,
            });

            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: LSB,
                value: new.lsb as f32 / 127.0,
            });

            context.send_event(NoteEvent::MidiProgramChange {
                timing: 1,
                channel,
                program: new.pc,
            });

            self.snapshot = Some(new);
        }

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
                } => {
                    if cc != MSB && cc != LSB && cc != VOL {
                        context.send_event(NoteEvent::MidiCC {
                            timing,
                            channel,
                            cc,
                            value,
                        });
                    }
                }

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

        if self.notes_on == 0 {
            while context.next_event().is_some() {}
        } else {
            let channel = self.last_active_ch;

            while let Some(event) = context.next_event() {
                match event {
                    NoteEvent::Choke {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::Choke {
                                timing,
                                voice_id,
                                channel,
                                note,
                            });
                        }
                    }

                    NoteEvent::MidiCC {
                        timing,
                        channel: _,
                        cc,
                        value,
                    } => {
                        if self.notes_on != 0 && (cc != MSB || cc != LSB || cc != VOL) {
                            context.send_event(NoteEvent::MidiCC {
                                timing,
                                channel,
                                cc,
                                value,
                            });
                        }
                    }

                    NoteEvent::MidiChannelPressure {
                        timing,
                        channel: _,
                        pressure,
                    } => {
                        if self.notes_on != 0 {
                            context.send_event(NoteEvent::MidiChannelPressure {
                                timing,
                                channel,
                                pressure,
                            });
                        }
                    }

                    NoteEvent::MidiPitchBend {
                        timing,
                        channel: _,
                        value,
                    } => {
                        if self.notes_on != 0 {
                            context.send_event(NoteEvent::MidiPitchBend {
                                timing,
                                channel,
                                value,
                            });
                        }
                    }

                    NoteEvent::MidiProgramChange { .. } => {}

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

                    NoteEvent::PolyBrightness {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        brightness,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyBrightness {
                                timing,
                                voice_id,
                                channel,
                                note,
                                brightness,
                            });
                        }
                    }

                    NoteEvent::PolyExpression {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        expression,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyExpression {
                                timing,
                                voice_id,
                                channel,
                                note,
                                expression,
                            });
                        }
                    }

                    NoteEvent::PolyPan {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        pan,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyPan {
                                timing,
                                voice_id,
                                channel,
                                note,
                                pan,
                            });
                        }
                    }

                    NoteEvent::PolyPressure {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        pressure,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyPressure {
                                timing,
                                voice_id,
                                channel,
                                note,
                                pressure,
                            });
                        }
                    }

                    NoteEvent::PolyTuning {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        tuning,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyTuning {
                                timing,
                                voice_id,
                                channel,
                                note,
                                tuning,
                            });
                        }
                    }

                    NoteEvent::PolyVibrato {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        vibrato,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyVibrato {
                                timing,
                                voice_id,
                                channel,
                                note,
                                vibrato,
                            });
                        }
                    }

                    NoteEvent::PolyVolume {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                        gain,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::PolyVolume {
                                timing,
                                voice_id,
                                channel,
                                note,
                                gain,
                            });
                        }
                    }

                    NoteEvent::VoiceTerminated {
                        timing,
                        voice_id,
                        channel: _,
                        note,
                    } => {
                        if self.is_note_on(note) {
                            context.send_event(NoteEvent::VoiceTerminated {
                                timing,
                                voice_id,
                                channel,
                                note,
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

    #[id = "channel"]
    ch: IntParam,

    #[id = "lsb"]
    lsb: IntParam,

    #[id = "msb"]
    msb: IntParam,

    #[id = "pc"]
    pc: IntParam,

    #[id = "vol"]
    vol: IntParam,
}

impl Default for ProgramChangeParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("active", true),
            ch: IntParam::new("channel", 1, IntRange::Linear { min: 1, max: 16 }),
            msb: IntParam::new("msb", 0, IntRange::Linear { min: 0, max: 127 }),
            lsb: IntParam::new("lsb", 0, IntRange::Linear { min: 0, max: 127 }),
            pc: IntParam::new("pc", 0, IntRange::Linear { min: 0, max: 127 }),
            vol: IntParam::new("vol", 100, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl ProgramChangeParams {
    fn snapshot(&self) -> ParamsSnapshot {
        ParamsSnapshot {
            ch: (self.ch.value().clamp(1, 16) - 1) as u8,
            msb: self.msb.value().clamp(0, 127) as u8,
            lsb: self.lsb.value().clamp(0, 127) as u8,
            pc: self.pc.value().clamp(0, 127) as u8,
            vol: self.vol.value().clamp(0, 127) as u8,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ParamsSnapshot {
    ch: u8,
    lsb: u8,
    msb: u8,
    pc: u8,
    vol: u8,
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
