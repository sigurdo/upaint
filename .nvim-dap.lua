local dap = require("dap")

dap.adapters.lldb = {
    type = "executable",
    command = "/usr/sbin/codelldb", -- adjust as needed
    name = "lldb",
}

dap.configurations.rust = {
    {
        name = "hello-world",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/hello-world"
        end,
        cwd = "${workspaceFolder}",
        stopOnEntry = false,
    },
    {
        name = "test_adopt_new_keystroke_system",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/test_adopt_new_keystroke_system"
        end,
        cwd = "${workspaceFolder}",
        stopOnEntry = false,
    },
}
