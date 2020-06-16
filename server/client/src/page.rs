use crate::*;
#[derive(Clone)]
pub enum Model {
    Login(login::Model),
    Register(register::Model),
    Home(home::Model),
    User(user::Model),
}
impl Model {
    pub fn home() -> Self {
        Self::Home(home::Model::default())
    }
    pub fn user(id: Id<User>) -> Self {
        Self::User(user::Model::from(id))
    }
    pub fn login() -> Self {
        Self::Login(login::Model::default())
    }
    pub fn register() -> Self {
        Self::Register(register::Model::default())
    }
}
#[derive(Clone)]
pub enum Msg {
    Home(home::Msg),
    User(user::Msg),
    Login(login::Msg),
    Register(register::Msg),
    Root(Box<root::Msg>)
}
impl From<home::Msg> for Msg {
    fn from(msg: home::Msg) -> Self {
        Self::Home(msg)
    }
}
impl From<user::Msg> for Msg {
    fn from(msg: user::Msg) -> Self {
        Self::User(msg)
    }
}
impl From<login::Msg> for Msg {
    fn from(msg: login::Msg) -> Self {
        Self::Login(msg)
    }
}
impl From<register::Msg> for Msg {
    fn from(msg: register::Msg) -> Self {
        Self::Register(msg)
    }
}
impl Default for Model {
    fn default() -> Self {
        Model::Home(home::Model::default())
    }
}
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Root(_) => {
        },
        Msg::Home(msg) => {
            if let Model::Home(home) = model {
                home::update(
                    msg,
                    home,
                    &mut orders.proxy(Msg::Home)
                );
            }
        },
        Msg::User(msg) => {
            if let Model::User(user) = model {
                user::update(
                    msg,
                    user,
                    &mut orders.proxy(Msg::User)
                );
            }
        },
        Msg::Login(msg) => {
            if let Model::Login(login) = model {
                match &msg {
                    login::Msg::Register => {
                        orders.send_msg(Msg::Root(Box::new(root::Msg::SetPage(Model::register()))));
                    },
                    _ => {},
                }
                login::update(
                    msg,
                    login,
                    &mut orders.proxy(Msg::Login)
                );
            }
        },
        Msg::Register(msg) => {
            if let Model::Register(register) = model {
                match &msg {
                    register::Msg::Login => {
                        orders.send_msg(Msg::Root(Box::new(root::Msg::SetPage(Model::login()))));
                    },
                    _ => {},
                }
                register::update(
                    msg,
                    register,
                    &mut orders.proxy(Msg::Register)
                );
            }
        },
    }
}
pub fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::Home(home) =>
            home::view(&home)
            .map_msg(Msg::Home),
        Model::User(user) =>
            user::view(&user)
            .map_msg(Msg::User),
        Model::Login(login) =>
            login::view(&login)
            .map_msg(Msg::Login),
        Model::Register(register) =>
            register::view(&register)
            .map_msg(Msg::Register),
    }
}
