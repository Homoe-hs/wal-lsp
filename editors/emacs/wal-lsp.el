;;; wal-lsp.el --- WAL Language Server configuration  -*- lexical-binding: t; -*-

;;; Commentary:
;; WAL LSP configuration for Emacs using lsp-mode

;;; Code:

(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '("\\.wal\\'" . "wal"))

(lsp-register-client
 (make-lsp-client
  :server-id 'wal-lsp
  :major-modes '(wal-mode)
  :language-id "wal"
  :new-connection (lsp-stdio-connection '("wal-lsp"))
  :notification-handlers (make-hash-table)
  :initialization-options (make-hash-table)
  :multi-root t))

(define-derived-mode wal-mode prog-mode "WAL"
  "Major mode for WAL (Waveform Analysis Language)."
  :syntax-table nil
  (setq font-lock-defaults nil))

(add-to-list 'auto-mode-alist '("\\.wal\\'" . wal-mode))

(provide 'wal-lsp)
;;; wal-lsp.el ends here
