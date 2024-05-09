use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;

#[derive(Default)]
pub struct AcidSync(pub Arc<Mutex<HashMap<String, Arc<RwLock<()>>>>>);

impl AcidSync {
    pub fn get_rw_lock(&self, table_name: String) -> Arc<RwLock<()>> {
        let mut sync_guard = self.0.lock().unwrap();
        Arc::clone(
            sync_guard
                .entry(table_name)
                .or_insert_with(|| Arc::new(RwLock::new(())))
        )
    }
}

impl Clone for AcidSync {
    fn clone(&self) -> AcidSync {
        AcidSync(Arc::clone(&self.0))
    }
  }

