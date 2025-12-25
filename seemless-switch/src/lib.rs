use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

mod editor;
mod params;
mod processor;
mod state;

use params::SeemlessSwitchParams;
use state::InternalState;

pub struct SeemlessSwitch {
    state: InternalState,
    params: Arc<SeemlessSwitchParams>,
    editor_state: Arc<nih_plug_vizia::ViziaState>,
}

impl Default for SeemlessSwitch {
    fn default() -> Self {
        Self {
            params: Arc::new(SeemlessSwitchParams::default()),
            state: InternalState::default(),
            editor_state: ViziaState::new(|| (300, 250)),
        }
    }
}

impl Plugin for SeemlessSwitch {
    const NAME: &'static str = "Seemless Switch";
    const VENDOR: &'static str = "Dany Laporte";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create_editor(self.params.clone(), self.editor_state.clone())
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
        processor::process_state_transitions(&mut self.state, &self.params, ctx);
        processor::process_events_on_state(&mut self.state, &self.params, ctx);
        ProcessStatus::Normal
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
