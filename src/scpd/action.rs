use crate::Error;
use crate::{find_in_xml, scpd::{StateVariable, DataType}};
use roxmltree::Node;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct Action {
    name: String,
    arguments: Vec<Argument>,
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name())?;

        for e in self.input_arguments().take(1) {
            write!(f, "{}", e)?;
        }
        for e in self.input_arguments().skip(1) {
            write!(f, ", {}", e)?;
        }
        write!(f, ")")?;

        let mut output_arguments = self.output_arguments();
        if let Some(arg) = output_arguments.next() {
            write!(f, " -> ({}", arg)?;
            for arg in output_arguments {
                write!(f, ", {}", arg)?;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl Action {
    pub(crate) fn from_xml(
        node: Node,
        state_variables: &[Rc<StateVariable>],
    ) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (name, arguments) = find_in_xml! { node => name, ?argumentList };

        let arguments = arguments
            .map(|args| {
                args.children()
                    .filter(Node::is_element)
                    .map(|node| Argument::from_xml(node, state_variables))
                    .collect::<Result<_, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            name: crate::parse_node_text(name)?,
            arguments,
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn arguments(&self) -> &Vec<Argument> {
        &self.arguments
    }

    pub fn input_arguments(&self) -> impl Iterator<Item = &Argument> {
        self.arguments.iter().filter(|arg| arg.direction.is_in())
    }
    pub fn output_arguments(&self) -> impl Iterator<Item = &Argument> {
        self.arguments.iter().filter(|arg| arg.direction.is_out())
    }
}

#[derive(Debug)]
pub struct Argument {
    pub name: String,
    pub direction: Direction,
    pub state_var: Rc<StateVariable>,
}
impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", &self.name)?;

        match (self.state_var.datatype(), self.state_var.allowed_values()) {
            (DataType::String, Some(allowed_values)) => write!(f, "[{}]", allowed_values.join(", "))?,
            (datatype, None) => write!(f, "{}", datatype)?,
            _ => unreachable!(),
        }

        if let Some(default) = self.state_var.default_value() {
            write!(f, " = {}", default)?;
        }

        Ok(())
    }
}

impl Argument {
    fn from_xml(node: Node, state_variables: &[Rc<StateVariable>]) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (name, direction, related_statevar) =
            find_in_xml! { node => name, direction, relatedStateVariable };

        let related_statevar = related_statevar
            .text()
            .ok_or_else(|| Error::XMLMissingText("relatedStateVariable".to_string()))?;

        let state_var = state_variables
            .iter()
            .find(|sv| sv.name().eq_ignore_ascii_case(related_statevar))
            .expect("every argument has it's corresponding state variable")
            .clone();

        Ok(Self {
            name: crate::parse_node_text(name)?,
            direction: crate::parse_node_text(direction)?,
            state_var,
        })
    } 
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn related_state_variable(&self) -> &Rc<StateVariable> {
        &self.state_var
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    In,
    Out,
}
impl Direction {
    pub fn is_in(self) -> bool {
        match self {
            Direction::In => true,
            Direction::Out => false,
        }
    }
    pub fn is_out(self) -> bool {
        !self.is_in()
    }
}
#[derive(Debug)]
pub struct ParseDirectionErr(String);
impl std::fmt::Display for ParseDirectionErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid direction: `{}`", &self.0)
    }
}
impl std::error::Error for ParseDirectionErr {}
impl std::str::FromStr for Direction {
    type Err = ParseDirectionErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in" => Ok(Direction::In),
            "out" => Ok(Direction::Out),
            _ => Err(ParseDirectionErr(s.to_string())),
        }
    }
}
