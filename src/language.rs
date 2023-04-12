use std::fs::File;

use egg::Language;

use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(Debug, Clone, Hash)]
struct Attributes {
    id: String,
    name: Option<String>,
    ty: Option<String>, // type
}

impl From<&Vec<OwnedAttribute>> for Attributes {
    fn from(attrs: &Vec<OwnedAttribute>) -> Self {
        let res: Attributes;
        res.name = None;
        res.ty = None;

        for attr in attrs {
            match attr.name.local_name.as_str() {
                "id" => res.id = attr.value,
                "name" => res.name = Some(attr.value),
                "type" => res.ty = Some(attr.value),
            }
        }
        res
    }
}

#[derive(Debug, Clone, Hash)]
pub enum RVSDG {
    Rvsdg(Option<Vec<RVSDG>>),
    Node {
        attr: Attributes,
        input: Option<Vec<RVSDG>>,  // in,
        output: Option<Vec<RVSDG>>, // out
        body: Option<Vec<RVSDG>>,   // body
    },
    Region(String, Option<Vec<RVSDG>>, Option<Box<RVSDG>>), // id, body, result

    Edge(String, String), // source and target id
    Result(String),       // id
    Input(String),        // id
    Output(String),       // id
    Argument(String),     // id
}

// TODO
// impl egg::Language for RVSDG {
// fn matches(&self, other: &Self) -> bool { unimplemented!() }
// fn children(&self) -> &[egg::Id] { unimplemented!() }
// fn children_mut(&mut self) -> &mut [egg::Id] { unimplemented!() }
// }

impl RVSDG {
    fn parse_rvsdg(reader: &mut EventReader<File>) -> Self {
        let body: Vec<RVSDG> = Vec::new();
        loop {
            let e = reader.next();
            if let Ok(XmlEvent::EndElement {
                name: OwnedName { local_name, .. },
            }) = e
            {
                if local_name == "rvsdg" {
                    return RVSDG::Rvsdg(Some(body));
                }
            } else {
                body.push(RVSDG::parse_elem(&e.unwrap(), reader).unwrap());
            }
        }
    }

    fn parse_node(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {}
    fn parse_region(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {}

    fn parse_edge(reader: &mut EventReader<File>, attributes: &Vec<OwnedAttribute>) -> Self {
        let e = reader.next();
        if let Ok(XmlEvent::EndElement {
            name: OwnedName { local_name, .. },
        }) = e
        {
            if local_name == "edge" {
                let target: String;
                let src: String;
                for a in attributes {
                    if a.name.local_name == "target" {
                        target = a.value;
                    } else if a.name.local_name == "source" {
                        src = a.value;
                    }
                }
                return RVSDG::Edge(src, target);
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
                };
            }
        }
        unreachable!()
    }

    fn parse_elem(element: &XmlEvent, reader: &mut EventReader<File>) -> Result<Self, String> {
        match element {
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
            },
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn parse(reader: &mut EventReader<File>) -> Result<Self, String> {
        RVSDG::parse_elem(&reader.next().unwrap(), reader)
    }
}
