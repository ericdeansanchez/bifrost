use std::fmt::{self, Debug};

/// The information that results from performing Bifrost operations.
pub struct OperationInfo {
    /// The name of the current operable workspace.
    pub name: String,
    /// The number of bytes involved in the operation.
    pub bytes: Option<u64>,
    /// The textual result of performing the given operation.
    pub text: Option<Vec<u8>>,
}

impl OperationInfo {
    pub fn new() -> Self {
        OperationInfo {
            ..Default::default()
        }
    }
}

impl Default for OperationInfo {
    fn default() -> Self {
        OperationInfo {
            name: String::new(),
            bytes: None,
            text: None,
        }
    }
}

impl Debug for OperationInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WorkingDir")
            .field("workspace", &self.name)
            .field("size", &self.bytes)
            .field("text", &self.text)
            .finish()
    }
}
