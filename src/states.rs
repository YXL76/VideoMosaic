use std::{lazy::SyncLazy, path::PathBuf, sync::RwLock};

#[derive(Default)]
pub struct State {
    pub target_type: TargetType,
    pub target_path: PathBuf,
}

pub static STATE: SyncLazy<RwLock<State>> = SyncLazy::new(|| RwLock::new(State::default()));

#[derive(Debug, Clone, Copy)]
pub enum TargetType {
    None,
    Image,
    Video,
}

impl Default for TargetType {
    fn default() -> Self {
        TargetType::None
    }
}
