use crate::budget::*;
use crate::currency::*;
use yew::*;
use stdweb::{
    *,
    unstable::TryInto,
    web::{
        *,
        html_element::*,
    },
};

pub enum Msg {
    Click,
    Init,
}

impl yew::Component for Budget<Euro> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        console!(log, "Create Budget");
        let mut b = Budget::create("My Budget", 0);
        b.get(100).set_purpose("Money");
        b.get(100).set_purpose("Money");
        b.get(100).set_purpose("Money");
        b
    }
    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Msg::Init => {
                console!(log, "Init");
                let canvas: CanvasElement =
                    document()
                    .get_element_by_id("MyCanvas")
                    .expect("Could not find MyCanvas")
                    .try_into()
                    .expect("Unable to convert Element to CanvasElement");
                canvas.set_width(200);
                canvas.set_height(200);
                let ctx: CanvasRenderingContext2d =
                    canvas
                    .get_context()
                    .expect("Cannot get CanvasContext");
                ctx.set_fill_style_color("#000000");
                ctx.fill_rect(0.0, 0.0, 200.0, 200.0);

                true
            },
            Msg::Click => {
                println!("Click");
                let canvas: CanvasElement =
                    document()
                    .get_element_by_id("MyCanvas")
                    .expect("Could not find MyCanvas")
                    .try_into()
                    .expect("Unable to convert Element to CanvasElement");
                let ctx: CanvasRenderingContext2d =
                    canvas
                    .get_context()
                    .expect("Cannot get CanvasContext");
                ctx.set_stroke_style_color("#FF0000");
                ctx.move_to(0.0, 0.0);
                ctx.line_to(100.0, 100.0);
                ctx.stroke();
                ctx.set_font("30px Arial");
                ctx.stroke_text("Hello World", 10.0, 50.0, None);

                false
            }
        }
    }
}

impl yew::Renderable<Self> for Budget<Euro> {
    fn view(&self) -> yew::Html<Self> {
        html!{
            <div>
                <h1 align="center">{self.name()}</h1>
                <style>{
                    "
                    body {
                        background-color: rgb(20, 20, 20);
                    }
                    h1, table, td, caption {
                        font-family: Impact, Charcoal, sans-serif;
                        color: lightblue;
                    }
                    .transaction {
                        color: blue;
                    }
                    .flex-container {
                        display: flex;
                        flex-direction: row;
                    }
                    table, th, td {
                        border-bottom: 1px solid lightgray;
                        border-collapse: collapse;
                        padding: 10px;
                    }
                    caption {
                        font-weight: bold;
                    }
                    "
                }</style>
                <table align="center">
                <caption onclick=|_| Msg::Click>{"Transactions"}</caption>
                <tr>
                    <td>{"Date"}</td>
                    <td>{"Amount"}</td>
                    <td>{"Partner"}</td>
                    <td>{"Purposes"}</td>
                </tr>{
                    for self.transactions.iter().map(yew::Renderable::view)
                }</table>
                <canvas id="MyCanvas"></canvas>
            </div>
        }
    }
}

use crate::transaction::Transaction;
impl yew::Component for Transaction<Euro> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Transaction::default()
    }
    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }
}
impl yew::Renderable<Budget<Euro>> for Transaction<Euro> {
    fn view(&self) -> yew::Html<Budget<Euro>> {
            html!{
                <tr>
                    <td>{self.get_date_string()}</td>
                    <td>{self.get_amount_string()}</td>
                    <td>{self.get_partner_string()}</td>
                    <td>{self.get_purpose_string()}</td>
                </tr>
            }
    }
}
