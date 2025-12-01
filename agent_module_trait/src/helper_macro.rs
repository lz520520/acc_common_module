


#[macro_export]
macro_rules! meta_insert_result {
    ($map:expr, $val:expr) => {{
        $map.insert(agent_module_trait::RAW_RESULT.clone(), $val.into());
    }};
}

#[macro_export]
macro_rules! meta_insert_result_obfstr {
    ($map:expr, $val:expr) => {{
        $map.insert(agent_module_trait::RAW_RESULT.clone(), obfstr::obfstr!($val).into());
    }};
}


#[macro_export]
macro_rules! meta_insert_error {
    ($map:expr, $val:expr) => {{
        $map.insert(agent_module_trait::RAW_ERROR.clone(), $val.into());
    }};
}

#[macro_export]
macro_rules! meta_insert_error_obfstr {
    ($map:expr, $val:expr) => {{
        $map.insert(agent_module_trait::RAW_ERROR.clone(), obfstr::obfstr!($val).into());
    }};
}


#[macro_export]
macro_rules! meta_over_log {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_result!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskOver), None, meta);
    }};
}


#[macro_export]
macro_rules! meta_over_log_obfstr {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_result_obfstr!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskOver), None, meta);
    }};
}
#[macro_export]
macro_rules! meta_process_log {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_result!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskProcess), None, meta);
    }};
}


#[macro_export]
macro_rules! meta_process_log_obfstr {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_result_obfstr!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskProcess), None, meta);
    }};
}

#[macro_export]
macro_rules! meta_error_log {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_error!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskOver), None, meta);
    }};
}
#[macro_export]
macro_rules! meta_error_log_obfstr {
    ($output:expr, $val:expr) => {{
        let mut meta = agent_module_trait::ParamMeta::new();
        agent_module_trait::meta_insert_error_obfstr!(meta, $val);
        $output.send(Some(agent_module_trait::TaskStatus::TaskOver), None, meta);
    }};
}
