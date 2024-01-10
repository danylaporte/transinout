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

impl ProgramChange {
    fn process_active(&mut self, context: &mut impl ProcessContext<Self>) {
        let new = self.params.snapshot();
        let ch = new.ch;

        self.last_active_ch = ch;

        if self.snapshot.map_or(true, |old| old != new) {
            let timing = 0;
            let channel = new.ch;

            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: 0,
                value: new.msb as f32 / 127.0,
            });

            context.send_event(NoteEvent::MidiCC {
                timing,
                channel,
                cc: 32,
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
                event @ NoteEvent::Choke { channel, note, .. }
                | event @ NoteEvent::NoteOff { channel, note, .. }
                | event @ NoteEvent::VoiceTerminated { channel, note, .. }
                    if channel == ch =>
                {
                    self.make_note_off(note);
                    context.send_event(event);
                }

                event @ NoteEvent::MidiCC { channel, .. }
                | event @ NoteEvent::MidiChannelPressure { channel, .. }
                | event @ NoteEvent::MidiPitchBend { channel, .. }
                | event @ NoteEvent::MidiProgramChange { channel, .. }
                | event @ NoteEvent::PolyBrightness { channel, .. }
                | event @ NoteEvent::PolyExpression { channel, .. }
                | event @ NoteEvent::PolyPan { channel, .. }
                | event @ NoteEvent::PolyPressure { channel, .. }
                | event @ NoteEvent::PolyTuning { channel, .. }
                | event @ NoteEvent::PolyVibrato { channel, .. }
                | event @ NoteEvent::PolyVolume { channel, .. }
                    if channel == ch =>
                {
                    context.send_event(event);
                }

                event @ NoteEvent::NoteOn {
                    channel,
                    note,
                    velocity,
                    ..
                } if channel == ch && velocity > 0.0 => {
                    self.make_note_on(note);
                    context.send_event(event);
                }

                e => context.send_event(e),
            }
        }
    }

    fn process_inactive(&mut self, context: &mut impl ProcessContext<Self>) {
        if self.notes_on != 0 {
            let ch = self.last_active_ch;

            while let Some(event) = context.next_event() {
                match event {
                    event @ NoteEvent::Choke { channel, note, .. }
                    | event @ NoteEvent::NoteOff { channel, note, .. }
                    | event @ NoteEvent::VoiceTerminated { channel, note, .. }
                        if channel == ch =>
                    {
                        if self.is_note_on(note) {
                            self.make_note_off(note);
                            context.send_event(event);

                            if self.notes_on == 0 {
                                break;
                            }
                        }
                    }

                    event @ NoteEvent::MidiCC { channel, .. }
                    | event @ NoteEvent::MidiChannelPressure { channel, .. }
                    | event @ NoteEvent::MidiPitchBend { channel, .. }
                    | event @ NoteEvent::MidiProgramChange { channel, .. }
                    | event @ NoteEvent::PolyBrightness { channel, .. }
                    | event @ NoteEvent::PolyExpression { channel, .. }
                    | event @ NoteEvent::PolyPan { channel, .. }
                    | event @ NoteEvent::PolyPressure { channel, .. }
                    | event @ NoteEvent::PolyTuning { channel, .. }
                    | event @ NoteEvent::PolyVibrato { channel, .. }
                    | event @ NoteEvent::PolyVolume { channel, .. }
                        if channel == ch =>
                    {
                        context.send_event(event);
                    }

                    _ => {}
                }
            }
        }

        self.snapshot = None;
    }

    fn is_note_on(&mut self, note: u8) -> bool {
        (self.notes_on & (1 << note)) != 0
    }

    fn make_note_on(&mut self, note: u8) {
        self.notes_on |= 1 << note;
    }

    fn make_note_off(&mut self, note: u8) {
        self.notes_on &= !(1 << note);
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
}

impl Default for ProgramChangeParams {
    fn default() -> Self {
        Self {
            active: BoolParam::new("active", true),
            ch: IntParam::new("channel", 1, IntRange::Linear { min: 1, max: 16 }),
            msb: IntParam::new("msb", 0, IntRange::Linear { min: 0, max: 127 }),
            lsb: IntParam::new("lsb", 0, IntRange::Linear { min: 0, max: 127 }),
            pc: IntParam::new("pc", 0, IntRange::Linear { min: 0, max: 127 }),
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
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct ParamsSnapshot {
    ch: u8,
    lsb: u8,
    msb: u8,
    pc: u8,
}

impl ClapPlugin for ProgramChange {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.program-change";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Transpose before and after midi effect");
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
