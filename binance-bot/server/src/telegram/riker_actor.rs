
use riker::actors::*;
use telegram_bot::Update;
use tokio::task::JoinHandle;
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    trace,
};
use futures::stream::{
    StreamExt,
};

#[actor(Update)]
#[derive(Default)]
pub struct TelegramActor {
    #[allow(unused)]
    handle: Option<JoinHandle<()>>
}

impl Actor for TelegramActor {
    type Msg = TelegramActorMsg;
    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        let myself = ctx.myself();
        let mut stream = crate::telegram::telegram().api.stream();
        self.handle = Some(ctx.run(async move {
            while let Some(res) = stream.next().await {
                match res {
                    Ok(update) => myself.tell(update, None),
                    Err(e) => error!("{}", e),
                }
            }
        }).expect("Failed to spawn telegram stream!"));
    }
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Option<BasicActorRef>) {
        self.receive(ctx, msg, sender);
    }
}
impl Receive<Update> for TelegramActor {
    type Msg = TelegramActorMsg;
    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: Update, _sender: Sender) {
        ctx.run(async move {
            let mut telegram = crate::telegram::telegram();
            telegram.update(msg).await.unwrap();
        }).expect("Failed to spawn telegram stream!");
    }
}
