//! Value-only boundary for a musl sREI module and a separate Agent runtime.
//! The exported C functions never pass Rust trait objects across that boundary.

use crate::params::ParamValue;
use crate::{AccModules, AgentModuleInfo, AgentModuleInput, AgentModuleOutput, AgentModuleParams};
use crate::{AgentModuleResult, AgentModuleTask, ParamMeta, StopReason, TaskRequest, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const BRIDGE_VERSION: u32 = 1;
pub const EVENT_KIND: c_int = 0x4d42;
pub const CALL_PENDING: u32 = 0;
pub const CALL_DONE: u32 = 1;

#[repr(C)]
pub struct InstallArgs {
    pub log_ptr: *const c_void,
    pub owner_id: c_int,
}

#[repr(C)]
pub struct ArgsList {
    pub params: *const c_char,
    pub params_length: c_int,
    pub task_id: c_int,
}

pub type EventCallback = extern "C" fn(c_int, c_int, *const c_char, c_int);

/// The caller owns the input until completion. The plugin owns the reply and
/// must release it through `bridge_free` while its runtime is still loaded.
#[repr(C)]
pub struct Invocation {
    pub request_ptr: *const u8,
    pub request_len: usize,
    pub response_ptr: AtomicUsize,
    pub response_len: AtomicUsize,
    pub state: AtomicU32,
}

impl Invocation {
    pub fn new(request: &[u8]) -> Self {
        Self {
            request_ptr: request.as_ptr(),
            request_len: request.len(),
            response_ptr: AtomicUsize::new(0),
            response_len: AtomicUsize::new(0),
            state: AtomicU32::new(CALL_PENDING),
        }
    }
}

/// The plugin owns the event and reply buffers until the host dispatcher
/// acknowledges them. No cross-runtime allocation or deallocation occurs.
#[repr(C)]
pub struct EventCall {
    pub call_id: u64,
    pub request_ptr: *const u8,
    pub request_len: usize,
    pub response_ptr: *mut u8,
    pub response_cap: usize,
    pub response_len: AtomicUsize,
    pub state: AtomicU32,
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    Describe,
    Run {
        module_id: u64,
        task_id: u64,
        header: ParamMeta,
        input: ParamMeta,
    },
    Stop {
        task_id: u64,
        reason: u8,
    },
    WaitStopped {
        task_id: u64,
        timeout_ms: u64,
    },
    ClearCache {
        task_id: u64,
        tag: String,
    },
    ToString {
        task_id: u64,
    },
}

#[derive(Serialize, Deserialize)]
pub enum Reply {
    Modules(Vec<(u64, bool)>),
    Bool(bool),
    String(String),
    Unit,
    Error(String),
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    Send {
        task_id: u64,
        status: Option<u8>,
        sub_task_type: Option<u64>,
        meta: ParamMeta,
    },
    Log {
        task_id: u64,
        level: u8,
        data: Vec<u8>,
    },
    GetInfo {
        task_id: u64,
        data_type: String,
        key: String,
    },
    UpdateInfo {
        task_id: u64,
        data_type: String,
        key: String,
        value: ParamValue,
    },
    StopTasks {
        task_id: u64,
        module_ids: Vec<u64>,
        timeout_ms: u64,
    },
}

#[derive(Serialize, Deserialize)]
pub enum EventReply {
    Unit,
    Value(Option<ParamValue>),
    Bool(bool),
    Error(String),
}

#[derive(Clone, Default)]
pub struct OwnedInput(pub ParamMeta);

impl AgentModuleInput for OwnedInput {
    fn get_value(&self, key: &str) -> Option<ParamValue> {
        self.0.get(key).cloned()
    }
    fn snapshot(&self) -> ParamMeta {
        self.0.clone()
    }
    fn new_instance(&self, key: &str) -> Arc<dyn AgentModuleInput> {
        Arc::new(OwnedInput(self.nested(key)))
    }
    fn get_iter_param(&self, key: &str) -> Box<dyn AgentModuleInput> {
        Box::new(OwnedInput(self.nested(key)))
    }
    fn get_iter_list_param(&self, key: &str) -> Vec<Box<dyn AgentModuleInput>> {
        match self.get_value(key) {
            Some(ParamValue::ParamList(items)) => items
                .into_iter()
                .map(|item| Box::new(OwnedInput(item)) as Box<dyn AgentModuleInput>)
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
        self.get_value(key).and_then(|v| v.try_into().ok())
    }
}

fn io_error(message: impl Into<String>) -> Box<dyn std::error::Error> {
    std::io::Error::other(message.into()).into()
}

fn stop_reason(code: u8) -> Result<StopReason, String> {
    match code {
        0 => Ok(StopReason::Terminal),
        1 => Ok(StopReason::Remote),
        2 => Ok(StopReason::Deleted),
        3 => Ok(StopReason::Offline),
        4 => Ok(StopReason::Unload),
        5 => Ok(StopReason::Failed),
        _ => Err(obfstr::obfstring!("invalid stop reason")),
    }
}

pub fn stop_reason_code(reason: StopReason) -> u8 {
    match reason {
        StopReason::Terminal => 0,
        StopReason::Remote => 1,
        StopReason::Deleted => 2,
        StopReason::Offline => 3,
        StopReason::Unload => 4,
        StopReason::Failed => 5,
    }
}

struct Runtime {
    modules: AccModules,
    tasks: Mutex<HashMap<u64, Arc<dyn AgentModuleTask>>>,
    workers: Mutex<Vec<std::thread::JoinHandle<()>>>,
    callback: EventCallback,
    owner_id: c_int,
}

static RUNTIME: OnceLock<Mutex<Option<Arc<Runtime>>>> = OnceLock::new();

fn runtime_cell() -> &'static Mutex<Option<Arc<Runtime>>> {
    RUNTIME.get_or_init(|| Mutex::new(None))
}

pub unsafe fn install(args: *const InstallArgs, len: usize, factory: fn() -> AccModules) -> i64 {
    if args.is_null()
        || len != std::mem::size_of::<InstallArgs>()
        || (*args).log_ptr.is_null()
        || (*args).owner_id <= 0
    {
        return -1;
    }
    let mut slot = runtime_cell().lock().unwrap();
    if slot.is_some() {
        return -2;
    }
    let modules = factory();
    if modules.is_empty() || modules.contains_key(&0) {
        return -3;
    }
    let callback: EventCallback = std::mem::transmute((*args).log_ptr);
    *slot = Some(Arc::new(Runtime {
        modules,
        tasks: Mutex::new(HashMap::new()),
        workers: Mutex::new(Vec::new()),
        callback,
        owner_id: (*args).owner_id,
    }));
    0
}

/// `run` only launches a plugin-owned worker; the host can release the TLS
/// switch immediately. Concurrent trait calls then use their own musl TLS.
pub unsafe fn run(args: *const ArgsList) -> i64 {
    if args.is_null()
        || (*args).params.is_null()
        || (*args).params_length != std::mem::size_of::<usize>() as c_int
    {
        return -1;
    }
    let invoke_ptr = std::ptr::read_unaligned((*args).params as *const *const Invocation);
    if invoke_ptr.is_null() {
        return -1;
    }
    let invoke = &*invoke_ptr;
    if invoke.request_ptr.is_null() || invoke.request_len == 0 {
        return -1;
    }
    let request = std::slice::from_raw_parts(invoke.request_ptr, invoke.request_len).to_vec();
    let Some(runtime) = runtime_cell().lock().unwrap().as_ref().cloned() else {
        return -2;
    };
    let mut workers = runtime.workers.lock().unwrap();
    let mut i = 0;
    while i < workers.len() {
        if workers[i].is_finished() {
            let _ = workers.swap_remove(i).join();
        } else {
            i += 1;
        }
    }
    let worker_runtime = runtime.clone();
    let invoke_address = invoke_ptr as usize;
    let spawned = std::thread::Builder::new().spawn(move || {
        let invoke = unsafe { &*(invoke_address as *const Invocation) };
        let reply = match bincode::deserialize::<Command>(&request) {
            Ok(command) => worker_runtime.execute(command),
            Err(error) => Reply::Error(error.to_string()),
        };
        let bytes = bincode::serialize(&reply)
            .unwrap_or_default()
            .into_boxed_slice();
        let len = bytes.len();
        let ptr = Box::into_raw(bytes) as *mut u8;
        invoke.response_len.store(len, Ordering::Release);
        invoke.response_ptr.store(ptr as usize, Ordering::Release);
        invoke.state.store(CALL_DONE, Ordering::Release);
    });
    match spawned {
        Ok(worker) => {
            workers.push(worker);
            0
        }
        Err(_) => -3,
    }
}

pub unsafe fn free_response(ptr: *const u8, len: usize) -> i64 {
    if ptr.is_null() {
        return if len == 0 { 0 } else { -1 };
    }
    drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(
        ptr as *mut u8,
        len,
    )));
    0
}

pub fn uninstall() -> i64 {
    let mut slot = runtime_cell().lock().unwrap();
    let Some(runtime) = slot.as_ref() else {
        return -1;
    };
    let mut workers = runtime.workers.lock().unwrap();
    if workers.iter().any(|worker| !worker.is_finished()) {
        return -2;
    }
    for worker in workers.drain(..) {
        let _ = worker.join();
    }
    if runtime
        .tasks
        .lock()
        .unwrap()
        .values()
        .any(|task| !matches!(task.wait_stopped(Instant::now()), Ok(true)))
    {
        return -2;
    }
    drop(workers);
    *slot = None;
    0
}

impl Runtime {
    fn execute(&self, command: Command) -> Reply {
        match self.try_execute(command) {
            Ok(reply) => reply,
            Err(error) => Reply::Error(error.to_string()),
        }
    }

    fn try_execute(&self, command: Command) -> AgentModuleResult<Reply> {
        match command {
            Command::Describe => Ok(Reply::Modules(
                self.modules
                    .iter()
                    .map(|(&id, factory)| (id, factory.is_channel()))
                    .collect(),
            )),
            Command::Run {
                module_id,
                task_id,
                header,
                input,
            } => {
                let factory = self
                    .modules
                    .get(&module_id)
                    .ok_or_else(|| io_error(obfstr::obfstring!("module id missing")))?;
                let task = {
                    let mut tasks = self.tasks.lock().unwrap();
                    tasks
                        .entry(task_id)
                        .or_insert_with(|| factory.create_task())
                        .clone()
                };
                task.run(AgentModuleParams {
                    request: TaskRequest {
                        header: Arc::new(OwnedInput(header)),
                        input: Arc::new(OwnedInput(input)),
                    },
                    info: Arc::new(Proxy {
                        callback: self.callback,
                        owner_id: self.owner_id,
                        task_id,
                    }),
                    output: Arc::new(Proxy {
                        callback: self.callback,
                        owner_id: self.owner_id,
                        task_id,
                    }),
                })?;
                Ok(Reply::Unit)
            }
            Command::Stop { task_id, reason } => {
                let task = self.tasks.lock().unwrap().get(&task_id).cloned();
                if let Some(task) = task {
                    task.request_stop(stop_reason(reason).map_err(io_error)?)?;
                }
                Ok(Reply::Unit)
            }
            Command::WaitStopped {
                task_id,
                timeout_ms,
            } => {
                let task = self.tasks.lock().unwrap().get(&task_id).cloned();
                let Some(task) = task else {
                    return Ok(Reply::Bool(true));
                };
                let done = task.wait_stopped(Instant::now() + Duration::from_millis(timeout_ms))?;
                if done {
                    self.tasks.lock().unwrap().remove(&task_id);
                }
                Ok(Reply::Bool(done))
            }
            Command::ClearCache { task_id, tag } => {
                if let Some(task) = self.tasks.lock().unwrap().get(&task_id).cloned() {
                    task.clear_cache(&tag);
                }
                Ok(Reply::Unit)
            }
            Command::ToString { task_id } => {
                let task = self.tasks.lock().unwrap().get(&task_id).cloned();
                Ok(Reply::String(
                    task.map_or_else(String::new, |task| task.to_string()),
                ))
            }
        }
    }
}

struct Proxy {
    callback: EventCallback,
    owner_id: c_int,
    task_id: u64,
}
static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

impl Proxy {
    fn call(&self, event: Event, retry_read: bool) -> AgentModuleResult<EventReply> {
        let request = bincode::serialize(&event)?.into_boxed_slice();
        let mut reply = vec![0u8; 8192].into_boxed_slice();
        let call_id = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);
        loop {
            let event_call = Box::new(EventCall {
                call_id,
                request_ptr: request.as_ptr(),
                request_len: request.len(),
                response_ptr: reply.as_mut_ptr(),
                response_cap: reply.len(),
                response_len: AtomicUsize::new(0),
                state: AtomicU32::new(CALL_PENDING),
            });
            let ptr = event_call.as_ref() as *const EventCall as usize;
            (self.callback)(
                self.owner_id,
                EVENT_KIND,
                &ptr as *const usize as *const c_char,
                std::mem::size_of::<usize>() as c_int,
            );
            let start = Instant::now();
            while event_call.state.load(Ordering::Acquire) == CALL_PENDING {
                if start.elapsed() > Duration::from_secs(300) {
                    // A delayed host callback may still write these buffers.
                    std::mem::forget(request);
                    std::mem::forget(reply);
                    std::mem::forget(event_call);
                    return Err(io_error(obfstr::obfstring!("module event timeout")));
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            let len = event_call.response_len.load(Ordering::Acquire);
            if len > reply.len() {
                if !retry_read {
                    return Err(io_error(obfstr::obfstring!("module event reply too large")));
                }
                reply = vec![0u8; len].into_boxed_slice();
                continue;
            }
            return Ok(bincode::deserialize(&reply[..len])?);
        }
    }
}

impl AgentModuleInfo for Proxy {
    fn get_extend_info(&self, data_type: &str, key: &str) -> Option<ParamValue> {
        match self.call(
            Event::GetInfo {
                task_id: self.task_id,
                data_type: data_type.into(),
                key: key.into(),
            },
            true,
        ) {
            Ok(EventReply::Value(value)) => value,
            _ => None,
        }
    }
    fn update_extend_info(
        &self,
        data_type: &str,
        key: &str,
        value: ParamValue,
    ) -> AgentModuleResult<()> {
        match self.call(
            Event::UpdateInfo {
                task_id: self.task_id,
                data_type: data_type.into(),
                key: key.into(),
                value,
            },
            false,
        )? {
            EventReply::Unit => Ok(()),
            EventReply::Error(error) => Err(io_error(error)),
            _ => Err(io_error(obfstr::obfstring!("invalid info reply"))),
        }
    }
    fn stop_module_tasks(&self, ids: &[u64], deadline: Instant) -> AgentModuleResult<bool> {
        let timeout_ms = deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(u64::MAX as u128) as u64;
        match self.call(
            Event::StopTasks {
                task_id: self.task_id,
                module_ids: ids.to_vec(),
                timeout_ms,
            },
            false,
        )? {
            EventReply::Bool(done) => Ok(done),
            EventReply::Error(error) => Err(io_error(error)),
            _ => Err(io_error(obfstr::obfstring!("invalid stop reply"))),
        }
    }
}

impl AgentModuleOutput for Proxy {
    fn send(
        &self,
        status: Option<TaskStatus>,
        sub_task_type: Option<u64>,
        meta: ParamMeta,
    ) -> AgentModuleResult<()> {
        match self.call(
            Event::Send {
                task_id: self.task_id,
                status: status.map(|s| s as u8),
                sub_task_type,
                meta,
            },
            false,
        )? {
            EventReply::Unit => Ok(()),
            EventReply::Error(error) => Err(io_error(error)),
            _ => Err(io_error(obfstr::obfstring!("invalid output reply"))),
        }
    }
    fn log_info(&self, msg: &str) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 0,
                data: msg.as_bytes().to_vec(),
            },
            false,
        );
    }
    fn log_success(&self, msg: &str) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 1,
                data: msg.as_bytes().to_vec(),
            },
            false,
        );
    }
    fn log_error(&self, msg: &str) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 2,
                data: msg.as_bytes().to_vec(),
            },
            false,
        );
    }
    fn log_warning(&self, msg: &str) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 3,
                data: msg.as_bytes().to_vec(),
            },
            false,
        );
    }
    fn log_data(&self, data: Vec<u8>) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 4,
                data,
            },
            false,
        );
    }
    fn log_big_data(&self, data: Vec<u8>) {
        let _ = self.call(
            Event::Log {
                task_id: self.task_id,
                level: 5,
                data,
            },
            false,
        );
    }
}

#[macro_export]
macro_rules! export_linux_module {
    ($factory:path) => {
        #[no_mangle]
        pub extern "C" fn AccModuleAbiVersion() -> u32 {
            agent_module_trait::MODULE_ABI_VERSION
        }
        #[no_mangle]
        pub extern "C" fn AccLinuxBridgeVersion() -> u32 {
            agent_module_trait::linux_bridge::BRIDGE_VERSION
        }
        #[no_mangle]
        pub unsafe extern "C" fn install(
            args: *const agent_module_trait::linux_bridge::InstallArgs,
            len: usize,
        ) -> i64 {
            agent_module_trait::linux_bridge::install(args, len, $factory)
        }
        #[no_mangle]
        pub unsafe extern "C" fn run(
            args: *const agent_module_trait::linux_bridge::ArgsList,
        ) -> i64 {
            agent_module_trait::linux_bridge::run(args)
        }
        #[no_mangle]
        pub extern "C" fn uninstall() -> i64 {
            agent_module_trait::linux_bridge::uninstall()
        }
        #[no_mangle]
        pub unsafe extern "C" fn bridge_free(ptr: *const u8, len: usize) -> i64 {
            agent_module_trait::linux_bridge::free_response(ptr, len)
        }
    };
}
