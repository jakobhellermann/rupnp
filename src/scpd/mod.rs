use crate::find_in_xml;
use crate::Error;
use crate::HttpResponseExt;
use isahc::http::Uri;
use roxmltree::Document;
use roxmltree::Node;
use ssdp_client::search::URN;
use std::rc::Rc;

mod action;
pub mod datatypes;
mod state_variable;
pub use action::*;
pub use state_variable::*;

#[derive(Debug)]
pub struct SCPD {
    urn: URN,
    state_variables: Vec<Rc<StateVariable>>,
    actions: Vec<Action>,
}
impl SCPD {
    pub fn urn(&self) -> &URN {
        &self.urn
    }
    pub fn state_variables(&self) -> &Vec<Rc<StateVariable>> {
        &self.state_variables
    }
    pub fn actions(&self) -> &Vec<Action> {
        &self.actions
    }

    pub async fn from_url(url: &Uri, urn: URN) -> Result<Self, Error> {
        let body = isahc::get_async(url)
            .await?
            .err_if_not_200()?
            .body_mut()
            .text_async()
            .await?;

        let document = Document::parse(&body)?;
        let scpd = crate::find_root(&document, "scpd", "Service Control Point Definition")?;

        #[allow(non_snake_case)]
        let (state_variables, actions) = find_in_xml! { scpd => serviceStateTable, actionList };

        let state_variables: Vec<_> = state_variables
            .children()
            .filter(Node::is_element)
            .map(StateVariable::from_xml)
            .map(|sv| sv.map(Rc::new))
            .collect::<Result<_, _>>()?;
        let actions = actions
            .children()
            .filter(Node::is_element)
            .map(|node| Action::from_xml(node, &state_variables))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            urn,
            state_variables,
            actions,
        })
    }
}
