use crate::{
    find_in_xml,
    utils::{self, HttpResponseExt},
    Error,
};

use http::Uri;
use roxmltree::{Document, Node};
use ssdp_client::URN;
use std::rc::Rc;

mod action;
mod state_variable;
pub use action::*;
pub use state_variable::*;

/// Service Control Protocol Description.
/// It contains information about a particular service, more specifically its actions and state
/// variables.
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

    /// Fetches the SCPD description.
    /// The `urn` has to be provided because it isn't included in the description.
    pub async fn from_url(url: &Uri, urn: URN) -> Result<Self, Error> {
        let body = isahc::get_async(url)
            .await?
            .err_if_not_200()?
            .body_mut()
            .text_async()
            .await?;

        let document = Document::parse(&body)?;
        let scpd = utils::find_root(&document, "scpd", "Service Control Point Definition")?;

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
