use nih_plug::prelude::*;
use std::sync::Arc;

struct ProgramChange {
    params: Arc<ProgramChangeParams>,
    sent: Option<SendProgramChange>,
}

impl Default for ProgramChange {
    fn default() -> Self {
        Self {
            params: Arc::new(ProgramChangeParams::default()),
            sent: None,
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

    fn deactivate(&mut self) {
        self.sent = None;
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sent = None;
        true
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
        while let Some(event) = context.next_event() {
            let expected = self.params.snapshot();

            if self.sent.as_ref().map_or(true, |s| s != &expected) {
                let channel = expected.channel.saturating_sub(1).clamp(0, 15) as u8;
    
                context.send_event(NoteEvent::MidiCC {
                    timing: 0,
                    channel,
                    cc: 0,
                    value: expected.msb.clamp(0, 127) as f32 / 127.0,
                });
                context.send_event(NoteEvent::MidiCC {
                    timing: 0,
                    channel,
                    cc: 32,
                    value: expected.lsb.clamp(0, 127) as f32 / 127.0,
                });
                context.send_event(NoteEvent::MidiProgramChange {
                    timing: 0,
                    channel,
                    program: expected.pc.clamp(1, 128) as u8 - 1,
                });
                
                self.sent = Some(expected);
            }

            context.send_event(event);
        }

        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        self.sent = None;
    }
}

#[derive(PartialEq)]
struct SendProgramChange {
    pc: i32,
    lsb: i32,
    msb: i32,
    channel: i32,
}

#[derive(Params)]
struct ProgramChangeParams {
    #[id = "channel"]
    channel: IntParam,

    #[id = "pc"]
    pc: IntParam,

    #[id = "msb"]
    msb: IntParam,

    #[id = "lsb"]
    lsb: IntParam,
}

impl ProgramChangeParams {
    fn snapshot(&self) -> SendProgramChange {
        SendProgramChange {
            msb: self.msb.value(),
            channel: self.channel.value(),
            pc: self.pc.value(),
            lsb: self.lsb.value(),
        }
    }
}

impl Default for ProgramChangeParams {
    fn default() -> Self {
        Self {
            channel: IntParam::new("channel", 1, IntRange::Linear { min: 1, max: 16 }),
            msb: IntParam::new("msb", 0, IntRange::Linear { min: 0, max: 127 }),
            lsb: IntParam::new("lsb", 0, IntRange::Linear { min: 0, max: 127 }),
            pc: IntParam::new("pc", 1, IntRange::Linear { min: 1, max: 128 }),
        }
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
