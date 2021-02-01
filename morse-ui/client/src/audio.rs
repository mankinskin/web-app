use components::{
    Component,
    Init,
    Viewable,
};
use seed::{
    prelude::*,
    *,
};
#[allow(unused)]
use tracing::{
    debug,
    error,
    info,
    trace,
};
use web_sys::{
    AudioContext,
    OscillatorNode,
    OscillatorType,
    GainNode,
};
use crate::slider::*;

#[derive(Debug, Clone)]
pub enum AudioMsg {
    Start,
    Stop,
}
#[derive(Clone)]
struct AudioRack {
    ctx: AudioContext,
    osc: Option<OscillatorNode>,
    gain: GainNode,
    transition_time: f64,
}
impl AudioRack {
    fn create() -> Self {
        info!("Initializing AudioRack");
        let ctx = web_sys::AudioContext::new().expect("Failed to create AudioContext.");
        let gain = ctx.create_gain().expect("Failed to create Gain.");
        gain.gain().set_value(0.0);

        gain.connect_with_audio_node(&ctx.destination()).expect("Can't connect Oscillator with Context destination.");
        Self {
            ctx,
            gain,
            osc: None,
            transition_time: 0.001,
        }
    }
    #[allow(unused)]
    pub fn set_gain(&mut self, gain: f32) {
        self.gain.gain().set_value(gain.clamp(0.0, 1.0));
    }
    pub fn start(&mut self) {
        let osc = self.ctx.create_oscillator().expect("Failed to create Oscillator.");
        osc.set_type(OscillatorType::Sine);
        osc.connect_with_audio_node(&self.gain).expect("Can't connect Oscillator with Gain.");
        let now = self.ctx.current_time();
        osc.start_with_when(now).expect("Failed to start Oscillator.");
        self.osc = Some(osc);
        self.linear_ramp(1.0, now + self.transition_time);
    }
    pub fn stop(&mut self) {
        if let Some(osc) = self.osc.take() {
            let now = self.ctx.current_time();
            self.set_gain_at(1.0, now);
            let length = self.transition_time;
            osc.stop_with_when(now + length).expect("Failed to stop Oscillator.");
            self.linear_ramp(0.0, now + length);
        }
    }
    pub fn set_transition_time(&mut self, time: f64) {
        self.transition_time = time;
    }
    pub fn linear_ramp(&mut self, target: f32, time: f64) {
        self.gain.gain()
            .linear_ramp_to_value_at_time(
                target.clamp(0.0, 1.0),
                time,
                )
            .expect("Failed to start gain transition.");
    }
    pub fn set_gain_at(&mut self, target: f32, time: f64) {
        self.gain.gain()
            .set_value_at_time(
                target.clamp(0.0, 1.0),
                time,
                )
            .expect("Failed to start gain transition.");
    }
}
impl Drop for AudioRack {
    fn drop(&mut self) {
        self.stop();
    }
}
#[derive(Clone)]
pub struct Audio {
    rack: Option<AudioRack>,
}
impl Init<()> for Audio {
    fn init(_: (), _orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        Self {
            rack: None,
        }
    }
}
impl Audio {
    fn initialize(&mut self) {
        self.rack.get_or_insert_with(AudioRack::create);
    }
    pub fn start(&mut self) {
        self.initialize();
        self.rack.as_mut().map(|rack| {
            rack.start();
            rack
        });
    }
    pub fn stop(&mut self) {
        self.rack.as_mut().map(|rack| {
            rack.stop();
            rack
        });
    }
    #[allow(unused)]
    pub fn set_gain(&mut self, gain: f32) {
        self.rack.as_mut().map(|rack| rack.set_gain(gain));
    }
}
impl Component for Audio {
    type Msg = AudioMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Self::Msg::Start => self.start(),
            Self::Msg::Stop => self.stop(),
        }
    }
}
impl Viewable for Audio {
    fn view(&self) -> Node<AudioMsg> {
        div![
        ]
    }
}
