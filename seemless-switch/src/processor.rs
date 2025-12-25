use crate::params::SeemlessSwitchParams;
use crate::state::{DamperState, InternalState, NotesState};
use crate::SeemlessSwitch;
use nih_plug::midi::control_change::DAMPER_PEDAL;
use nih_plug::prelude::*;
use std::mem::take;

pub fn process_state_transitions(
    state: &mut InternalState,
    params: &SeemlessSwitchParams,
    ctx: &mut impl ProcessContext<SeemlessSwitch>,
) {
    const OFF: bool = false;
    const ON: bool = true;

    *state = match (take(state), params.active.value()) {
        (InternalState::Off, OFF) => InternalState::Off,

        (InternalState::Off, ON) => {
            let snapshot = params.snapshot();
            snapshot.send(None, ctx);
            InternalState::On {
                damper: Default::default(),
                notes: Default::default(),
                snapshot,
            }
        }

        (
            InternalState::On {
                damper,
                notes,
                snapshot,
            }
            | InternalState::SeamlessSwitch {
                damper,
                notes,
                snapshot,
            },
            ON,
        ) => {
            let new = params.snapshot();
            new.send(Some(&snapshot), ctx);
            InternalState::On {
                damper,
                notes,
                snapshot: new,
            }
        }

        (
            InternalState::On {
                damper,
                notes,
                snapshot,
            }
            | InternalState::SeamlessSwitch {
                damper,
                notes,
                snapshot,
            },
            OFF,
        ) => {
            if notes.is_all_off() && damper.is_off() {
                InternalState::Off
            } else {
                InternalState::SeamlessSwitch {
                    damper,
                    notes,
                    snapshot,
                }
            }
        }
    };
}

pub fn process_events_on_state(
    state: &mut InternalState,
    params: &SeemlessSwitchParams,
    ctx: &mut impl ProcessContext<SeemlessSwitch>,
) {
    match state {
        InternalState::Off => {}

        InternalState::On { damper, notes, .. } => {
            while let Some(event) = ctx.next_event() {
                process_event_on(ctx, event, damper, notes, params);
            }
        }

        InternalState::SeamlessSwitch { damper, notes, .. } => {
            while let Some(event) = ctx.next_event() {
                process_event_seamless(ctx, event, damper, notes);
            }

            if damper.is_off() && notes.is_all_off() {
                *state = InternalState::Off;
            }
        }
    }
}

fn process_event_on(
    ctx: &mut impl ProcessContext<SeemlessSwitch>,
    event: NoteEvent<()>,
    damper: &mut DamperState,
    notes: &mut NotesState,
    params: &SeemlessSwitchParams,
) {
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
                channel: 0,
                note,
            });
        }

        NoteEvent::MidiCC {
            timing, cc, value, ..
        } => {
            if cc == DAMPER_PEDAL {
                if value >= 0.5 {
                    damper.set_on();
                } else {
                    damper.set_off();
                }
            }
            ctx.send_event(NoteEvent::MidiCC {
                timing,
                cc,
                channel: 0,
                value,
            });
        }

        NoteEvent::MidiChannelPressure {
            timing, pressure, ..
        } => {
            ctx.send_event(NoteEvent::MidiChannelPressure {
                timing,
                channel: 0,
                pressure,
            });
        }

        NoteEvent::MidiPitchBend { timing, value, .. } => {
            if params.allow_pitch_bend.value() {
                ctx.send_event(NoteEvent::MidiPitchBend {
                    timing,
                    channel: 0,
                    value,
                });
            }
        }

        NoteEvent::MidiProgramChange { .. } => {
            // Ignore program change events
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
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
                channel: 0,
                note,
            });
        }

        event => ctx.send_event(event),
    }
}

fn process_event_seamless(
    ctx: &mut impl ProcessContext<SeemlessSwitch>,
    event: NoteEvent<()>,
    damper: &mut DamperState,
    notes: &mut NotesState,
) {
    match event {
        NoteEvent::Choke {
            timing,
            voice_id,
            note,
            ..
        } => {
            notes.set_off(note);
            damper.set_off();
            ctx.send_event(NoteEvent::Choke {
                timing,
                voice_id,
                channel: 0,
                note,
            });
        }

        NoteEvent::MidiCC {
            timing, cc, value, ..
        } => {
            if cc == DAMPER_PEDAL && value < 0.5 {
                damper.set_off();
                ctx.send_event(NoteEvent::MidiCC {
                    timing,
                    channel: 0,
                    cc,
                    value: 0.0,
                });
            }
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
                channel: 0,
                note,
                velocity,
            });
        }

        _ => {
            // Ignore all other events during seamless switch
        }
    }
}
