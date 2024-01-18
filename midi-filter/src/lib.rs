use nih_plug::{
    midi::control_change::{
        BREATH_CONTROLLER_LSB, BREATH_CONTROLLER_MSB, DAMPER_PEDAL, DATA_ENTRY_LSB, DATA_ENTRY_MSB,
        EXPRESSION_CONTROLLER_LSB, EXPRESSION_CONTROLLER_MSB, FOOT_CONTROLLER_LSB,
        FOOT_CONTROLLER_MSB, MODULATION_LSB, MODULATION_MSB,
    },
    prelude::*,
};
use std::sync::Arc;

struct MidiFilter {
    params: Arc<MidiFilterParams>,
}

impl Default for MidiFilter {
    fn default() -> Self {
        Self {
            params: Arc::new(MidiFilterParams::default()),
        }
    }
}

impl Plugin for MidiFilter {
    const NAME: &'static str = "Midi filter";
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
                event @ NoteEvent::MidiCC { cc, .. } => {
                    let allow = match cc {
                        BREATH_CONTROLLER_MSB | BREATH_CONTROLLER_LSB => self.params.bc.value(),
                        DATA_ENTRY_MSB | DATA_ENTRY_LSB => self.params.de.value(),
                        DAMPER_PEDAL => self.params.dp.value(),
                        EXPRESSION_CONTROLLER_MSB | EXPRESSION_CONTROLLER_LSB => {
                            self.params.ex.value()
                        }
                        FOOT_CONTROLLER_MSB | FOOT_CONTROLLER_LSB => self.params.fp.value(),
                        MODULATION_MSB | MODULATION_LSB => self.params.mw.value(),
                        _ => true,
                    };

                    if allow {
                        context.send_event(event);
                    }
                }
                event @ NoteEvent::MidiPitchBend { .. } => {
                    if self.params.pb.value() {
                        context.send_event(event);
                    }
                }
                event => context.send_event(event),
            }
        }

        ProcessStatus::Normal
    }
}

#[derive(Params)]
struct MidiFilterParams {
    /// Breath Controller
    #[id = "bc"]
    bc: BoolParam,

    /// Data Entry
    #[id = "de"]
    de: BoolParam,

    /// Damper Pedal (sustain)
    #[id = "dp"]
    dp: BoolParam,

    /// Expression
    #[id = "ex"]
    ex: BoolParam,

    /// Foot Pedal
    #[id = "fp"]
    fp: BoolParam,

    /// Modulation Wheel
    #[id = "mw"]
    mw: BoolParam,

    /// Pitch-Bend
    #[id = "pb"]
    pb: BoolParam,
}

impl Default for MidiFilterParams {
    fn default() -> Self {
        Self {
            bc: BoolParam::new("Breath Controller", false),
            dp: BoolParam::new("Sustain Pedal", false),
            de: BoolParam::new("Data Entry", false),
            fp: BoolParam::new("Foot Pedal", false),
            ex: BoolParam::new("Expression", false),
            mw: BoolParam::new("Mod Wheel", false),
            pb: BoolParam::new("Pitch Bend", false),
        }
    }
}

impl ClapPlugin for MidiFilter {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.midi-filter";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Filter midi message.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::NoteEffect, ClapFeature::Utility];
}

impl Vst3Plugin for MidiFilter {
    const VST3_CLASS_ID: [u8; 16] = *b"MidiFilter202401";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Tools];
}

nih_export_clap!(MidiFilter);
nih_export_vst3!(MidiFilter);
