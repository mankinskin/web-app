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
use url::{
    *,
};
use common::{
    fetch::*,
};
use std::fmt::{Debug};
use stdweb::web::{
    *,
    html_element::{TextAreaElement},
};
use stdweb::unstable::TryInto;

pub enum Msg {
    SetText(String),
    Note(FetchResponse<()>),
    PostNote,
}
impl From<FetchResponse<()>> for Msg {
    fn from(m: FetchResponse<()>) -> Self {
        Self::Note(m)
    }
}

#[derive(Properties, Clone, Debug)]
pub struct NoteData {
    pub note: Url
}
pub struct NoteEditor {
    props: NoteData,
    link: ComponentLink<Self>,
    note: Option<Note>
}

impl NoteEditor {
    fn text_input_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::SetText(input.value)
        })
    }
    fn post_note_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(move |input: ClickEvent| {
            Msg::PostNote
        })
    }
}
impl Component for NoteEditor {
    type Message = Msg;
    type Properties = NoteData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut s = Self {
            link,
            props,
            note: None,
        };
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
                self.note =
                    self.note.clone().map(|mut note| {
                        note.set_text(text.clone());
                        note
                    })
                    .or_else(move || Some(Note::new(text)));
            },
            Msg::Note(res) => {
                console!(log, format!("{:#?}", res));
                match res.into_inner() {
                    Ok(()) => {}
                    Err(e) => console!(log, format!("{:#?}", e)),
                }
            },
            Msg::PostNote => {
                Fetch::post(self.props.note.clone(), self.note.clone().unwrap_or(Note::new("")))
                    .responder(self.link.callback(|response| {
                        Msg::Note(response)
                    }))
                    .send()
                    .expect("Fetch request failed");
            },
        }
        true
    }
}
