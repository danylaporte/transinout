mod shared;

use nih_plug::prelude::*;
use shared::Shared;
use std::sync::Arc;

struct TransInOut {
    shared: Option<Arc<Shared>>,
    params: Arc<TransInOutParams>,
}

impl Default for TransInOut {
    fn default() -> Self {
        Self {
            params: Arc::new(TransInOutParams::default()),
            shared: None,
        }
    }
}

impl Plugin for TransInOut {
    const NAME: &'static str = "Trans In Out";
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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let expected_song_id = self.params.song_id.value();
        let actual_song_id = self.shared.as_ref().map_or(0, |s| s.id());

        if actual_song_id != expected_song_id {
            self.shared = Some(Shared::get_or_init(expected_song_id))
        }

        true
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
                } => context.send_event(NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note: self.transpose(note),
                    velocity,
                }),
                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => context.send_event(NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note: self.transpose(note),
                    velocity,
                }),
                event => context.send_event(event),
            }
        }

        ProcessStatus::Normal
    }
}

impl TransInOut {
    fn transpose(&self, note: u8) -> u8 {
        let mut amount = self.shared.as_ref().map_or(0, |s| s.amount()) as i16;

        if self.params.invert.value() {
            amount = -amount;
        }

        i16::from(note).saturating_add(amount).clamp(0, 127) as u8
    }
}

#[derive(Params)]
struct TransInOutParams {
    #[id = "id"]
    song_id: IntParam,

    #[id = "invert"]
    invert: BoolParam,
}

impl Default for TransInOutParams {
    fn default() -> Self {
        Self {
            song_id: IntParam::new("song id", 0, IntRange::Linear { min: 0, max: i32::MAX }),
            invert: BoolParam::new("invert", false),
        }
    }
}

impl ClapPlugin for TransInOut {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.trans-in-out";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Transpose before and after midi effect");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for TransInOut {
    const VST3_CLASS_ID: [u8; 16] = *b"TransInOut202312";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(TransInOut);
nih_export_vst3!(TransInOut);
