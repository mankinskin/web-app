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
fn init_tracing() {
	console_error_panic_hook::set_once();
	tracing_wasm::set_as_global_default();
	debug!("Tracing initialized.");
	debug!("Debug logs enabled.");
	info!("Info logs enabled.");
	trace!("Trace logs enabled.");
	error!("Error logs enabled.");
}
#[wasm_bindgen(start)]
pub fn render() {
    init_tracing();
    debug!("Starting App");
    App::start(
        "app",
        Root::init,
        |msg, model, orders| model.update(msg, orders),
        Viewable::view,
    );
}
#[derive(Debug, Clone)]
enum Msg {
    Button(ButtonMsg),
    Audio(AudioMsg),
    Click,
    Release
}
struct Root {
    button: Button,
    audio: Audio,   
}

impl Init<Url> for Root {
    fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Self {
        orders.subscribe(|msg: ButtonMsg| {
            match msg {
                ButtonMsg::Click => Some(Msg::Click),
                ButtonMsg::Release => Some(Msg::Release),
            }
        });
        Self {
            button: Button,
            audio: Audio::init((), &mut orders.proxy(Msg::Audio)),
        }
    }
}
impl Component for Root {
    type Msg = Msg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::Button(msg) => {
                self.button.update(msg, &mut orders.proxy(Msg::Button));
            },
            Msg::Audio(msg) => {
                self.audio.update(msg, &mut orders.proxy(Msg::Audio));
            },
            Self::Msg::Click => {
                debug!("Click!");
                self.audio.start();
            },
            Self::Msg::Release => {
                debug!("Release!");
                self.audio.stop();
            },
        }
    }
}
impl Viewable for Root {
    fn view(&self) -> Node<Msg> {
        div![
            "Hello World",
            self.button,
            self.audio,
        ]
    }
}

#[derive(Debug, Clone)]
enum AudioMsg {
    Start,
    Stop,
    Slider(SliderMsg),
}
struct Audio {
    ctx: AudioContext,
    osc: OscillatorNode,
    gain: GainNode,
    transition_time: Slider,
}
impl Init<()> for Audio {
    fn init(_: (), orders: &mut impl Orders<<Self as Component>::Msg>) -> Self {
        let ctx = web_sys::AudioContext::new().expect("Failed to create AudioContext.");
        let mut gain = ctx.create_gain().expect("Failed to create Gain.");
        let osc = ctx.create_oscillator().expect("Failed to create Oscillator.");
        gain.gain().set_value(0.0);
        osc.set_type(OscillatorType::Sine);

        gain.connect_with_audio_node(&ctx.destination()).expect("Can't connect Oscillator with Context destination.");
        osc.connect_with_audio_node(&gain).expect("Can't connect Oscillator with Gain.");
        osc.start().expect("Failed to start Oscillator.");
        Self {
            ctx,
            osc,
            gain,
            transition_time: Slider::new(0.0, 0.0, 10.0, "transition_time"),
        }
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.osc.stop().expect("Failed to stop Oscillator.");
    }
}
impl Audio {
    pub fn start(&mut self) {
        self.gain_transition(1.0, 0.01);
    }
    pub fn stop(&mut self) {
        self.gain_transition(0.0, 0.02);
    }
    pub fn set_transition_time(&mut self, t: f64) {
        self.transition_time.set_value(t);
    }
    pub fn set_gain(&mut self, gain: f32) {
        self.gain.gain().set_value(gain.clamp(0.0, 1.0));
    }
    pub fn gain_transition(&mut self, target: f32, curve: f64) {
        self.gain.gain()
            .set_target_at_time(
                target.clamp(0.0, 1.0),
                self.transition_time.get_value(),
                curve,
            )
            .expect("Failed to start gain transition.");
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
            self.transition_time,
        ]
    }
}
impl UpdateEl<Msg> for &Audio {
    fn update_el(self, el: &mut El<Msg>) {
        self.view().map_msg(Msg::Audio).update_el(el)
    }
}

#[derive(Debug, Clone)]
enum SliderMsg {
    Change(f64),
}
struct Slider {
    label: String,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
}
impl Slider {
    pub fn new(value: f64, min: f64, max: f64, label: impl ToString) -> Self {
        Self {
            value,
            min,
            max,
            label: label.to_string(),
            step: 0.001,
        }
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
    pub fn set_value(&mut self, v: f64) {
        self.value = v;
    }
}
impl Component for Slider {
    type Msg = SliderMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match &msg {
            Self::Msg::Change(v) => self.value = *v,
        }
        orders.notify(msg);
    }
}
impl Viewable for Slider {
    fn view(&self) -> Node<SliderMsg> {
        div![
            &self.label,
            input![
                attrs!{
                    At::Type => "range";
                    At::Min => format!("{}", self.min);
                    At::Max => format!("{}", self.max);
                    At::Value => format!("{}", self.value);
                    At::Step => format!("{}", self.step);
                },
                input_ev(Ev::Input, |s| {
                    SliderMsg::Change(s.parse::<f64>().expect("Failed to parse slider value to f64"))
                })
            ],
            format!("{}", self.value),
        ]
    }
}
impl UpdateEl<AudioMsg> for &Slider {
    fn update_el(self, el: &mut El<AudioMsg>) {
        self.view().map_msg(AudioMsg::Slider).update_el(el)
    }
}
#[derive(Debug, Clone)]
enum ButtonMsg {
    Click,
    Release,
}
struct Button;

impl UpdateEl<Msg> for &Button {
    fn update_el(self, el: &mut El<Msg>) {
        self.view().map_msg(Msg::Button).update_el(el)
    }
}
impl Component for Button {
    type Msg = ButtonMsg;
    fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) {
        match &msg {
            Self::Msg::Click => {},
            Self::Msg::Release => {},
        }
        orders.notify(msg);
    }
}
impl Viewable for Button {
    fn view(&self) -> Node<<Self as Component>::Msg> {
        button![
            "Click!",
            ev(Ev::MouseDown, |_| {
                Self::Msg::Click       
            }),
            ev(Ev::MouseLeave, |_| {
                Self::Msg::Release       
            }),
            ev(Ev::MouseUp, |_| {
                Self::Msg::Release       
            }),
        ]
    }
}
