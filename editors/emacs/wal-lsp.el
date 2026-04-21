;;; wal-lsp.el --- WAL Language Server configuration  -*- lexical-binding: t; -*-

;;; Commentary:
;; WAL LSP configuration for Emacs using lsp-mode

;;; Code:

(require 'lsp-mode')

(add-to-list 'lsp-language-id-configuration '("\\.wal\\'" . "wal"))

(lsp-register-client
 (make-lsp-client :server-id 'wal-lsp
                  :major-modes '(wal-mode)
                  :notification-handlers (ht<-alist)
                  :initialization-options (make-hash-table)
                  :cmd-processor (lambda (cmd)
                                  (cons (expand-file-name "/home/hesheng/Projects/WAL-lsp/target/release/wal-lsp")
                                        (when (cdr cmd) (list (cdr cmd))))))

(define-derived-mode wal-mode scheme-mode "WAL"
  "Major mode for WAL (Waveform Analysis Language)."
  :syntax-table nil
  (setq font-lock-defaults nil))

(provide 'wal-lsp)
;;; wal-lsp.el ends here
