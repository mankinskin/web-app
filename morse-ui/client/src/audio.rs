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
    Slider(SliderMsg),
}
#[derive(Clone)]
struct AudioRack {
    ctx: AudioContext,
    osc: OscillatorNode,
    gain: GainNode,
    transition_time: f64,
}
impl AudioRack {
    fn create() -> Self {
        info!("Initializing AudioRack");
        let ctx = web_sys::AudioContext::new().expect("Failed to create AudioContext.");
        let osc = ctx.create_oscillator().expect("Failed to create Oscillator.");
        osc.set_type(OscillatorType::Sine);
        let gain = ctx.create_gain().expect("Failed to create Gain.");
        gain.gain().set_value(0.0);

        osc.connect_with_audio_node(&gain).expect("Can't connect Oscillator with Gain.");
        gain.connect_with_audio_node(&ctx.destination()).expect("Can't connect Oscillator with Context destination.");
        osc.start().expect("Failed to start Oscillator.");
        Self {
            ctx,
            osc,
            gain,
            transition_time: 0.0004,
        }
    }
    #[allow(unused)]
    pub fn set_gain(&mut self, gain: f32) {
        self.gain.gain().set_value(gain.clamp(0.0, 1.0));
    }
    pub fn start(&mut self) {
        self.gain_transition(1.0, 0.01);
    }
    pub fn stop(&mut self) {
        self.gain_transition(0.0, 0.02);
    }
    pub fn set_transition_time(&mut self, time: f64) {
        self.transition_time = time;
    }
    pub fn gain_transition(&mut self, target: f32, curve: f64) {
        info!("Transition time {}", self.transition_time);
        self.gain.gain()
            .set_target_at_time(
                target.clamp(0.0, 1.0),
                curve,
                self.transition_time,
                )
            .expect("Failed to start gain transition.");
    }
}
impl Drop for AudioRack {
    fn drop(&mut self) {
        self.osc.stop().expect("Failed to stop Oscillator.");
    }
}
#[derive(Clone)]
pub struct Audio {
    rack: Option<AudioRack>,
    transition_time: Slider,
}
impl Init<()> for Audio {
    fn init(_: (), _orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        Self {
            rack: None,
            transition_time: Slider::new(0.0004, 0.0, 0.1, "transition_time"),
        }
    }
}
impl Audio {
    fn initialize(&mut self) {
        self.rack.get_or_insert_with(AudioRack::create);
    }
    pub fn start(&mut self) {
        self.initialize();
        let time = self.transition_time.get_value();
        self.rack.as_mut().map(|rack| {
            rack.set_transition_time(time);
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
            Self::Msg::Slider(msg) => self.transition_time
                .update(msg, &mut orders.proxy(Self::Msg::Slider)),
        }
    }
}
impl Viewable for Audio {
    fn view(&self) -> Node<AudioMsg> {
        div![
            self.transition_time
                .view()
                .map_msg(AudioMsg::Slider),
        ]
    }
}
