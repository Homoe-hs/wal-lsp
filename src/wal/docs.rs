use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub example: Option<String>,
}

static FUNCTION_DOCS: Lazy<HashMap<String, FunctionDoc>> = Lazy::new(|| {
    let mut docs = HashMap::new();

    // ---- Arithmetic Operators ----
    docs.insert(
        "+".to_string(),
        FunctionDoc {
            name: "+".to_string(),
            signature: "(+ expr*)".to_string(),
            description: "Addition. Accepts zero or more arguments. Returns the sum of all arguments.".to_string(),
            example: Some("(+ 1 2 3 4) ;; => 10".to_string()),
        },
    );
    docs.insert(
        "-".to_string(),
        FunctionDoc {
            name: "-".to_string(),
            signature: "(- expr*)".to_string(),
            description: "Subtraction. With one argument, negates. With N arguments, subtracts all from the first.".to_string(),
            example: Some("(- 10 3) ;; => 7\n(- 42) ;; => -42".to_string()),
        },
    );
    docs.insert(
        "*".to_string(),
        FunctionDoc {
            name: "*".to_string(),
            signature: "(* expr*)".to_string(),
            description: "Multiplication. Accepts zero or more arguments. Returns the product.".to_string(),
            example: Some("(* 2 3 4) ;; => 24".to_string()),
        },
    );
    docs.insert(
        "/".to_string(),
        FunctionDoc {
            name: "/".to_string(),
            signature: "(/ a b)".to_string(),
            description: "Division. Returns a / b as a floating point number.".to_string(),
            example: Some("(/ 10 3) ;; => 3.3333333333333335".to_string()),
        },
    );
    docs.insert(
        "**".to_string(),
        FunctionDoc {
            name: "**".to_string(),
            signature: "(** base exponent)".to_string(),
            description: "Power/exponentiation. Returns base raised to the power of exponent.".to_string(),
            example: Some("(** 2 8) ;; => 256".to_string()),
        },
    );

    // ---- Logic Operators ----
    docs.insert(
        "!".to_string(),
        FunctionDoc {
            name: "!".to_string(),
            signature: "(! expr)".to_string(),
            description: "Logical NOT. Returns #t if expr is falsy (0, #f, empty list), #f otherwise.".to_string(),
            example: Some("(! #f) ;; => #t\n(! 0) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "&&".to_string(),
        FunctionDoc {
            name: "&&".to_string(),
            signature: "(&& expr*)".to_string(),
            description: "Logical AND. Returns true if ALL arguments are truthy (int > 0, #t, or non-empty list). 0 and #f are equivalent.".to_string(),
            example: Some("(&& #t #t #t) ;; => #t\n(&& #t #f) ;; => #f".to_string()),
        },
    );
    docs.insert(
        "||".to_string(),
        FunctionDoc {
            name: "||".to_string(),
            signature: "(|| expr*)".to_string(),
            description: "Logical OR. Returns true if ANY argument is truthy.".to_string(),
            example: Some("(|| #f #f #t) ;; => #t\n(|| 0 0) ;; => #f".to_string()),
        },
    );
    docs.insert(
        "=".to_string(),
        FunctionDoc {
            name: "=".to_string(),
            signature: "(= expr*)".to_string(),
            description: "Equality check. Returns #t if all arguments are equal. Works with ints, strings, booleans, and lists.".to_string(),
            example: Some("(= 5 5) ;; => #t\n(= '(1 2) '(1 2)) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "!=".to_string(),
        FunctionDoc {
            name: "!=".to_string(),
            signature: "(!= expr*)".to_string(),
            description: "Inequality check. Returns #t if any argument differs from the others.".to_string(),
            example: Some("(!= 1 2) ;; => #t\n(!= 5 5) ;; => #f".to_string()),
        },
    );

    // ---- Comparisons ----
    docs.insert(
        ">".to_string(),
        FunctionDoc {
            name: ">".to_string(),
            signature: "(> a b)".to_string(),
            description: "Greater than. Returns #t if a > b.".to_string(),
            example: Some("(> 5 3) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "<".to_string(),
        FunctionDoc {
            name: "<".to_string(),
            signature: "(< a b)".to_string(),
            description: "Less than. Returns #t if a < b.".to_string(),
            example: Some("(< 3 5) ;; => #t".to_string()),
        },
    );
    docs.insert(
        ">=".to_string(),
        FunctionDoc {
            name: ">=".to_string(),
            signature: "(>= a b)".to_string(),
            description: "Greater than or equal. Returns #t if a >= b.".to_string(),
            example: Some("(>= 5 5) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "<=".to_string(),
        FunctionDoc {
            name: "<=".to_string(),
            signature: "(<= a b)".to_string(),
            description: "Less than or equal. Returns #t if a <= b.".to_string(),
            example: Some("(<= 3 5) ;; => #t".to_string()),
        },
    );

    // ---- Program State ----
    docs.insert(
        "let".to_string(),
        FunctionDoc {
            name: "let".to_string(),
            signature: "(let ([id expr]+) body)".to_string(),
            description: "Local bindings. Binds results of exprs to ids during evaluation of body. Later bindings can reference earlier ones.".to_string(),
            example: Some("(let ([x 10] [y x]) (+ x y)) ;; => 20".to_string()),
        },
    );
    docs.insert(
        "set!".to_string(),
        FunctionDoc {
            name: "set!".to_string(),
            signature: "(set! id expr)".to_string(),
            description: "Update an existing binding. Evaluates expr and assigns the result to id. Returns the result.".to_string(),
            example: Some("(define x 10)\n(set! x (+ x x)) ;; => 20".to_string()),
        },
    );

    // ---- Functions ----
    docs.insert(
        "defun".to_string(),
        FunctionDoc {
            name: "defun".to_string(),
            signature: "(defun name args body+)".to_string(),
            description: "Define a named function. args can be a list of symbols (fixed arity) or a single symbol (variadic). body is one or more expressions.".to_string(),
            example: Some("(defun add [a b] (+ a b))\n(defun sum-all xs (fold + 0 xs))".to_string()),
        },
    );

    // ---- Control Flow ----
    docs.insert(
        "cond".to_string(),
        FunctionDoc {
            name: "cond".to_string(),
            signature: "(cond (guard expr+)+)".to_string(),
            description: "Multi-way conditional. Evaluates exprs of the first clause whose guard is truthy. Use #t as the last guard for a default case.".to_string(),
            example: Some("(cond [(= n 0) \"zero\"] [(= n 1) \"one\"] [#t \"other\"])".to_string()),
        },
    );
    docs.insert(
        "case".to_string(),
        FunctionDoc {
            name: "case".to_string(),
            signature: "(case key (value expr+)+)".to_string(),
            description: "Value-based branching. Matches key against each value. Use 'default' for the fallback case.".to_string(),
            example: Some("(case (+ a b) [1 \"one\"] [2 \"two\"] [default \"many\"])".to_string()),
        },
    );
    docs.insert(
        "when".to_string(),
        FunctionDoc {
            name: "when".to_string(),
            signature: "(when cond body+)".to_string(),
            description: "Evaluate body only when cond is truthy (int > 0, #t, non-empty list). Returns the last body expression or None.".to_string(),
            example: Some("(when (> x 0) (print \"positive\"))".to_string()),
        },
    );
    docs.insert(
        "unless".to_string(),
        FunctionDoc {
            name: "unless".to_string(),
            signature: "(unless cond body+)".to_string(),
            description: "Evaluate body only when cond is falsy (0, #f, empty list). Returns the last body expression or None.".to_string(),
            example: Some("(unless error (print \"All good!\"))".to_string()),
        },
    );
    docs.insert(
        "do".to_string(),
        FunctionDoc {
            name: "do".to_string(),
            signature: "(do body+)".to_string(),
            description: "Sequence evaluation. Evaluates all expressions in order, returns result of the last one. Useful for grouping multiple expressions where only one is allowed.".to_string(),
            example: Some("(do (define x 10) (print x) x)".to_string()),
        },
    );

    // ---- IO ----
    docs.insert(
        "printf".to_string(),
        FunctionDoc {
            name: "printf".to_string(),
            signature: "(printf format args*)".to_string(),
            description: "C-style formatted print. Uses Python printf-style formatting rules (e.g., %d, %s, %f).".to_string(),
            example: Some("(printf \"%d + %d = %d\" 2 3 (+ 2 3))".to_string()),
        },
    );
    docs.insert(
        "exit".to_string(),
        FunctionDoc {
            name: "exit".to_string(),
            signature: "(exit code)".to_string(),
            description: "Exit the WAL program with the given return code.".to_string(),
            example: Some("(exit 0)".to_string()),
        },
    );

    // ---- Utility ----
    docs.insert(
        "eval-file".to_string(),
        FunctionDoc {
            name: "eval-file".to_string(),
            signature: "(eval-file file)".to_string(),
            description: "Evaluate WAL code from file.wal and merge the resulting program state into the current state. Definitions in file can overwrite existing definitions.".to_string(),
            example: Some("(eval-file my-module)".to_string()),
        },
    );

    // ---- Waveform ----
    docs.insert(
        "load".to_string(),
        FunctionDoc {
            name: "load".to_string(),
            signature: "(load file [id])".to_string(),
            description: "Load a waveform file and register it in the WAL kernel. Supported formats: VCD, CSV, FST (requires pylibfst). If no id given, auto-generates t0, t1, ...".to_string(),
            example: Some("(load \"counter.fst\")".to_string()),
        },
    );
    docs.insert(
        "step".to_string(),
        FunctionDoc {
            name: "step".to_string(),
            signature: "(step [id] [amount])".to_string(),
            description: "Step trace by amount. If no id given, all traces are stepped. Returns #f if end of trace reached.".to_string(),
            example: Some("(step 1) ;; advance 1\n(step -1) ;; rewind 1".to_string()),
        },
    );
    docs.insert(
        "alias".to_string(),
        FunctionDoc {
            name: "alias".to_string(),
            signature: "(alias name signal)".to_string(),
            description: "Create an alias so signal can also be referenced by name. Aliases are compatible with groups and scopes.".to_string(),
            example: Some("(alias 'myclk 'tb.clk)".to_string()),
        },
    );
    docs.insert(
        "whenever".to_string(),
        FunctionDoc {
            name: "whenever".to_string(),
            signature: "(whenever cond body+)".to_string(),
            description: "Evaluate body on each waveform index where cond is true. Returns the last body value from the last matching index.".to_string(),
            example: Some("(whenever (= tb.overflow 1) (print INDEX))".to_string()),
        },
    );
    docs.insert(
        "find".to_string(),
        FunctionDoc {
            name: "find".to_string(),
            signature: "(find cond)".to_string(),
            description: "Returns a list of all waveform indices at which cond evaluates to true.".to_string(),
            example: Some("(find (= tb.overflow 1))".to_string()),
        },
    );
    docs.insert(
        "count".to_string(),
        FunctionDoc {
            name: "count".to_string(),
            signature: "(count cond)".to_string(),
            description: "Returns the number of indices at which cond evaluates to true.".to_string(),
            example: Some("(count (= tb.clk 1))".to_string()),
        },
    );
    docs.insert(
        "timeframe".to_string(),
        FunctionDoc {
            name: "timeframe".to_string(),
            signature: "(timeframe body+)".to_string(),
            description: "Saves the current INDEX of every loaded trace before evaluating body, then restores them after. Enables local time operations without losing position.".to_string(),
            example: Some("(timeframe (while (! ready) (step)) (print INDEX))".to_string()),
        },
    );

    // ---- Signal Access ----
    docs.insert(
        "get".to_string(),
        FunctionDoc {
            name: "get".to_string(),
            signature: "(get signal)".to_string(),
            description: "Returns the value of the specified signal at the current INDEX.".to_string(),
            example: Some("(get tb.clk)".to_string()),
        },
    );
    docs.insert(
        "slice".to_string(),
        FunctionDoc {
            name: "slice".to_string(),
            signature: "(slice signal upper lower)".to_string(),
            description: "Extracts bits or list elements from upper to lower (inclusive). For lists, follows Python slicing semantics.".to_string(),
            example: Some("(slice tb.data 7 0) ;; extract bits 7:0".to_string()),
        },
    );
    docs.insert(
        "reval".to_string(),
        FunctionDoc {
            name: "reval".to_string(),
            signature: "(reval expr offset)".to_string(),
            description: "Relative evaluation. Evaluates expr at current INDEX + offset. The @ macro is shorthand: expr@offset expands to (reval expr offset).".to_string(),
            example: Some("(reval INDEX -1) ;; INDEX at previous time\nINDEX@-1 ;; equivalent".to_string()),
        },
    );

    // ---- Groups and Scopes ----
    docs.insert(
        "groups".to_string(),
        FunctionDoc {
            name: "groups".to_string(),
            signature: "(groups posts*)".to_string(),
            description: "Returns all partial signal name prefixes for which prepending to each post yields a valid signal name. Used to find structural groups in the design hierarchy.".to_string(),
            example: Some("(groups \"valid\" \"ready\") ;; find all handshake groups".to_string()),
        },
    );
    docs.insert(
        "in-groups".to_string(),
        FunctionDoc {
            name: "in-groups".to_string(),
            signature: "(in-groups groups expr)".to_string(),
            description: "Evaluates expr in every group. The # prefix resolves to the current group's signal. CG gives the current group name.".to_string(),
            example: Some("(in-groups (groups \"valid\" \"ready\") (print CG \":\" INDEX))".to_string()),
        },
    );

    // ---- Lists ----
    docs.insert(
        "list".to_string(),
        FunctionDoc {
            name: "list".to_string(),
            signature: "(list expr*)".to_string(),
            description: "Creates a list from the evaluated expressions in order. Shorthand: '(a b c).".to_string(),
            example: Some("(list 1 2 3) ;; => (1 2 3)".to_string()),
        },
    );
    docs.insert(
        "map".to_string(),
        FunctionDoc {
            name: "map".to_string(),
            signature: "(map f xs)".to_string(),
            description: "Applies function f to each element of list xs, returning a new list of results.".to_string(),
            example: Some("(map (fn [x] (* x 2)) '(1 2 3)) ;; => (2 4 6)".to_string()),
        },
    );
    docs.insert(
        "fold".to_string(),
        FunctionDoc {
            name: "fold".to_string(),
            signature: "(fold f init xs)".to_string(),
            description: "Left fold (reduce). Folds list xs using binary function f, starting from init. Each step: acc = (f acc x).".to_string(),
            example: Some("(fold + 0 '(1 2 3 4 5)) ;; => 15".to_string()),
        },
    );
    docs.insert(
        "zip".to_string(),
        FunctionDoc {
            name: "zip".to_string(),
            signature: "(zip xs ys)".to_string(),
            description: "Returns a list of pairs combining elements from xs and ys.".to_string(),
            example: Some("(zip '(1 2 3) '(a b c)) ;; => ((1 a) (2 b) (3 c))".to_string()),
        },
    );

    // ---- Arrays ----
    docs.insert(
        "array".to_string(),
        FunctionDoc {
            name: "array".to_string(),
            signature: "(array (id expr)*)".to_string(),
            description: "Creates a hashmap array. Keys are stored as strings. Initialized with key-value pairs. Printed as {(\"key\" value) ...}.".to_string(),
            example: Some("(array ['x 10] ['y 20]) ;; => {(\"x\" 10) (\"y\" 20)}".to_string()),
        },
    );
    docs.insert(
        "seta".to_string(),
        FunctionDoc {
            name: "seta".to_string(),
            signature: "(seta array key value)".to_string(),
            description: "Insert or update a value in the array. Key is converted to string. Returns the updated array.".to_string(),
            example: Some("(seta (array) 'x 10) ;; => {(\"x\" 10)}".to_string()),
        },
    );
    docs.insert(
        "geta".to_string(),
        FunctionDoc {
            name: "geta".to_string(),
            signature: "(geta array key)".to_string(),
            description: "Retrieve a value from the array by key. Key is converted to string. Error if key not found.".to_string(),
            example: Some("(geta (array ['x 10]) 'x) ;; => 10".to_string()),
        },
    );
    docs.insert(
        "geta/default".to_string(),
        FunctionDoc {
            name: "geta/default".to_string(),
            signature: "(geta/default array default key)".to_string(),
            description: "Retrieve value from array by key, or return default if key is not found.".to_string(),
            example: Some("(geta/default (array ['x 10]) 5 'y) ;; => 5".to_string()),
        },
    );

    // ---- Types ----
    docs.insert(
        "convert/bin".to_string(),
        FunctionDoc {
            name: "convert/bin".to_string(),
            signature: "(convert/bin x width)".to_string(),
            description: "Converts integer x to a binary string representation with width bits.".to_string(),
            example: Some("(convert/bin 5 8) ;; => \"00000101\"".to_string()),
        },
    );

    // ---- Existing entries (kept) ----
    docs.insert(
        "define".to_string(),
        FunctionDoc {
            name: "define".to_string(),
            signature: "(define id expr)".to_string(),
            description: "Define a global variable. Evaluates expr and binds the result to id. Returns the result.".to_string(),
            example: Some("(define x 42)\n(define add (fn [a b] (+ a b)))".to_string()),
        },
    );
    docs.insert(
        "fn".to_string(),
        FunctionDoc {
            name: "fn".to_string(),
            signature: "(fn [args] body+)".to_string(),
            description: "Create an anonymous function. args is a parameter list. body is one or more expressions. Returns the created function (supports closures).".to_string(),
            example: Some("(fn [x y] (+ x y))".to_string()),
        },
    );
    docs.insert(
        "if".to_string(),
        FunctionDoc {
            name: "if".to_string(),
            signature: "(if cond then else)".to_string(),
            description: "Conditional branching. Evaluates then if cond is truthy (int > 0, #t, non-empty list), else evaluates else. Both then and else are single expressions; use do for multiple.".to_string(),
            example: Some("(if (> x 0) (print \"positive\") (print \"non-positive\"))".to_string()),
        },
    );
    docs.insert(
        "while".to_string(),
        FunctionDoc {
            name: "while".to_string(),
            signature: "(while cond body+)".to_string(),
            description: "Loop while cond remains truthy. Evaluates body on each iteration. Useful with step for waveform traversal.".to_string(),
            example: Some("(while (step 1)\n  (when (= INDEX 100)\n    (print \"Found!\")))".to_string()),
        },
    );
    docs.insert(
        "print".to_string(),
        FunctionDoc {
            name: "print".to_string(),
            signature: "(print args*)".to_string(),
            description: "Print all arguments to stdout. Evaluates args in order, prints them separated by spaces, appends newline.".to_string(),
            example: Some("(print \"Index: \" INDEX)".to_string()),
        },
    );
    docs.insert(
        "signal?".to_string(),
        FunctionDoc {
            name: "signal?".to_string(),
            signature: "(signal? name)".to_string(),
            description: "Checks whether name is a signal in any loaded waveform.".to_string(),
            example: Some("(signal? \"tb.clk\")".to_string()),
        },
    );

    // ---- List accessors ----
    docs.insert(
        "first".to_string(),
        FunctionDoc {
            name: "first".to_string(),
            signature: "(first xs)".to_string(),
            description: "Returns the first element of list xs.".to_string(),
            example: Some("(first '(10 20 30)) ;; => 10".to_string()),
        },
    );
    docs.insert(
        "second".to_string(),
        FunctionDoc {
            name: "second".to_string(),
            signature: "(second xs)".to_string(),
            description: "Returns the second element of list xs.".to_string(),
            example: Some("(second '(10 20 30)) ;; => 20".to_string()),
        },
    );
    docs.insert(
        "last".to_string(),
        FunctionDoc {
            name: "last".to_string(),
            signature: "(last xs)".to_string(),
            description: "Returns the last element of list xs.".to_string(),
            example: Some("(last '(1 2 3 4 5)) ;; => 5".to_string()),
        },
    );
    docs.insert(
        "rest".to_string(),
        FunctionDoc {
            name: "rest".to_string(),
            signature: "(rest xs)".to_string(),
            description: "Returns a list containing all but the first element of xs.".to_string(),
            example: Some("(rest '(1 2 3)) ;; => (2 3)".to_string()),
        },
    );
    docs.insert(
        "in".to_string(),
        FunctionDoc {
            name: "in".to_string(),
            signature: "(in x xs)".to_string(),
            description: "Membership test. Returns true if x is an element in list xs.".to_string(),
            example: Some("(in 2 '(1 2 3)) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "min".to_string(),
        FunctionDoc {
            name: "min".to_string(),
            signature: "(min xs)".to_string(),
            description: "Returns the smallest element in list xs.".to_string(),
            example: Some("(min '(1 5 3 9 2)) ;; => 1".to_string()),
        },
    );
    docs.insert(
        "max".to_string(),
        FunctionDoc {
            name: "max".to_string(),
            signature: "(max xs)".to_string(),
            description: "Returns the largest element in list xs.".to_string(),
            example: Some("(max '(1 5 3 9 2)) ;; => 9".to_string()),
        },
    );
    docs.insert(
        "sum".to_string(),
        FunctionDoc {
            name: "sum".to_string(),
            signature: "(sum xs)".to_string(),
            description: "Returns the sum of all elements in list xs.".to_string(),
            example: Some("(sum '(1 2 3 4 5)) ;; => 15".to_string()),
        },
    );
    docs.insert(
        "average".to_string(),
        FunctionDoc {
            name: "average".to_string(),
            signature: "(average xs)".to_string(),
            description: "Returns the arithmetic mean of all elements in list xs.".to_string(),
            example: Some("(average '(1 2 3 4 5)) ;; => 3".to_string()),
        },
    );
    docs.insert(
        "length".to_string(),
        FunctionDoc {
            name: "length".to_string(),
            signature: "(length xs)".to_string(),
            description: "Returns the number of elements in list xs.".to_string(),
            example: Some("(length '(a b c)) ;; => 3".to_string()),
        },
    );

    // ---- Waveform: unload, alias, unalias ----
    docs.insert(
        "unload".to_string(),
        FunctionDoc {
            name: "unload".to_string(),
            signature: "(unload id)".to_string(),
            description: "Removes the waveform specified by id from the WAL kernel.".to_string(),
            example: Some("(unload t0)".to_string()),
        },
    );
    docs.insert(
        "alias".to_string(),
        FunctionDoc {
            name: "alias".to_string(),
            signature: "(alias name signal)".to_string(),
            description: "Create an alias so signal can also be referenced by name. Aliases are compatible with groups and scopes.".to_string(),
            example: Some("(alias 'myclk 'tb.clk)".to_string()),
        },
    );
    docs.insert(
        "unalias".to_string(),
        FunctionDoc {
            name: "unalias".to_string(),
            signature: "(unalias name)".to_string(),
            description: "Removes the alias identified by name.".to_string(),
            example: Some("(unalias 'myclk)".to_string()),
        },
    );

    // ---- Groups and Scopes ----
    docs.insert(
        "resolve-group".to_string(),
        FunctionDoc {
            name: "resolve-group".to_string(),
            signature: "(resolve-group name)".to_string(),
            description: "Evaluates signal name appended by CG (current group) and returns the signal value at the current INDEX. Equivalent to the #name shorthand.".to_string(),
            example: Some("(resolve-group #valid) ;; eval CG+\"valid\"".to_string()),
        },
    );
    docs.insert(
        "in-scopes".to_string(),
        FunctionDoc {
            name: "in-scopes".to_string(),
            signature: "(in-scopes scopes body+)".to_string(),
            description: "Evaluates body in every scope in scopes.".to_string(),
            example: Some("(in-scopes (all-scopes) (print CS \":\" ~clk))".to_string()),
        },
    );
    docs.insert(
        "all-scopes".to_string(),
        FunctionDoc {
            name: "all-scopes".to_string(),
            signature: "(all-scopes)".to_string(),
            description: "Returns a list of all available scopes.".to_string(),
            example: Some("(all-scopes)".to_string()),
        },
    );

    // ---- Arrays ----
    docs.insert(
        "dela".to_string(),
        FunctionDoc {
            name: "dela".to_string(),
            signature: "(dela array key)".to_string(),
            description: "Removes the value at key from array. Key is converted to string.".to_string(),
            example: Some("(dela (array ['x 10] ['y 20]) 'x) ;; => {(\"y\" 20)}".to_string()),
        },
    );
    docs.insert(
        "mapa".to_string(),
        FunctionDoc {
            name: "mapa".to_string(),
            signature: "(mapa f array)".to_string(),
            description: "Applies function f to every (key value) pair in array. f must take exactly two parameters: key and value. Returns a list.".to_string(),
            example: Some("(mapa (fn [k v] (list k v)) (array ['x 10] ['y 20]))".to_string()),
        },
    );

    // ---- Type predicates ----
    docs.insert(
        "atom?".to_string(),
        FunctionDoc {
            name: "atom?".to_string(),
            signature: "(atom? x)".to_string(),
            description: "Returns true if x is an atom (symbol, integer, boolean, or string). Returns false for lists and arrays.".to_string(),
            example: Some("(atom? 42) ;; => #t\n(atom? '(1 2)) ;; => #f".to_string()),
        },
    );
    docs.insert(
        "symbol?".to_string(),
        FunctionDoc {
            name: "symbol?".to_string(),
            signature: "(symbol? x)".to_string(),
            description: "Returns true if x is a symbol.".to_string(),
            example: Some("(symbol? 'hello) ;; => #t".to_string()),
        },
    );
    docs.insert(
        "string?".to_string(),
        FunctionDoc {
            name: "string?".to_string(),
            signature: "(string? x)".to_string(),
            description: "Returns true if x is a string.".to_string(),
            example: Some("(string? \"hello\") ;; => #t".to_string()),
        },
    );
    docs.insert(
        "int?".to_string(),
        FunctionDoc {
            name: "int?".to_string(),
            signature: "(int? x)".to_string(),
            description: "Returns true if x is an integer.".to_string(),
            example: Some("(int? 42) ;; => #t\n(int? 3.14) ;; => #f".to_string()),
        },
    );
    docs.insert(
        "list?".to_string(),
        FunctionDoc {
            name: "list?".to_string(),
            signature: "(list? x)".to_string(),
            description: "Returns true if x is a list.".to_string(),
            example: Some("(list? '(1 2 3)) ;; => #t".to_string()),
        },
    );

    docs
});

pub fn get_doc(name: &str) -> Option<FunctionDoc> {
    FUNCTION_DOCS.get(name).cloned()
}
