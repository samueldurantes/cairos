local M = {}

function M.setup()
	local lspconfig = require("lspconfig")
	local configs = require("lspconfig.configs")
	local util = require("lspconfig.util")

	if not configs.cairos_ls then
		configs.cairos_ls = {
			default_config = {
				cmd = { "/home/tiagopg/projects/cairos/target/debug/cli", "language-server" },
				filetypes = { "*" },
				root_dir = function(fname)
					return util.find_git_ancestor(fname) or vim.loop.cwd()
				end,
				settings = {},
			},
		}
	end

	lspconfig.cairos_ls.setup({
		on_attach = function(_, bufnr)
			print("Cairos attached to buffer " .. bufnr)
		end,
	})
end

return M
