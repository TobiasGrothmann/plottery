#[derive(Debug, Clone)]
pub enum RunningState {
    Idle,
    Preparing { msg: String },
    StartingBuild { msg: String },
    Building { msg: String },
    BuildFailed { msg: String },
    StartingGetParams { msg: String },
    GetParams { msg: String },
    GetParamsFailed { msg: String },
    StartingRun { msg: String },
    Running { msg: String },
    RunFailed { msg: String },
    Updating { msg: String },
}
impl RunningState {
    pub fn is_busy(&self) -> bool {
        !matches!(
            self,
            RunningState::Idle
                | RunningState::BuildFailed { .. }
                | RunningState::RunFailed { .. }
                | RunningState::GetParamsFailed { .. }
        )
    }
    pub fn get_msg(&self) -> String {
        match self {
            RunningState::Idle => "".to_string(),
            RunningState::Preparing { msg } => msg.clone(),
            RunningState::StartingBuild { msg } => msg.clone(),
            RunningState::Building { msg } => msg.clone(),
            RunningState::BuildFailed { msg } => msg.clone(),
            RunningState::StartingRun { msg } => msg.clone(),
            RunningState::Running { msg } => msg.clone(),
            RunningState::RunFailed { msg } => msg.clone(),
            RunningState::Updating { msg } => msg.clone(),
            RunningState::StartingGetParams { msg } => msg.clone(),
            RunningState::GetParams { msg } => msg.clone(),
            RunningState::GetParamsFailed { msg } => msg.clone(),
        }
    }
}
