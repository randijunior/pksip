use crate::{Endpoint};

pub mod dialogs;
pub mod invite;

pub use dialogs::*;


pub struct Ua {
    dialogs: Dialogs,
    endpoint: Endpoint
}


impl Ua {
    pub fn new(endpoint: Endpoint) -> Self {
        Ua {
            dialogs: Dialogs::new(),
            endpoint
        }
    }

    pub fn register_dialog(&self, dialog: Dialog) {
        self.dialogs.insert(dialog);
    }
    
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}