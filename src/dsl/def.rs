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

pub type NodeKinds = Vec<NodeKind>;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<Link>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub name: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Link {
    /// Linked node name
    pub node: String,
    /// Linked out name
    pub out: String,
}

pub type Nodes = Vec<Node>;

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
