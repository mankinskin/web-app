pub use plans::{
    *,
    note::*,
};
use yew::{
    *,
};
use crate::{
    *,
};
use common::{
    remote_data::*,
};
use std::fmt::{Debug};
use stdweb::web::{
    *,
    html_element::{TextAreaElement},
};
use stdweb::unstable::TryInto;
use futures::{Future, FutureExt};
use std::result::{Result};

pub enum Msg {
    SetText(String),
    RemoteNote(RemoteMsg<Note>),
}

#[derive(Properties, Clone, Debug)]
pub struct NoteData {
    pub note: RemoteData<Note>
}
pub struct NoteEditor {
    link: ComponentLink<Self>,
    props: NoteData,
}

impl NoteEditor {
    fn text_input_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::SetText(input.value)
        })
    }
    fn post_note_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(move |input: ClickEvent| {
            Msg::RemoteNote(RemoteMsg::Request(FetchMethod::Post))
        })
    }
    fn note_responder(&self) -> Callback<FetchResponse<Note>> {
        self.link.callback(move |response: FetchResponse<Note>| {
            Msg::RemoteNote(RemoteMsg::Response(response))
        })
    }
    fn note_request(&self, method: FetchMethod) -> Result<impl Future<Output=()> + 'static, anyhow::Error> {
        let callback = self.note_responder().clone();
        Ok(self.props.note.fetch_request(method)?
            .then(move |res: FetchResponse<Note>| {
                futures::future::ready(callback.emit(res))
            }))
    }
}
impl Component for NoteEditor {
    type Message = Msg;
    type Properties = NoteData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Self {
            link,
            props,
        };
        s.props.note.set_data(Note::new(""));
        s
    }
    fn rendered(&mut self, _first_render: bool) {
        let text_area: TextAreaElement = stdweb::web::document()
            .query_selector(".note-editor-textarea").unwrap()
            .expect("note-editor-textarea not found")
            .try_into()
            .expect("Failed to cast to HtmlElement");
        text_area.clone().add_event_listener(move |_: KeyDownEvent| {
            text_area.set_attribute("style", "height: auto; padding: 0;")
                .expect("Failed to set attribute");
            let scrolled_px: i32 = text_area.scroll_top() as i32;
            let height = text_area.offset_height();
            let cmd = format!("height: {}px;", scrolled_px + height);
            text_area.set_attribute("style", &cmd)
                .expect("Failed to set attribute");
        });
    }
    fn view(&self) -> Html {
        html!{
            <div class="note-editor-container">
                <div class="note-editor-header">{
                    "New Note"
                }</div>
                <textarea class="note-editor-textarea" oninput=self.text_input_callback()/>
                <div class="note-preview-container">{
                    format!("{:#?}", self.props.note)
                }</div>
                <button class="note-submit-button" onclick=self.post_note_callback()>{"Submit"}</button>
                //<div class="submit-status">
                //    <StatusStackView<(), String> stack={self.post_status.clone()} />
                //</div>
            </div>
        }
    }
    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetText(text) => {
                *self.props.note.data_mut() =
                    self.props.note.data().clone().map(|mut note| {
                        note.set_text(text.clone());
                        note
                    })
                    .or_else(move || Some(Note::new(text)));
            },
            Msg::RemoteNote(msg) => {
                console!(log, format!("{:#?}", msg));
                match msg {
                    RemoteMsg::Request(request) => {
                        let future = self
                            .note_request(request)
                            .expect("Failed to make request");
                        wasm_bindgen_futures::spawn_local(future);
                    },
                    RemoteMsg::Response(response) => {
                        if let Err(e) = self.props.note.respond(response) {
                            console!(log, format!("{:#?}", e));
                        }
                    },
                }
            },
        }
        true
    }
}
