pub use plans::{
    *,
    note::*,
};
use yew::{
    *,
    services::{
        *,
        fetch::{
            *,
            FetchTask,
        },
    },
    format::{
        Nothing,
    },
};
use crate::{
    *,
    status_stack::*,
};
use std::fmt::{Debug};
use stdweb::web::{
    *,
    Element,
    HtmlElement,
    html_element::TextAreaElement,
};
use stdweb::unstable::TryInto;
use std::convert::{TryFrom};

pub enum Msg {
    SetText(String),
    PostNote,
    PostNoteStatus(Result<(), String>),
}

#[derive(Properties, Clone, Debug)]
pub struct NoteData {
    pub note: Option<Note>
}
impl From<Note> for NoteData {
    fn from(note: Note) -> Self {
        Self {
            note: Some(note),
        }
    }
}
pub struct NoteEditor {
    link: ComponentLink<Self>,
    props: NoteData,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
    post_status: StatusStack<(), String>,
}

impl NoteEditor {
    fn text_input_callback(&self) -> Callback<InputData> {
        self.link.callback(|input: InputData| {
            Msg::SetText(input.value)
        })
    }
    fn post_note_callback(&self) -> Callback<ClickEvent> {
        self.link.callback(|_: ClickEvent| {
            Msg::PostNote
        })
    }
    fn post_note(&mut self, note: Note) -> Result<(), String> {
        let json = serde_json::to_string(&note).unwrap();
        let req = Request::post("/api/note")
            .header("Content-Type", "application/json")
            .body(Ok(json))
            .unwrap();
        let callback = self.link.callback(|response: Response<Nothing>| {
            let (meta, Nothing) = response.into_parts();
            if meta.status.is_success() {
                Msg::PostNoteStatus(Ok(()))
            } else {
                Msg::PostNoteStatus(Err(
                    meta.status.clone()
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("Got StatusCode {}", meta.status)))
                )
            }
        });
        let task = self.fetch_service.fetch(req, callback);
        match task {
            Ok(task) => {
                self.fetch_task = Some(task);
                Ok(())
            },
            Err(err) => {
                Err(err.to_string())
            },
        }
    }
}
impl Component for NoteEditor {
    type Message = Msg;
    type Properties = NoteData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            fetch_service: FetchService::new(),
            fetch_task: None,
            post_status: StatusStack::new(),
        }
    }
    fn mounted(&mut self) -> ShouldRender {
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
        true
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
                <div class="submit-status">
                    <StatusStackView<(), String> stack={self.post_status.clone()} />
                </div>
            </div>
        }
    }
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetText(text) => {
                self.props.note =
                    self.props.note.clone().map(|mut note| {
                        note.set_text(text.clone());
                        note
                    })
                    .or_else(move || Some(Note::new(text)));
                true
            },
            Msg::PostNote => {
                self.post_status.clear();
                match self.props.note.clone() {
                    Some(note) => {
                        let status = self.post_note(note);
                        self.link.send_message(Msg::PostNoteStatus(status));
                        true
                    },
                    None => {
                        false
                    }
                }
            },
            Msg::PostNoteStatus(status) => {
                self.post_status.push(status);
                true
            },
        }
    }
}
