use super::{
    Msg,
    ExpanderView,
};
use crate::{
    parent_child::*,
};
use std::ops::{
    Deref,
    DerefMut,
};
use yew::{
    *,
};
#[derive(Properties, Clone, Debug)]
pub struct ExpanderData {
    expanded: bool,
}
impl From<bool> for ExpanderData {
    fn from(expanded: bool) -> Self {
        Self {
            expanded,
        }
    }
}
impl ExpanderData {
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }
    pub fn toggle_expanded(&mut self) {
        self.set_expanded(!self.expanded);
    }
    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }
}
#[derive(Properties, Clone, Debug)]
pub struct ExpanderProps {
    pub data: ExpanderData,
    pub children: Children,
    pub parent_callback: Callback<<ExpanderView as Component>::Message>,
}
impl ExpanderProps
{
    pub fn new(expanded: bool) -> Self {
        Self {
            data: ExpanderData::from(expanded),
            children: Children::new(vec![]),
            parent_callback: Callback::noop(),
        }
    }
}
impl Deref for ExpanderProps {
    type Target = ExpanderData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl DerefMut for ExpanderProps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl MessageUpdate<Msg> for ExpanderData
{
    fn update(&mut self, msg: Msg) {
        match msg.clone() {
            Msg::ToggleExpand => {
                self.toggle_expanded();
            },
        }
    }
}
