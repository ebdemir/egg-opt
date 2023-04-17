extern crate xml;

mod language;
use language::*;

// use egg::*;
use std::{env, fs};
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[allow(dead_code)]
fn print_xml(parser: EventReader<fs::File>) {
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                //println!("Start -> Name: {}, Attr: {:?}", name, attributes);

                let attr: Vec<(String, String)> = attributes
                    .into_iter()
                    .map(|attr| match attr {
                        OwnedAttribute {
                            name: OwnedName { local_name, .. },
                            value,
                        } => (local_name, value),
                    })
                    .collect();
                println!("Start -> Name: {}, Attr: {:?}", name, attr);
            }
            Ok(XmlEvent::EndElement { name }) => {
                println!("End: {}", name);
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file = fs::File::open(&args[1]).unwrap();
    let mut parser = EventReader::new(file);
    // let _src_rvsdg: String = fs::read_to_string(&args[1]).unwrap();
    let rvsdg = RVSDG::<String>::parse(&mut parser);

    println!("{:?}", rvsdg);

}
