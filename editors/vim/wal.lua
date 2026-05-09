-- WAL LSP Configuration for Neovim
-- Add to your init.lua

local lspconfig = require('lspconfig')

lspconfig.wal_lsp = {
  default_config = {
    cmd = { "wal-lsp" },
    filetypes = { "wal" },
    root_dir = function(fname)
      return lspconfig.util.root_pattern(".git", "*.wal")(fname)
    end,
    settings = {},
  },
}

lspconfig.wal_lsp.setup({})

-- Filetype detection
vim.filetype.add({
  extension = {
    wal = "wal",
  },
})
