use std::{collections::{HashMap, hash_map::DefaultHasher}, fmt::Debug, fs::File, hash::{Hash, Hasher}};

use crate::language::*;

use egg::{Id, Language};
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Attributes {
    id: Id,
    name: Option<String>,
    ty: Option<String>, // type
}

impl From<&Vec<OwnedAttribute>> for Attributes {
    fn from(attrs: &Vec<OwnedAttribute>) -> Self {
        let mut map = HashMap::<&str, String>::new();

        for attr in attrs {
            map.insert(attr.name.local_name.as_str(), attr.value.clone());
        }
        Attributes {
            id: egg_id_from_str(map["id"].as_str()).unwrap(),
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
pub enum RVSDG {
    Rvsdg(Option<Box<[Id]>>),
    Node {
        attr: Attributes,
        body: Option<Box<[Id]>>, // body
    },
    Region(Id, Option<Box<[Id]>>, Option<Box<[Id]>>), // id, body, results

    Edge(Id, Id, Id), // edge, source, and target id
    Result(Id),   // id
    Input(Id),    // id
    Output(Id),   // id
    Argument(Id), // id
}

// TODO
impl egg::Language for RVSDG {
    fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Rvsdg(_), _) => false,
            (
                Self::Node { attr, .. },
                Self::Node {
                    attr: other_attr, ..
                },
            ) => attr.id == other_attr.id,
            (Self::Region(id, _, _), Self::Region(other_id, _, _)) => id == other_id,
            (Self::Edge(id, src, target), Self::Edge(other_id, other_src, other_target)) => {
                id == other_id && src == other_src && target == other_target
            }
            (Self::Result(id), Self::Result(other_id)) => id == other_id,
            (Self::Input(id), Self::Input(other_id)) => id == other_id,
            (Self::Output(id), Self::Output(other_id)) => id == other_id,
            (Self::Argument(id), Self::Argument(other_id)) => id == other_id,
            _ => false,
        }
    }
    fn children(&self) -> &[egg::Id] {
        match self {
            Self::Rvsdg(Some(b)) | Self::Region(_, Some(b), _) => &*b,
            _ => &[],
        }
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        match self {
            Self::Rvsdg(Some(b)) | Self::Region(_, Some(b), _) => &mut *b,
            _ => &mut [],
        }
    }
}

impl RVSDG {
    fn get_id(&self) -> Option<&Id> {
        match self {
            Self::Node { attr, body } => Some(&attr.id),
            Self::Region(id, _, _)
            | Self::Input(id)
            | Self::Output(id)
            | Self::Result(id)
            | Self::Argument(id) 
            | Self::Edge(id, _, _)=> Some(id),
            _ => None,
        }
    }

    // this is stupid
    fn gen_edge_id(src: &String, target: &String) -> Id {
        let mut hasher = DefaultHasher::new();
        (src.to_owned() + target).hash(&mut hasher);
        let res = hasher.finish() as usize;
        res.into()
    }

    fn parse_rvsdg(reader: &mut EventReader<File>) -> Self {
        let mut body: Vec<Id> = Vec::new();
        loop {
            let e = reader.next();
            match e {
                Ok(XmlEvent::EndElement {
                    name: OwnedName { local_name, .. },
                }) => {
                    if local_name == "rvsdg" {
                        let res: Box<[Id]> = body.into();
                        return RVSDG::Rvsdg(Some(res));
                    }
                }
                Ok(XmlEvent::Whitespace(_)) => {
                    continue;
                }
                Ok(elem) => {
                    body.push(*RVSDG::parse_elem(&elem, reader).unwrap().get_id().unwrap());
                }
                Err(_) => panic!("Error while parsing `RVSDG`"),
            }
        }
    }

    fn parse_node(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let attr = Attributes::from(attributes);
        let mut body: Vec<Id> = Vec::new();
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
                    body.push(*RVSDG::parse_elem(&elem, reader).unwrap().get_id().unwrap());
                }
                Err(_) => panic!("Error while parsing `Node`"),
            }
        }
        let body: Box<[Id]> = body.into();
        RVSDG::Node {
            attr,
            body: if body.is_empty() { None } else { Some(body) },
        }
    }
    fn parse_region(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let attr = Attributes::from(attributes);
        let mut rest: Vec<RVSDG> = Vec::new();
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
        let mut results: Vec<Id> = Vec::new();
        let mut body: Vec<Id> = Vec::new();
        for e in rest {
            match e {
                RVSDG::Result(_) => results.push(*e.get_id().unwrap()),
                _ => body.push(*e.get_id().unwrap()),
            }
        }
        let body: Box<[Id]> = body.into();
        let results: Box<[Id]> = results.into();
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
                return RVSDG::Edge(
                    Self::gen_edge_id(&src, &target),
                    egg_id_from_str(&src).unwrap(),
                    egg_id_from_str(&target).unwrap(),
                );
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
