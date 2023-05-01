use std::{collections::HashMap, fmt::Debug, fs::File, hash::Hash};

use crate::language::*;

//use egg::Language;

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Attributes<Id: std::str::FromStr> {
    id: Id,
    name: Option<String>,
    ty: Option<String>, // type
}

impl<Id: std::str::FromStr + Debug> From<&Vec<OwnedAttribute>> for Attributes<Id>
where
    <Id as std::str::FromStr>::Err: Debug,
{
    fn from(attrs: &Vec<OwnedAttribute>) -> Self {
        let mut map = HashMap::<&str, String>::new();

        for attr in attrs {
            map.insert(attr.name.local_name.as_str(), attr.value.clone());
        }
        Attributes {
            id: Id::from_str(map["id"].as_str()).unwrap(),
            name: if map.contains_key("name") {
                Some(map["name"].clone())
            } else {
                None
            },
            ty: if map.contains_key("type") {
                Some(map["type"].clone())
            } else {
                None
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum RVSDG<Id: std::str::FromStr + Debug> {
    Rvsdg(Option<Vec<RVSDG<Id>>>),
    Node {
        attr: Attributes<Id>,
        body: Option<Vec<RVSDG<Id>>>, // body
    },
    Region(Id, Option<Vec<RVSDG<Id>>>, Option<Vec<RVSDG<Id>>>), // id, body, results

    Edge(Id, Id), // source and target id
    Result(Id),   // id
    Input(Id),    // id
    Output(Id),   // id
    Argument(Id), // id
}

// TODO
impl egg::Language for RVSDG<EggIdWrapper> {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rvsdg(_), _) => false,
            (
                Self::Node { attr, .. },
                Self::Node {
                    attr: other_attr, ..
                },
            ) => attr.id.id == other_attr.id.id,
            (Self::Region(id, _, _), Self::Region(other_id, _, _)) => id.id == other_id.id,
            (Self::Edge(src, target), Self::Edge(other_src, other_target)) => {
                src.id == other_src.id && target.id == other_target.id
            }
            (Self::Result(id), Self::Result(other_id)) => id.id == other_id.id,
            (Self::Input(id), Self::Input(other_id)) => id.id == other_id.id,
            (Self::Output(id), Self::Output(other_id)) => id.id == other_id.id,
            (Self::Argument(id), Self::Argument(other_id)) => id.id == other_id.id,
            _ => false,
        }
    }
    fn children(&self) -> &[egg::Id] {
        todo!()
        // let mut res: Vec<egg::Id> = Vec::new();
        // match self {
        //     Self::Rvsdg(Some(b))
        //     | Self::Region(_, Some(b), _) => b.map(|e| e.get_id()),
        //     _ => unreachable!(),
        // }
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        todo!()
    }
}

impl<Id: std::str::FromStr + Debug> RVSDG<Id>
where
    <Id as std::str::FromStr>::Err: Debug,
{
    fn get_id(&self) -> Option<&Id> {
        match self {
            Self::Node { attr, body } => Some(&attr.id),
            Self::Region(id, _, _)
            | Self::Input(id)
            | Self::Output(id)
            | Self::Result(id)
            | Self::Argument(id) => Some(id),
            _ => None,
        }
    }

    fn parse_rvsdg(reader: &mut EventReader<File>) -> Self {
        let mut body: Vec<RVSDG<Id>> = Vec::new();
        loop {
            let e = reader.next();
            match e {
                Ok(XmlEvent::EndElement {
                    name: OwnedName { local_name, .. },
                }) => {
                    if local_name == "rvsdg" {
                        return RVSDG::Rvsdg(Some(body));
                    }
                }
                Ok(XmlEvent::Whitespace(_)) => {
                    continue;
                }
                Ok(elem) => {
                    body.push(RVSDG::parse_elem(&elem, reader).unwrap());
                }
                Err(_) => panic!("Error while parsing `RVSDG`"),
            }
        }
    }

    fn parse_node(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let attr = Attributes::<Id>::from(attributes);
        let mut body: Vec<RVSDG<Id>> = Vec::new();
        loop {
            let e = reader.next();
            match e {
                Ok(XmlEvent::EndElement {
                    name: OwnedName { local_name, .. },
                }) => {
                    if local_name == "node" {
                        break;
                    }
                }
                Ok(XmlEvent::Whitespace(_)) => {
                    continue;
                }
                Ok(elem) => {
                    body.push(RVSDG::parse_elem(&elem, reader).unwrap());
                }
                Err(_) => panic!("Error while parsing `Node`"),
            }
        }
        RVSDG::Node {
            attr,
            body: if body.is_empty() { None } else { Some(body) },
        }
    }
    fn parse_region(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let attr = Attributes::from(attributes);
        let mut rest: Vec<RVSDG<Id>> = Vec::new();
        loop {
            let e = reader.next();
            match e {
                Ok(XmlEvent::EndElement {
                    name: OwnedName { local_name, .. },
                }) => {
                    if local_name == "region" {
                        break;
                    }
                }
                Ok(XmlEvent::Whitespace(_)) => {
                    continue;
                }
                Ok(elem) => {
                    rest.push(RVSDG::parse_elem(&elem, reader).unwrap());
                }
                Err(_) => panic!("Error while parsing `Region`"),
            }
        }
        let mut results: Vec<RVSDG<Id>> = Vec::new();
        let mut body: Vec<RVSDG<Id>> = Vec::new();
        for e in rest {
            match e {
                RVSDG::Result(_) => results.push(e),
                _ => body.push(e),
            }
        }
        RVSDG::Region(
            attr.id,
            if body.is_empty() { None } else { Some(body) },
            if results.is_empty() {
                None
            } else {
                Some(results)
            },
        )
    }

    fn parse_edge(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let e = reader.next();
        if let Ok(XmlEvent::EndElement {
            name: OwnedName { local_name, .. },
        }) = e
        {
            if local_name == "edge" {
                let mut target = String::new();
                let mut src = String::new();
                for a in attributes {
                    if a.name.local_name == "target" {
                        target = a.value.clone();
                    } else if a.name.local_name == "source" {
                        src = a.value.clone();
                    }
                }
                return RVSDG::Edge(Id::from_str(&src).unwrap(), Id::from_str(&target).unwrap());
            }
        }
        unreachable!()
    }

    fn parse_atom(
        tag: &str,
        reader: &mut EventReader<File>,
        attributes: &Vec<OwnedAttribute>,
    ) -> Self {
        let e = reader.next();
        if let Ok(XmlEvent::EndElement {
            name: OwnedName { local_name, .. },
        }) = e
        {
            if local_name == tag {
                return match tag {
                    "result" => RVSDG::Result(Attributes::from(attributes).id),
                    "input" => RVSDG::Input(Attributes::from(attributes).id),
                    "output" => RVSDG::Output(Attributes::from(attributes).id),
                    "argument" => RVSDG::Argument(Attributes::from(attributes).id),
                    _ => {
                        unreachable!()
                    }
                };
            }
        }
        unreachable!()
    }

    fn parse_elem(element: &XmlEvent, reader: &mut EventReader<File>) -> Result<Self, String> {
        match element {
            XmlEvent::StartDocument { .. } => RVSDG::parse_elem(&reader.next().unwrap(), reader),
            XmlEvent::Whitespace(_) => RVSDG::parse_elem(&reader.next().unwrap(), reader),
            XmlEvent::StartElement {
                name: OwnedName { local_name, .. },
                attributes,
                ..
            } => match local_name.as_str() {
                "rvsdg" => Ok(RVSDG::parse_rvsdg(reader)),
                "node" => Ok(RVSDG::parse_node(reader, &attributes)),
                "region" => Ok(RVSDG::parse_region(reader, &attributes)),
                "edge" => Ok(RVSDG::parse_edge(reader, &attributes)),
                "result" | "input" | "output" | "argument" => {
                    Ok(RVSDG::parse_atom(&local_name, reader, &attributes))
                }
                _ => {
                    unreachable!()
                }
            },
            e => {
                println!("{:?}", e);
                unimplemented!()
            }
        }
    }

    pub fn parse(reader: &mut EventReader<File>) -> Result<Self, String> {
        RVSDG::parse_elem(&reader.next().unwrap(), reader)
    }
}
