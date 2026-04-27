# 1. Core Language

## Arithmetic

```
(+ expr*) ↦ (int?)
(- expr*) ↦ (int?)
(* expr*) ↦ (int?)
     expr : (int?)

(/ a b) ↦ (int?)

(** base exponent) ↦ (int?)
     base : (int?)
 exponent : (int?)
```

### Examples

```wal
>-> (+ 1 2 3 4)
10
>-> (/ 10 3)
3.3333333333333335
```

## Logic and Comparisons

```
(! expr) ↦ (int?)
(&& expr*) ↦ (int?)
(|| expr*) ↦ (int?)
     expr : (int?), (list?), (array?)
(= expr*) ↦ (int?)
(!= expr*) ↦ (int?)

(> a b) ↦ (int?)
(< a b) ↦ (int?)
(>= a b) ↦ (int?)
(<= a b) ↦ (int?)
        a : (int?)
        b : (int?)
```

Many logical functions support more than two inputs.
In the case of `&&` all arguments must be positive.
`1` and `0` are equivalent to the boolean literals `#t` and `#f`.

### Examples

```wal
>-> (! #f)
#t
>-> (&& #t #t)
#t
>-> (&& #t #t #f)
#f
>-> (&& #t #t #0)
#f
>-> (= 5 5)
#t
>-> (= '(1 2) '(1 2))
#t
```

## Program State

### let

```
(let ((id expr)+) body)
       id : (symbol?)
     expr : WAL expression
     body : WAL expression
```

The `let` function locally binds the results of the expressions to the symbols
during the evaluation of `body`. Later bindings can use earlier bindings.

```wal
>-> (let ([x 10])
        x)
10
>-> (let ([x 10]
          [y 12])
         (+ x y))
22
>-> (let ([x 10]
          [y x])
         (+ x y))
20
```

### define

```
(define id expr)
       id : (symbol?)
     expr : WAL expression
```

Evaluates `expr` and binds the result to `id` and returns the result.

```wal
>-> (define x 10)
10
>-> x
10
>-> (define y (+ x x))
20
```

### set!

```
(set! id expr)
       id : (symbol?)
     expr : WAL expression
```

Updates the existing binding `id` to the result of evaluating `expr`.

```wal
>-> (define x 10)
10
>-> x
10
>-> (set! x (+ x x))
20
>-> x
20
```

## Functions

### defun

```
(defun name (args+) body+)
     name : (symbol?)
     args : (symbol?) or ((symbol?)+)
     body : WAL expressions
```

New functions can be defined using `defun`. The parameters can either be a
list of symbols, or a single symbol. If it is a list of symbols, the function
expects one argument for each entry. If it is a single symbol, all arguments
are passed to the function as a list (variadic).

**Formatting Style**: Typically, the argument list is enclosed in `[]` braces.

```wal
>-> (defun times-two [n] (* n 2))
Function: times-two
>-> (times-two 5)
10
```

Variadic example:

```wal
>-> (defun times-two-list xs (for/list [x xs] (* x 2)))
Function: times-two-list
>-> (times-two-list 1 2 3)
(2 4 6)
```

### fn (anonymous functions)

```
(fn (args+) body+)
     args : (symbol?) or ((symbol?)+)
     body : WAL expressions
```

```wal
>-> ((fn [a b] (+ a b)) 1 2)
3
```

### Closures

WAL has closures, thus functions can capture variables.

```wal
(defun make-counter [name]
  (define cnt 0)
  (fn [] (set! cnt (+ cnt 1))
         (print name ": " cnt)))

(define cnt1 (make-counter "Cnt1"))
(define cnt2 (make-counter "Cnt2"))
(cnt1)
(cnt1)
(cnt2)
(cnt1)
```

Output:
```
Cnt1: 1
Cnt1: 2
Cnt2: 1
Cnt1: 3
```

## Control Flow

### do

```
(do body+)
     body : WAL expression
```

Evaluates expressions in body in order, returns the result of the last element.

### when / unless

```
(when cond body+)
(unless cond body+)
```

`when`: evaluates body if cond is truthy (int > 0, #t, or non-empty list).
`unless`: evaluates body if cond is falsy (0, #f, or empty list).

### if

```
(if cond then else)
```

Evaluates `then` if `cond` is positive, otherwise evaluates `else`.
Both `then` and `else` can only be single expressions. Use `do` for multiple.

```wal
(if a[2]
  (do (print "Option a")
      (set! a (list)))
  (do (print "Option b")
      (set! a (list))))
```

### cond

```
(cond (guard expr+)+)
    guard : WAL expression
     expr : WAL expression
```

Multiple conditional cases. Goes through all clauses and evaluates `exprs`
for the first clause whose `guard` is positive.

```wal
(defun fib [n]
  (cond [(= n 1) 1]
        [(= n 2) 1]
        [#t (+ (fib (- n 1))
               (fib (- n 2)))]))
```

### case

```
(case key (value expr+)+)
      key : WAL expression
    value : WAL expression
     expr : WAL expression
```

Checks if `value` equals `key`. A `default` value catches unmatched cases.

```wal
(case (+ a b)
      [1 "one"]
      [2 "two"]
      [3 "three"]
      [default "> three"])
```

## Printing

### print / printf

```
(print args*)
(printf format args*)
```

`printf` follows Python `printf-style` formatting rules.

## Utility

### eval-file

```
(eval-file file)
     file : (symbol?)
```

Evaluates WAL code in `file.wal` and combines the resulting program state.

### exit

```
(exit code)
      code : (int?)
```

Exits with `code` as the return value.
