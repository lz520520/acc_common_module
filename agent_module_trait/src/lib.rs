pub mod params;
mod helper_macro;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::params::ParamValue;

pub static RAW_RESULT: Lazy<String> = Lazy::new(|| {obfstr::obfstring!("raw_result")});
pub static RAW_ERROR: Lazy<String> = Lazy::new(|| {obfstr::obfstring!("raw_error")});

pub type AccModules = HashMap<u64, Arc<dyn AgentModule>>;

#[repr(u8)]
pub enum TaskStatus {
    TaskStart = 1,
    TaskOver = 2,
    TaskProcess = 3,
}
impl TryFrom<u8> for TaskStatus {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(TaskStatus::TaskStart),
            2 => Ok(TaskStatus::TaskOver),
            3 => Ok(TaskStatus::TaskProcess),
            _ => Err(obfstr::obfstring!("Unknown TaskStatus")),
        }
    }
}

pub struct AgentModuleParams {
    pub input: Arc<dyn AgentModuleInput>,
    pub info: Arc<dyn AgentModuleInfo>,
    pub output: Arc<dyn AgentModuleOutput>,
}

pub type AgentModuleResult<T> = Result<T, Box<dyn Error>>;

pub type ParamMeta = HashMap<String, ParamValue>;



pub trait AgentModuleInput: Send + Sync {
    fn new_instance(&self, key: &str)  -> Arc<dyn AgentModuleInput>;
    fn get_bool_param(&self, key: &str) -> bool;
    fn get_bytes_param(&self, key: &str) -> Vec<u8>;
    fn get_string_param(&self, key: &str) -> String;


    fn get_f32_param(&self, key: &str) -> f32;
    fn get_f64_param(&self, key: &str) -> f64;

    fn get_i8_param(&self, key: &str) -> i8;
    fn get_i16_param(&self, key: &str) -> i16;
    fn get_i32_param(&self, key: &str) -> i32;
    fn get_i64_param(&self, key: &str) -> i64;

    fn get_u8_param(&self, key: &str) -> u8;
    fn get_u16_param(&self, key: &str) -> u16;
    fn get_u32_param(&self, key: &str) -> u32;
    fn get_u64_param(&self, key: &str) -> u64;

    fn get_iter_param(&self, key: &str) -> Box<dyn AgentModuleInput>;
    fn get_iter_list_param(&self, key: &str) -> Vec<Box<dyn AgentModuleInput>>;

}

pub trait AgentModuleInfo: Send + Sync {
    fn get_extend_info(&self,data_type: &str, key: &str) -> Option<ParamValue>;
    fn update_extend_info(&self,data_type: &str, key: &str, value: ParamValue) -> AgentModuleResult<()>;
}
pub trait AgentModuleOutput: Send + Sync  {
    fn send(&self, task_status: Option<TaskStatus>, sub_task_type: Option<u64>, meta: ParamMeta);
    fn allow_close(&self, allow: bool);

    fn log_info(&self, msg: &str);
    fn log_success(&self, msg: &str);
    fn log_error(&self, msg: &str);
    fn log_warning(&self, msg: &str);

    fn log_data(&self, data: Vec<u8>);
    fn log_big_data(&self, data: Vec<u8>);

}

pub trait AgentModule: Send + Sync + AgentModuleTask {
    fn new() -> Self where Self: Sized;
    fn new_instance(&self) -> Arc<dyn AgentModule>;

    fn clear_cache(&self, tag: &str);
    fn close(&self) -> AgentModuleResult<()>;
    fn is_channel(&self) -> bool;
    fn to_string(&self) -> String;
}


pub trait  AgentModuleTask: Send + Sync {
    fn run(self: Arc<Self>, params: AgentModuleParams) -> AgentModuleResult<()>;
}

