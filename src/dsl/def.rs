use decimal::{d128};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeKind {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ins: Vec<InputKind>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outs: Vec<OutputKind>,
}

impl NodeKind {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().into(),
            info: None,
            ins: Vec::new(),
            outs: Vec::new(),
        }
    }

    pub fn get_in<S: AsRef<str>>(&self, name: S) -> Option<&InputKind> {
        self.ins.iter().find(|&input| input.name == name.as_ref())
    }

    pub fn get_out<S: AsRef<str>>(&self, name: S) -> Option<&OutputKind> {
        self.outs.iter().find(|&output| output.name == name.as_ref())
    }

    pub fn with_info<S: AsRef<str>>(mut self, info: S) -> Self {
        self.info = Some(info.as_ref().into());
        self
    }

    pub fn with_in(mut self, input: InputKind) -> Self {
        self.ins.push(input);
        self
    }

    pub fn with_out(mut self, output: OutputKind) -> Self {
        self.outs.push(output);
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputKind {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

impl InputKind {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().into(),
            info: None,
        }
    }

    pub fn with_info<S: AsRef<str>>(mut self, info: S) -> Self {
        self.info = Some(info.as_ref().into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputKind {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

impl OutputKind {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().into(),
            info: None,
        }
    }

    pub fn with_info<S: AsRef<str>>(mut self, info: S) -> Self {
        self.info = Some(info.as_ref().into());
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub kind: String,
    pub name: String,
    
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ins: Vec<Input>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outs: Vec<Output>,
}

impl Node {
    pub fn get_in<S: AsRef<str>>(&self, name: S) -> Option<&Input> {
        self.ins.iter().find(|&input| input.name == name.as_ref())
    }

    pub fn get_out<S: AsRef<str>>(&self, name: S) -> Option<&Output> {
        self.outs.iter().find(|&output| output.name == name.as_ref())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,

    //#[serde(default)]
    //#[serde(skip_serializing_if = "Option::is_none")]
    pub link: Link,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum Link {
    Output {
        /// Linked node name
        node: String,
        /// Linked output name
        out: String,
    },
    Ctrl {
        /// Linked control name
        name: String,
    }
}

impl Link {
    pub fn output<S: Into<String>>(node: S, out: S) -> Self {
        Link::Output { node: node.into(), out: out.into() }
    }
    
    pub fn ctrl<S: Into<String>>(name: S) -> Self {
        Link::Ctrl { name: name.into() }
    }
}

pub type Value = d128;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ctrl {
    pub name: String,

    pub value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    /// nodes
    pub nodes: Vec<Node>,

    /// controls
    pub ctrls: Vec<Ctrl>,
}

impl Mesh {
    pub fn get_node<S: AsRef<str>>(&self, name: S) -> Option<&Node> {
        self.nodes.iter().find(|n| n.name == name.as_ref())
    }

    pub fn get_ctrl<S: AsRef<str>>(&self, name: S) -> Option<&Ctrl> {
        self.ctrls.iter().find(|c| c.name == name.as_ref())
    }
}

#[cfg(test)]
mod test {
    use super::{NodeKind, InputKind, OutputKind};
    use serde_json::{to_string};

    #[test]
    fn test_adder_kind() {
        let adder_kind = NodeKind::new("+")
            .with_in(InputKind::new("a"))
            .with_in(InputKind::new("b"))
            .with_out(OutputKind::new("="));

        assert_eq!(to_string(&adder_kind).unwrap(), String::from("{\"name\":\"+\",\"ins\":[{\"name\":\"a\"},{\"name\":\"b\"}],\"outs\":[{\"name\":\"=\"}]}"));
    }
}
