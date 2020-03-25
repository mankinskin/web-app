use plans::{
    *,
    currency::*,
    transaction::Transaction,
};
use yew::*;
use stdweb::web::*;

pub enum Msg {
    PrevImage,
    NextImage,
    Init,
}

pub struct BudgetView<C: Currency> {
    model: Budget<C>,
}
impl<C: Currency> BudgetView<C> {
    pub fn new(b: Budget<C>) -> Self {
        Self {
            model: b,
        }
    }
}
impl yew::Component for BudgetView<Euro> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        let mut b = Budget::create("My Budget", 0);
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        b.get(100).add_purpose("Money");
        BudgetView::new(b)
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

impl yew::Renderable<Self> for BudgetView<Euro> {
    fn view(&self) -> yew::Html<Self> {
        html!{
            <div>
                <h1>{self.model.name()}</h1>
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
struct TransactionsView<C: Currency> {
    model: Transactions<C>,
}
impl yew::Component for TransactionsView<Euro> {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self {
            model: Vec::new().into()
        }
    }
    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }
}
impl yew::Renderable<BudgetView<Euro>> for TransactionsView<Euro> {
    fn view(&self) -> yew::Html<BudgetView<Euro>> {
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
                {for self.model.iter().map(|t| TransactionView::from(t.clone()).view())}
            </table>
                </div>
        }
    }
}
struct TransactionView<C: Currency> {
    model: Transaction<C>,
}
impl From<Transaction<Euro>> for TransactionView<Euro> {
    fn from(transaction: Transaction<Euro>) -> Self {
        Self {
            model: transaction,
        }
    }
}
impl yew::Component for TransactionView<Euro> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self::from(Transaction::default())
    }
    fn update(&mut self, _msg: Self::Message) -> yew::ShouldRender {
        true
    }
}
impl yew::Renderable<BudgetView<Euro>> for TransactionView<Euro> {
    fn view(&self) -> yew::Html<BudgetView<Euro>> {
        html!{
            <tr>
                <td>{format!("{:#?}", self.model.get_date())}</td>
                <td>{self.model.get_amount().to_string()}</td>
                <td>{self.model.get_recipient().map(|s| s.to_string()).unwrap_or("None".into())}</td>
                <td>{self.model.get_purposes().map(|ps| ps.to_string()).unwrap_or("None".into())}</td>
                </tr>
        }
    }
}
