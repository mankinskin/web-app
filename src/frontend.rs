use crate::budget::*;
use crate::currency::*;
use yew::*;
use stdweb::web::*;

pub enum Msg {
    Click,
    PrevImage,
    NextImage,
    Init,
}
impl yew::Component for Budget<Euro> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        let mut b = Budget::create("My Budget", 0);
        b.get(100).set_purpose("Money");
        b.get(100).set_purpose("Money");
        b.get(100).set_purpose("Money");
        b
    }
    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        use std::sync::{Mutex};
        lazy_static!{
            static ref INDEX: Mutex<u32> = Mutex::new(0);
        }

        match msg {
            Msg::Init => {
                true
            },
            Msg::PrevImage => {
                let mut i = INDEX.lock().unwrap();
                *i = ((100+*i) - 1)%100;
                document().get_element_by_id("image")
                    .unwrap()
                    .set_attribute("src",
                        &get_img_src(*i))
                    .unwrap();
                true
            },
            Msg::NextImage => {
                let mut i = INDEX.lock().unwrap();
                *i = ((100+*i) + 1)%100;
                document().get_element_by_id("image")
                    .unwrap()
                    .set_attribute("src",
                        &get_img_src(*i))
                    .unwrap();
                true
            },
            _ => false
        }
    }
}

fn get_img_src(index: u32) -> String {
    const SIZE: (u32, u32) = (1000, 1000);
    format!("https://i.picsum.photos/id/{}/{}/{}.jpg",
        index%100,
        SIZE.0,
        SIZE.1)
}

impl yew::Renderable<Self> for Budget<Euro> {
    fn view(&self) -> yew::Html<Self> {
        html!{
            <div>
                <h1>{self.name()}</h1>
                //{self.transactions.view()}
                <img id="image" src={get_img_src(0)}/>
                <br/>
                <button type="button"
                        id="prev-image-button"
                        class="image-navigation-button"
                        onclick=|_| Msg::PrevImage>
                    {"<"}
                </button>
                    <button type="button"
                    id="next-image-button"
                    class="image-navigation-button"
                    onclick=|_| Msg::NextImage>
                    {">"}
                    </button>
                    </div>
        }
    }
}

impl yew::Component for Transactions<Euro> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Vec::new().into()
    }
    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }
}
impl yew::Renderable<Budget<Euro>> for crate::budget::Transactions<Euro> {
    fn view(&self) -> yew::Html<Budget<Euro>> {
        html!{
            <div>
                <table class="transaction-table">
                <caption>{"Your Transactions"}</caption>
                <tr class="transaction-header">
                <th>{"Date"}</th>
                <th>{"Amount"}</th>
                <th>{"Partner"}</th>
                <th>{"Purposes"}</th>
                </tr>
                {for self.iter().map(yew::Renderable::view)}
            </table>
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
