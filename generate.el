(require 'rx)

(defmacro jimb-rx (&rest body-forms)
   (let ((add-ins (list
                   `(ws* . ,(rx (* (char space ?\n))))
                   `(ws+ . ,(rx (+ (char space ?\n))))
                   `(ident . ,(rx (+ (char alnum digit "_"))))
                   `(type . ,(rx (opt "&mut ") (+ (char alnum digit "_&<>[]"))))
                   `(string . ,(rx ?\" (* (not (any ?\"))) ?\")))))
     (let ((rx-constituents (append add-ins rx-constituents nil)))
       (macroexpand `(rx ,@body-forms)))))

(defun jimb-gen-record ()
  (interactive)
  (save-excursion
    (unless (looking-at (jimb-rx (opt ws* "unsafe") ws* "fn" ws+ (group ident) "(" ws* "&self"))
      (error "not looking at start of function"))
    (let ((fn (match-string-no-properties 1)))
      (goto-char (match-end 0))
      (let (args)
        (while (looking-at (jimb-rx ws* "," ws* (group ident) ws* ":" ws* (group type) ws*))
          (push (list (match-string-no-properties 1)
                      (match-string-no-properties 2))
                args)
          (goto-char (match-end 0)))
        (setq args (nreverse args))
        (unless (looking-at (jimb-rx (opt ws* ",") ws* ")"
                                     (opt ws* "->" ws* type)
                                     ws* "{"
                                     ws* (group "unimplemented!"
                                                ws* "(" ws* string ws* ")" ws* ";")))
          (error "Didn't see expected function body"))
        (goto-char (match-beginning 1))
        (delete-region (point) (match-end 1))
        (insert "simple!(self." fn "(")
        (dolist (arg-type args)
          (insert (car arg-type) ", "))
        (when args
          (delete-region (- (point) 2) (point)))
        (insert "))")

        (set-buffer "call.rs")
        (save-excursion
          (goto-char (point-min))
          (search-forward "pub enum Call ")
          (forward-sexp)
          (forward-line 0)
          (insert "    " fn " { ")
          (dolist (arg-type args)
            (insert (car arg-type) ": " (cadr arg-type) ", "))
          (when args
            (delete-region (- (point) 2) (point)))
          (insert " },\n")
          (save-buffer))))))
