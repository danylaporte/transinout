use nih_plug::prelude::*;
use std::sync::Arc;

struct SingleNote {
    params: Arc<SingleNoteParams>,
    states: NotesState,
}

impl Default for SingleNote {
    fn default() -> Self {
        Self {
            params: Arc::new(SingleNoteParams::default()),
            states: Default::default(),
        }
    }
}

impl Plugin for SingleNote {
    const NAME: &'static str = "Single Note";
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
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    let is_off = self.states.is_all_off();

                    self.states.set_on(note);

                    if is_off {
                        context.send_event(NoteEvent::NoteOn {
                            timing,
                            voice_id,
                            channel,
                            note: self.params.note(),
                            velocity,
                        });
                    }
                }
                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    self.states.set_off(note);

                    let is_off = self.states.is_all_off();

                    if is_off {
                        context.send_event(NoteEvent::NoteOff {
                            timing,
                            voice_id,
                            channel,
                            note: self.params.note(),
                            velocity,
                        });
                    }
                }

                event => context.send_event(event),
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

#[derive(Params)]
struct SingleNoteParams {
    #[id = "note"]
    note: IntParam,
}

impl Default for SingleNoteParams {
    fn default() -> Self {
        Self {
            note: IntParam::new("Target note", 0, IntRange::Linear { min: 0, max: 127 }),
        }
    }
}

impl SingleNoteParams {
    fn note(&self) -> u8 {
        self.note.value().clamp(0, 127) as u8
    }
}

impl ClapPlugin for SingleNote {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.signle-note";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Rewrite all notes to a single note.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for SingleNote {
    const VST3_CLASS_ID: [u8; 16] = *b"SingleNote202401";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(SingleNote);
nih_export_vst3!(SingleNote);
