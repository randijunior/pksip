

mod dialog;

pub use dialog::*;
use std::{collections::HashMap, sync::Mutex};

pub struct Dialogs {
    dialogs: Mutex<HashMap<DialogID, Dialog>>,
}

impl Dialogs {
    pub fn new() -> Self {
        Dialogs {
            dialogs: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn insert(&self, dialog: Dialog) {
        self.dialogs.lock().unwrap().insert(dialog.id().clone(), dialog);
    }
}

