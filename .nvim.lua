-- Project-local nvim-dap configuration for the kfs kernel.
--
-- Usage:
--   1. Open Neovim from the repository root.
--   2. Set breakpoints normally.
--   3. Run :DapContinue.
--   4. Pick either:
--        - "kfs kernel" to debug the normal kernel binary
--        - "kfs tests"  to debug the kernel test binary
--
-- The chosen configuration will:
--   - build the requested binary,
--   - start QEMU paused with a GDB stub,
--   - attach GDB automatically.

local dap = require("dap")
local uv = vim.uv or vim.loop

-- -----------------------------------------------------------------------------
-- Project paths
-- -----------------------------------------------------------------------------

local script_path = debug.getinfo(1, "S").source:sub(2)
local project_root = vim.fn.fnamemodify(script_path, ":p:h")

local kernel_path = project_root .. "/target/i386-unknown-none/debug/kfs"
local qemu_runner = project_root .. "/scripts/gdb.sh"
local qemu_pid_file = project_root .. "/.logs/gdb-qemu.pid"

local gdb_host = "127.0.0.1"
local gdb_port = 1234
local managed_qemu = false

-- -----------------------------------------------------------------------------
-- Small helpers
-- -----------------------------------------------------------------------------

local function notify(message, level)
	vim.notify(message, level or vim.log.levels.INFO, { title = "kfs debug" })
end

local function run_command(command)
	local output = vim.fn.system(command)
	if vim.v.shell_error ~= 0 then
		error(output)
	end
	return output
end

local function resolve_program_path(program)
	local program_path = program

	if type(program_path) == "function" then
		local ok, resolved_path = pcall(program_path)
		if not ok then
			notify(resolved_path, vim.log.levels.ERROR)
			return nil
		end
		program_path = resolved_path
	end

	if type(program_path) ~= "string" or program_path == "" then
		notify("invalid program path for kfs debug configuration", vim.log.levels.ERROR)
		return nil
	end

	return program_path
end

local function stop_qemu_debug()
	managed_qemu = false

	if vim.fn.filereadable(qemu_pid_file) == 0 then
		return
	end

	local pid = vim.fn.readfile(qemu_pid_file)[1]
	if pid and pid ~= "" then
		vim.fn.system({ "kill", pid })
	end

	vim.fn.delete(qemu_pid_file)
end

-- QEMU starts in the background, so we poll the TCP port until the GDB stub is ready.
local function wait_for_gdb_stub(on_ready)
	local retries = 40
	local delay_ms = 100

	local function try_connect(remaining)
		local socket = uv.new_tcp()

		socket:connect(gdb_host, gdb_port, function(err)
			socket:close()

			if not err then
				vim.schedule(on_ready)
				return
			end

			if remaining > 1 then
				vim.defer_fn(function()
					try_connect(remaining - 1)
				end, delay_ms)
				return
			end

			vim.schedule(function()
				notify(
					string.format("timed out waiting for GDB stub on %s:%d", gdb_host, gdb_port),
					vim.log.levels.ERROR
				)
			end)
		end)
	end

	try_connect(retries)
end

-- -----------------------------------------------------------------------------
-- Build helpers
-- -----------------------------------------------------------------------------

local function build_kernel_binary()
	run_command({ "cargo", "build" })
	return kernel_path
end

local function build_test_binary()
	local output = run_command({ "cargo", "test", "--no-run", "--message-format=json" })
	local test_binary = nil

	for _, line in ipairs(vim.split(output, "\n", { trimempty = true })) do
		local ok, decoded = pcall(vim.json.decode, line)
		if
			ok
			and decoded.reason == "compiler-artifact"
			and decoded.executable ~= vim.NIL
			and decoded.executable ~= nil
			and decoded.profile
			and decoded.profile.test
			and decoded.target
			and decoded.target.name == "kfs"
		then
			test_binary = decoded.executable
		end
	end

	if not test_binary then
		error("could not locate kernel test executable")
	end

	return test_binary
end

-- -----------------------------------------------------------------------------
-- QEMU launcher
-- -----------------------------------------------------------------------------

local function start_qemu(program_path, on_ready)
	managed_qemu = true

	vim.fn.jobstart({ qemu_runner, program_path }, {
		cwd = project_root,
		stdout_buffered = true,
		stderr_buffered = true,
		on_exit = function(_, code)
			if code ~= 0 then
				vim.schedule(function()
					managed_qemu = false
					notify("failed to start QEMU debug runner", vim.log.levels.ERROR)
				end)
				return
			end

			wait_for_gdb_stub(on_ready)
		end,
	})
end

-- -----------------------------------------------------------------------------
-- DAP adapter
-- -----------------------------------------------------------------------------

-- This adapter is dynamic:
--   1. resolve the requested program path,
--   2. start QEMU for it,
--   3. then start GDB in DAP mode.
dap.adapters["kfs-gdb"] = function(callback, config)
	local program_path = resolve_program_path(config.program)
	if not program_path then
		return
	end

	start_qemu(program_path, function()
		callback({
			type = "executable",
			command = "gdb",
			args = {
				"-q",
				-- Load symbols immediately so source breakpoints resolve.
				"-ex",
				"file " .. program_path,
				-- This project targets 32-bit i386.
				"-ex",
				"set architecture i386",
				-- Let GDB keep unresolved breakpoints until symbols/target are ready.
				"-ex",
				"set breakpoint pending on",
				"-i",
				"dap",
			},
		})
	end)
end

-- -----------------------------------------------------------------------------
-- DAP configurations shown by :DapContinue
-- -----------------------------------------------------------------------------

dap.configurations.rust = {
	{
		name = "kfs kernel",
		type = "kfs-gdb",
		request = "attach",
		target = string.format("%s:%d", gdb_host, gdb_port),
		cwd = project_root,
		program = build_kernel_binary,
	},
	{
		name = "kfs tests",
		type = "kfs-gdb",
		request = "attach",
		target = string.format("%s:%d", gdb_host, gdb_port),
		cwd = project_root,
		program = build_test_binary,
	},
}

-- -----------------------------------------------------------------------------
-- Session cleanup
-- -----------------------------------------------------------------------------

local function cleanup_debug_qemu()
	if managed_qemu then
		stop_qemu_debug()
	end
end

dap.listeners.before.disconnect["kfs-debug-cleanup"] = cleanup_debug_qemu
dap.listeners.before.event_terminated["kfs-debug-cleanup"] = cleanup_debug_qemu
dap.listeners.before.event_exited["kfs-debug-cleanup"] = cleanup_debug_qemu

-- Optional manual cleanup command.
vim.api.nvim_create_user_command("KfsDebugStop", stop_qemu_debug, {})
