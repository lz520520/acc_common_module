use crate::params::ParamValue;
use crate::{AgentModuleInput, ParamMeta};
use std::sync::Arc;

/// An owned input map for hosts and tests implementing the Rust module contract.
#[derive(Clone, Default)]
pub struct OwnedInput(pub ParamMeta);

impl OwnedInput {
    fn nested(&self, key: &str) -> ParamMeta {
        match self.get_value(key) {
            Some(ParamValue::Params(meta)) => meta,
            _ => ParamMeta::new(),
        }
    }

    fn value<T>(&self, key: &str) -> Option<T>
    where
        ParamValue: TryInto<T>,
    {
        self.get_value(key).and_then(|value| value.try_into().ok())
    }
}

impl AgentModuleInput for OwnedInput {
    fn get_value(&self, key: &str) -> Option<ParamValue> {
        self.0.get(key).cloned()
    }

    fn snapshot(&self) -> ParamMeta {
        self.0.clone()
    }

    fn new_instance(&self, key: &str) -> Arc<dyn AgentModuleInput> {
        Arc::new(Self(self.nested(key)))
    }

    fn get_iter_param(&self, key: &str) -> Box<dyn AgentModuleInput> {
        Box::new(Self(self.nested(key)))
    }

    fn get_iter_list_param(&self, key: &str) -> Vec<Box<dyn AgentModuleInput>> {
        match self.get_value(key) {
            Some(ParamValue::ParamList(items)) => items
                .into_iter()
                .map(|item| Box::new(Self(item)) as Box<dyn AgentModuleInput>)
                .collect(),
            _ => Vec::new(),
        }
    }

    fn get_bool_param(&self, key: &str) -> bool {
        self.value(key).unwrap_or_default()
    }
    fn get_bytes_param(&self, key: &str) -> Vec<u8> {
        self.value(key).unwrap_or_default()
    }
    fn get_string_param(&self, key: &str) -> String {
        self.value(key).unwrap_or_default()
    }
    fn get_f32_param(&self, key: &str) -> f32 {
        self.value(key).unwrap_or_default()
    }
    fn get_f64_param(&self, key: &str) -> f64 {
        self.value(key).unwrap_or_default()
    }
    fn get_i8_param(&self, key: &str) -> i8 {
        self.value(key).unwrap_or_default()
    }
    fn get_i16_param(&self, key: &str) -> i16 {
        self.value(key).unwrap_or_default()
    }
    fn get_i32_param(&self, key: &str) -> i32 {
        self.value(key).unwrap_or_default()
    }
    fn get_i64_param(&self, key: &str) -> i64 {
        self.value(key).unwrap_or_default()
    }
    fn get_u8_param(&self, key: &str) -> u8 {
        self.value(key).unwrap_or_default()
    }
    fn get_u16_param(&self, key: &str) -> u16 {
        self.value(key).unwrap_or_default()
    }
    fn get_u32_param(&self, key: &str) -> u32 {
        self.value(key).unwrap_or_default()
    }
    fn get_u64_param(&self, key: &str) -> u64 {
        self.value(key).unwrap_or_default()
    }
}
