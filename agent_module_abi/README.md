# Linux Agent Module ABI 2

`agent_module_abi` is the small, `no_std` contract between a Linux V2 plugin
and its Agent host. The plugin exports `AccModuleAbiVersion()` and `DLLWMain()`;
the latter returns a static `Manifest`. The Agent owns the registration table
and its Rust `AgentModule` proxies. The plugin owns every opaque task handle
and destroys it through its own `destroy_task` callback.

Only C-layout values, borrowed buffers, function pointers, and opaque handles
cross the binary boundary. The Agent supplies a `HostApi` for each `run` call;
that table and its context remain valid for that call only. Returned input
buffers and `info_get` values also remain valid until `run` returns. Plugins
must copy anything they retain beyond that point. No Rust `String`, `Vec`,
`Arc`, allocator ownership, or Rust trait object crosses the ELF boundary.

`get_field` and `get_index` return 0 for a value, 1 when absent, and -1 on
error. `info_get` follows the same convention. `info_update` and `send` return
0 on success and -1 on error. `stop_modules` returns 1 when all selected tasks
stopped, 0 when they did not, and -1 on error. `send` uses `has_subtask` so a
valid subtask value of zero is distinct from no subtask.

The normal Agent side still implements the Rust `AgentModule` traits through
its proxy. Plugins may use Rust internally, but their final Linux `.so` must
be self-contained: no unresolved dynamic symbols, `PT_INTERP`, or
`DT_NEEDED`. Stub-provided TLS imports are the only explicit exception when
needed by an image. The server packs that `.so` with `elf2shc` ABI 2 before
delivery; the Agent receives shellcode, not the original ELF.
