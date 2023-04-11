use egg::Language;

pub enum RVSDG {
    RVSDG(Option<Vec<RVSDG>>),
    Node(
        Option<Input>,      // in,
        Option<Output>,     // out
        Option<String>,     // id
        Option<String>,     // name
        Option<String>,     // type
        Option<Vec<RVSDG>>, // body
    ),
    Region(Option<Vec<RVSDG>>, Option<Result>),
    Edge(String, String), // source and target id
    Result(String),       // id
    Input(String),        // id
    Output(String),       // id
}

// TODO
impl egg::Language for RVSDG {
    fn matches(&self, other: &Self) -> bool {
        unimplemented!()
    }
    fn children(&self) -> &[egg::Id] {
        unimplemented!()
    }
    fn children_mut(&mut self) -> &mut [egg::Id] {
        unimplemented!()
    }
}
