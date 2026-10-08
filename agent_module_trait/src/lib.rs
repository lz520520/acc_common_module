mod helper_macro;
pub mod params;
mod task_workers;

pub use task_workers::TaskWorkers;

use crate::params::ParamValue;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

pub static RAW_RESULT: Lazy<String> = Lazy::new(|| obfstr::obfstring!("raw_result"));
pub static RAW_ERROR: Lazy<String> = Lazy::new(|| obfstr::obfstring!("raw_error"));

pub type AccModules = HashMap<u64, Arc<dyn AgentModule>>;

pub const MODULE_ABI_VERSION: u32 = 2;

#[macro_export]
macro_rules! export_agent_module_abi {
    () => {
        #[no_mangle]
        pub extern "C" fn AccModuleAbiVersion() -> u32 {
            agent_module_trait::MODULE_ABI_VERSION
        }
    };
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub request: TaskRequest,
    pub info: Arc<dyn AgentModuleInfo>,
    pub output: Arc<dyn AgentModuleOutput>,
}

pub struct TaskRequest {
    pub header: Arc<dyn AgentModuleInput>,
    pub input: Arc<dyn AgentModuleInput>,
}

impl TaskRequest {
    pub fn status(&self) -> AgentModuleResult<TaskStatus> {
        let value: u8 = self
            .header
            .required_u64(obfstr::obfstr!("task_status"))?
            .try_into()?;
        Ok(value.try_into()?)
    }
}

impl std::ops::Deref for TaskRequest {
    type Target = dyn AgentModuleInput;

    fn deref(&self) -> &Self::Target {
        self.input.as_ref()
    }
}

impl std::ops::Deref for AgentModuleParams {
    type Target = TaskRequest;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}

pub type AgentModuleResult<T> = Result<T, Box<dyn Error>>;

pub type ParamMeta = HashMap<String, ParamValue>;

pub trait AgentModuleInput: Send + Sync {
    fn get_value(&self, key: &str) -> Option<ParamValue>;

    fn required_u64(&self, key: &str) -> AgentModuleResult<u64> {
        match self.get_value(key) {
            Some(ParamValue::Uint64(value)) => Ok(value),
            Some(ParamValue::Uint32(value)) => Ok(value.into()),
            Some(ParamValue::Uint16(value)) => Ok(value.into()),
            Some(ParamValue::Uint8(value)) => Ok(value.into()),
            Some(ParamValue::Int64(value)) => Ok(value.try_into()?),
            Some(ParamValue::Int32(value)) => Ok(value.try_into()?),
            Some(ParamValue::Int16(value)) => Ok(value.try_into()?),
            Some(ParamValue::Int8(value)) => Ok(value.try_into()?),
            Some(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{}: {}", key, obfstr::obfstr!("expected integer")),
            )
            .into()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{}: {}",
                    key,
                    obfstr::obfstr!("required parameter is missing")
                ),
            )
            .into()),
        }
    }

    fn optional_u64(&self, key: &str, default: u64) -> AgentModuleResult<u64> {
        if self.get_value(key).is_none() {
            Ok(default)
        } else {
            self.required_u64(key)
        }
    }

    fn required_u16(&self, key: &str) -> AgentModuleResult<u16> {
        Ok(self.required_u64(key)?.try_into()?)
    }

    fn required_string(&self, key: &str) -> AgentModuleResult<String> {
        match self.get_value(key) {
            Some(ParamValue::Str(value)) => Ok(value),
            Some(ParamValue::Bytes(value)) => Ok(String::from_utf8(value)?),
            Some(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{}: {}", key, obfstr::obfstr!("expected string")),
            )
            .into()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{}: {}",
                    key,
                    obfstr::obfstr!("required parameter is missing")
                ),
            )
            .into()),
        }
    }

    fn optional_string(&self, key: &str, default: &str) -> AgentModuleResult<String> {
        if self.get_value(key).is_none() {
            Ok(default.to_owned())
        } else {
            self.required_string(key)
        }
    }

    fn required_bool(&self, key: &str) -> AgentModuleResult<bool> {
        match self.get_value(key) {
            Some(ParamValue::Bool(value)) => Ok(value),
            Some(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{}: {}", key, obfstr::obfstr!("expected bool")),
            )
            .into()),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{}: {}",
                    key,
                    obfstr::obfstr!("required parameter is missing")
                ),
            )
            .into()),
        }
    }

    fn new_instance(&self, key: &str) -> Arc<dyn AgentModuleInput>;
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
    fn get_extend_info(&self, data_type: &str, key: &str) -> Option<ParamValue>;
    fn update_extend_info(
        &self,
        data_type: &str,
        key: &str,
        value: ParamValue,
    ) -> AgentModuleResult<()>;
    fn stop_module_tasks(
        &self,
        _module_ids: &[u64],
        _deadline: Instant,
    ) -> AgentModuleResult<bool> {
        Ok(false)
    }
}
pub trait AgentModuleOutput: Send + Sync {
    fn send(
        &self,
        task_status: Option<TaskStatus>,
        sub_task_type: Option<u64>,
        meta: ParamMeta,
    ) -> AgentModuleResult<()>;

    fn log_info(&self, msg: &str);
    fn log_success(&self, msg: &str);
    fn log_error(&self, msg: &str);
    fn log_warning(&self, msg: &str);

    fn log_data(&self, data: Vec<u8>);
    fn log_big_data(&self, data: Vec<u8>);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    Terminal,
    Remote,
    Deleted,
    Offline,
    Unload,
    Failed,
}

pub trait AgentModule: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn create_task(&self) -> Arc<dyn AgentModuleTask>;
    fn is_channel(&self) -> bool {
        false
    }
}

pub trait AgentModuleTask: Send + Sync {
    fn run(self: Arc<Self>, params: AgentModuleParams) -> AgentModuleResult<()>;
    fn request_stop(&self, _reason: StopReason) -> AgentModuleResult<()> {
        Ok(())
    }
    fn wait_stopped(&self, _deadline: Instant) -> AgentModuleResult<bool> {
        Ok(true)
    }
    fn clear_cache(&self, _tag: &str) {}
    fn to_string(&self) -> String {
        obfstr::obfstring!("")
    }
}
