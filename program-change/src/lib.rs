use nih_plug::prelude::*;
use std::sync::Arc;

struct ProgramChange {
    params: Arc<ProgramChangeParams>,
}

impl Default for ProgramChange {
    fn default() -> Self {
        Self {
            params: Arc::new(ProgramChangeParams::default()),
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
        while let Some(event) = context.next_event() {
            context.send_event(match event {
                NoteEvent::MidiCC {
                    timing,
                    channel,
                    cc: 03,
                    value,
                } => NoteEvent::MidiProgramChange {
                    timing,
                    channel,
                    program: (value * 127.0) as u8,
                },
                e => e,
            });
        }

        ProcessStatus::Normal
    }
}

#[derive(Params)]
struct ProgramChangeParams {}

impl Default for ProgramChangeParams {
    fn default() -> Self {
        Self {}
    }
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
