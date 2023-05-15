use std::{collections::HashMap, fmt::Debug, fs::File, hash::Hash};

use egg::{define_language, EGraph, Id, Language};
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Attributes {
    id: String,
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
            id: map["id"].as_str().to_owned(),
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
define_language! {
    pub enum RVSDG {
        "rvsdg" = Rvsdg(Box<[Id]>),
        "region" = Region(Box<[Id]>),
        "node" = Node(Box<[Id]>),
        "edge" = Edge([Id; 2]),
        "result" = Result,
        "input" = Input,
        "output" = Output,
        "arg" = Argument,
        Symbol(egg::Symbol),
    }
}

impl RVSDG {
    fn parse_rvsdg(
        reader: &mut EventReader<File>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
        let mut body: Vec<Id> = Vec::new();
        loop {
            let e = reader.next();
            match e {
                Ok(XmlEvent::EndElement {
                    name: OwnedName { local_name, .. },
                }) => {
                    if local_name == "rvsdg" {
                        let res: Box<[Id]> = body.clone().into();
                        let expr = RVSDG::Rvsdg(res);
                        egraph.add(expr.clone());
                    }
                }
                Ok(XmlEvent::Whitespace(_)) => {
                    continue;
                }
                Ok(elem) => {
                    body.push(RVSDG::parse_elem(&elem, reader, egraph, id_table));
                }
                Err(_) => panic!("Error while parsing `RVSDG`"),
            }
        }
    }

    fn parse_node(
        reader: &mut EventReader<File>,
        attributes: &Vec<OwnedAttribute>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
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
                    body.push(RVSDG::parse_elem(&elem, reader, egraph, id_table));
                }
                Err(_) => panic!("Error while parsing `Node`"),
            }
        }
        println!("RecExpr: {:?}", egraph);
        let body: Box<[Id]> = body.into();
        let expr = RVSDG::Node(body);
        println!("Node children: {:?}", expr.children());
        let id = egraph.add(expr.clone());
        id_table.insert(attr.id, id);
        id
    }
    fn parse_region(
        reader: &mut EventReader<File>,
        attributes: &Vec<OwnedAttribute>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
        let attr = Attributes::from(attributes);
        let mut body: Vec<Id> = Vec::new();
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
                    body.push(RVSDG::parse_elem(&elem, reader, egraph, id_table));
                }
                Err(_) => panic!("Error while parsing `Region`"),
            }
        }
        let body: Box<[Id]> = body.into();
        let expr = RVSDG::Region(body);
        let id = egraph.add(expr.clone());
        id_table.insert(attr.id, id);
        id
    }

    fn parse_edge(
        reader: &mut EventReader<File>,
        attributes: &Vec<OwnedAttribute>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
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
                let expr = RVSDG::Edge([id_table[&src], id_table[&target]]);
                return egraph.add(expr.clone());
            }
        }
        unreachable!()
    }

    fn parse_atom(
        tag: &str,
        reader: &mut EventReader<File>,
        attributes: &Vec<OwnedAttribute>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
        let e = reader.next();
        if let Ok(XmlEvent::EndElement {
            name: OwnedName { local_name, .. },
        }) = e
        {
            if local_name == tag {
                let expr = match tag {
                    "result" => RVSDG::Result,
                    "input" => RVSDG::Input,
                    "output" => RVSDG::Output,
                    "argument" => RVSDG::Argument,
                    _ => {
                        unreachable!()
                    }
                };
                let id = egraph.add(expr.clone());
                id_table.insert(Attributes::from(attributes).id, id);
                return id;
            }
        }
        unreachable!()
    }

    fn parse_elem(
        element: &XmlEvent,
        reader: &mut EventReader<File>,
        egraph: &mut EGraph<Self, ()>,
        id_table: &mut HashMap<String, Id>,
    ) -> Id {
        match element {
            XmlEvent::StartDocument { .. } => {
                RVSDG::parse_elem(&reader.next().unwrap(), reader, egraph, id_table)
            }
            XmlEvent::Whitespace(_) => {
                RVSDG::parse_elem(&reader.next().unwrap(), reader, egraph, id_table)
            }
            XmlEvent::StartElement {
                name: OwnedName { local_name, .. },
                attributes,
                ..
            } => match local_name.as_str() {
                "rvsdg" => RVSDG::parse_rvsdg(reader, egraph, id_table),
                "node" => RVSDG::parse_node(reader, &attributes, egraph, id_table),
                "region" => RVSDG::parse_region(reader, &attributes, egraph, id_table),
                "edge" => RVSDG::parse_edge(reader, &attributes, egraph, id_table),
                "result" | "input" | "output" | "argument" => {
                    RVSDG::parse_atom(&local_name, reader, &attributes, egraph, id_table)
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

    pub fn parse(reader: &mut EventReader<File>) -> Result<EGraph<Self, ()>, String> {
        let mut egraph: EGraph<Self, ()> = Default::default();
        let mut id_table: HashMap<String, Id> = HashMap::new();
        let _ = RVSDG::parse_elem(&reader.next().unwrap(), reader, &mut egraph, &mut id_table);
        egraph.rebuild();
        Ok(egraph)
    }
}
