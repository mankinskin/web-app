use budget::{
    currency::*,
    transaction::Transaction,
};
use yew::{
    *,
};

pub struct TransactionView<C: 'static + Currency> {
    props: Transaction<C>,
}
impl<C: 'static + Currency> From<Transaction<C>> for TransactionView<C> {
    fn from(transaction: Transaction<C>) -> Self {
        Self {
            props: transaction,
        }
    }
}
impl<C: 'static + Currency> yew::Component for TransactionView<C> {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: yew::ComponentLink<Self>) -> Self {
        Self::from(Transaction::default())
    }
    fn update(&mut self, _: Self::Message) -> yew::ShouldRender {
        true
    }
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }
    fn view(&self) -> yew::Html {
        html!{
            <div class="transaction-row">
                <div class="transaction-cell">{
                    self.props.get_date().map(|d| format!("{}", d)).unwrap_or("unknown".into())
                }</div>
                <div class="transaction-cell">{
                    self.props.get_amount().to_string()
                }</div>
                <div class="transaction-cell">{
                    self.props.get_recipient().map(|s| s.to_string()).unwrap_or("None".into())
                }</div>
                <div class="transaction-cell">{
                    self.props.get_purposes().map(|ps| ps.to_string()).unwrap_or("None".into())
                }</div>
            </div>
        }
    }
}
