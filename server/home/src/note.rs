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
        Json,
        Nothing,
    },
};
use http::{
    *,
    request::*,
};
use crate::{
    *,
    budget::*,
};
use anyhow::Error;
use std::result::Result;
use std::fmt::{Display, Debug};

pub enum Msg {
    SetText(String),
    PostNote,
    PostNoteSuccess,
    PostNoteError(String),
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
pub struct NoteInput {
    link: ComponentLink<Self>,
    props: NoteData,
    fetch_task: Option<FetchTask>,
    fetch_service: FetchService,
}

impl NoteInput {
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
}
impl Component for NoteInput {
    type Message = Msg;
    type Properties = NoteData;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            fetch_service: FetchService::new(),
            fetch_task: None,
        }
    }
    fn view(&self) -> Html {
        html!{
            <div class="note-writer-container">
                <textarea class="note-writer-input" oninput=self.text_input_callback()/>
                <button class="note-submit-button" onclick=self.post_note_callback()>{"Submit"}</button>
                <div>{
                    format!("{:#?}", self.props.note)
                }</div>
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
                let note = self.props.note.clone().unwrap();
                let json = serde_json::to_string(&note).unwrap();
                let req = Request::post("/api/note")
                    .header("Content-Type", "application/json")
                    .body(Ok(json))
                    .unwrap();
                let callback = self.link.callback(|response: Response<Nothing>| {
                    let (meta, Nothing) = response.into_parts();
                    if meta.status.is_success() {
                        console!(log, "Post success");
                        Msg::PostNoteSuccess
                    } else {
                        Msg::PostNoteError(
                            meta.status.clone()
                                .canonical_reason()
                                .map(ToString::to_string)
                                .unwrap_or(format!("Got StatusCode {}", meta.status))
                        )
                    }
                });
                let mut fetch_service = FetchService::new();
                let task = fetch_service.fetch(req, callback);
                match task {
                    Ok(task) => {
                        self.fetch_task = Some(task)
                    },
                    Err(err) => {
                        self.link.send_message(Msg::PostNoteError(err.to_string()))
                    },
                }
                true
            },
            Msg::PostNoteSuccess => {
                true
            },
            Msg::PostNoteError(_) => {
                true
            },
        }
    }
}
