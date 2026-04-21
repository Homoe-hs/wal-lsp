-- WAL LSP Configuration for Neovim
-- Add to your init.lua or create a separate file

local lspconfig = require('lspconfig')

lspconfig.wal_lsp.setup({
  cmd = {"/home/hesheng/Projects/WAL-lsp/target/release/wal-lsp"},
  filetypes = {"wal"},
  root_dir = function(fname)
    return lspconfig.util.find_root({"*.wal"}, fname)
  end,
  settings = {
    wal = {}
  },
})

-- Optional: Add to filetype detection
vim.filetype.add({
  extension = {
    wal = "wal",
  },
})
