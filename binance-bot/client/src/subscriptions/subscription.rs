use components::{
	Component,
	Init,
	Viewable,
	Editor,
	Previewable,
	editor,
};
use seed::{
	prelude::*,
	*,
};
use shared::{
	subscriptions::{
		PriceSubscription,
	},
	Route,
};
use crate::{
	subscriptions::{
		chart::{
			self,
			SubscriptionChart,
		},
	},
};
use database_table::{
	Entry,
	TableRoutable,
};
use rql::*;
#[allow(unused)]
use tracing::{
	instrument,
	debug,
	info,
};

#[derive(Debug)]
pub struct SubscriptionInfo {
	subscription: PriceSubscription,
	chart: SubscriptionChart,
	editor: Option<Editor<PriceSubscription>>,
}
impl TableRoutable<PriceSubscription> for SubscriptionInfo {
	type Route = shared::subscriptions::Route;
	fn table_route() -> Self::Route {
		PriceSubscription::table_route()
	}
	fn entry_route(id: Id<PriceSubscription>) -> Self::Route {
		PriceSubscription::entry_route(id)
	}
}
//#[async_trait(?Send)]
//impl RemoteTable<PriceSubscription> for SubscriptionInfo {
//	type Error = <PriceSubscription as RemoteTable>::Error;
//	async fn get(id: Id<PriceSubscription>) -> Result<Option<Entry<PriceSubscription>>, Self::Error> {
//		PriceSubscription::get(id).await
//	}
//	async fn delete(id: Id<PriceSubscription>) -> Result<Option<PriceSubscription>, Self::Error> {
//		PriceSubscription::delete(id).await
//	}
//	async fn get_all() -> Result<Vec<Entry<PriceSubscription>>, Self::Error> {
//		debug!("PriceSubscription::get_all");
//		PriceSubscription::get_all().await
//	}
//	async fn post(data: PriceSubscription) -> Result<Id<PriceSubscription>, Self::Error> {
//		PriceSubscription::post(data).await
//	}
//}
#[derive(Clone, Debug)]
pub enum Msg {
	OpenEditor,
	Editor(<Editor<PriceSubscription> as Component>::Msg),
	Chart(chart::Msg),
}
impl Init<Entry<PriceSubscription>> for SubscriptionInfo {
	fn init(entry: Entry<PriceSubscription>, orders: &mut impl Orders<Msg>) -> Self {
		Self {
			subscription: entry.data().clone(),
			chart: SubscriptionChart::init(entry.id().clone(), &mut orders.proxy(Msg::Chart)),
			editor: None,	
		}
	}
}
impl Component for SubscriptionInfo {
	type Msg = Msg;
	fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
		match msg {
			Msg::OpenEditor => {
				self.editor = Some(Editor::default());
			}
			Msg::Editor(msg) => {
				if let Some(ed) = &mut self.editor {
					let new = match msg {
						editor::Msg::Cancel => Some(None),
						editor::Msg::Submit => Some(None),
						_ => None,
					};
					ed.update(msg, &mut orders.proxy(Msg::Editor));
					if let Some(new) = new {
						self.editor = new;
					}
				}
			},
			Msg::Chart(msg) => {
				self.chart.update(msg, &mut orders.proxy(Msg::Chart));
			}
		}
	}
}
impl Viewable for SubscriptionInfo {
	fn view(&self) -> Node<Msg> {
		div![
			format!("{:?}", self.subscription),
			self.chart.view().map_msg(move |msg| Msg::Chart(msg))
		]
	}
}
impl Previewable for SubscriptionInfo {
	fn preview(&self) -> Node<Msg> {
		div![
			format!("{:?}", self.subscription)
		]
	}
}
