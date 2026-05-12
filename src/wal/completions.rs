use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionKind {
    Keyword,
    Function,
    Operator,
    Variable,
    Signal,
}

pub const OPERATORS: &[(&str, &str)] = &[
    ("+", "加法 (add)"),
    ("-", "减法 (subtract)"),
    ("*", "乘法 (multiply)"),
    ("/", "除法 (divide)"),
    ("**", "幂运算 (power)"),
    ("=", "相等 (equal)"),
    ("!=", "不等 (not equal)"),
    (">", "大于 (greater than)"),
    ("<", "小于 (less than)"),
    (">=", "大于等于"),
    ("<=", "小于等于"),
    ("!", "逻辑非 (logical not)"),
    ("&&", "逻辑与 (logical and)"),
    ("||", "逻辑或 (logical or)"),
    ("not", "逻辑非 (logical not) — ! 的别名"),
    ("and", "逻辑与 (logical and) — && 的别名"),
    ("or", "逻辑或 (logical or) — || 的别名"),
    ("bor", "按位或 (bitwise or)"),
    ("band", "按位与 (bitwise and)"),
    ("bxor", "按位异或 (bitwise xor)"),
];

pub const SPECIAL_FORMS: &[(&str, &str)] = &[
    ("define", "定义变量或函数 (define name value)"),
    ("set", "设置变量值 (set name value)"),
    ("let", "局部绑定 (let ([x 1]) body)"),
    ("fn", "定义函数 (fn [args] body)"),
    ("defmacro", "定义宏 (defmacro args body)"),
    ("if", "条件表达式 (if cond then else)"),
    ("while", "循环 (while cond body)"),

    ("cond", "多分支条件 (cond [cond1 body1] [cond2 body2])"),
    ("case", "值匹配分支 (case key [value expr+] [default expr+])"),
    ("quote", "引用 (阻止求值) (quote expr)"),
    ("quasiquote", "准引用 (quasiquote expr)"),
    ("unquote", "解引用 (unquote expr)"),
    ("eval", "求值 (eval expr)"),
    ("parse", "解析字符串为 AST (parse string)"),
    ("macroexpand", "宏展开 (macroexpand expr)"),
    ("gensym", "生成唯一符号 (gensym)"),
    ("list", "创建列表 (list args...)"),
    ("array", "创建数组 (array args...)"),
    ("do", "执行序列 (do expr...)"),
    ("require", "加载模块 (require module)"),
    ("import", "导入绑定 (import module)"),
    ("eval-file", "求值外部文件 (eval-file file) — evaluates file.wal, merges state"),
    ("resolve-group", "解析组信号 (resolve-group name) — 等价于 #name"),
    ("unquote-splice", "拼接解引用 (unquote-splice expr) — ,@ 的函数形式"),
];

pub const BUILTIN_FUNCTIONS: &[(&str, &str)] = &[
    // 列表操作
    ("first", "返回列表第一个元素 (first list)"),
    ("second", "返回列表第二个元素 (second list)"),
    ("last", "返回列表最后一个元素 (last list)"),
    ("rest", "返回列表除第一个外的部分 (rest list)"),
    ("length", "返回列表长度 (length list)"),
    ("map", "对列表每个元素应用函数 (map fn list)"),
    ("fold", "折叠列表 (fold fn init list)"),
    ("in", "成员检查 (in x xs) — returns true if x is element of xs"),
    ("zip", "合并两个列表 (zip list1 list2)"),
    ("range", "生成范围列表 (range start end)"),
    ("slice", "切片 (slice list start end)"),
    ("get", "获取元素 (get list index)"),
    ("caar", "取列表第一个元素的第一个元素 (caar xs)"),
    ("cddr", "取列表除前两个元素外的部分 (cddr xs)"),
    ("call", "调用函数 (call fn args...)"),
    ("seta", "设置数组元素 (seta array index value)"),
    ("geta", "获取数组元素 (geta array index)"),
    ("geta/default", "获取数组元素带默认值 (geta/default array default key)"),
    ("dela", "删除数组元素 (dela array index)"),
    ("mapa", "映射数组 (mapa fn array)"),
    // 数学
    ("max", "列表最大值 (max xs) — returns largest element in list"),
    ("min", "列表最小值 (min xs) — returns smallest element in list"),
    ("average", "平均值 (average list)"),
    ("floor", "向下取整 (floor n)"),
    ("ceil", "向上取整 (ceil n)"),
    ("round", "四舍五入 (round n)"),
    ("mod", "取模 (mod a b)"),
    ("abs", "绝对值 (abs n)"),
    // 类型检查
    ("atom?", "是否为原子 (atom? x)"),
    ("symbol?", "是否为符号 (symbol? x)"),
    ("string?", "是否为字符串 (string? x)"),
    ("int?", "是否为整数 (int? x)"),
    ("list?", "是否为列表 (list? x)"),
    ("defined?", "是否定义 (defined? name)"),
    ("signal?", "是否为信号 (signal? name)"),
    ("null?", "是否为空 (null? x)"),
    ("type", "返回值的类型 (type x)"),
    // 类型转换
    ("string->int", "字符串转整数 (string->int str)"),
    ("int->string", "整数转字符串 (int->string n)"),
    ("symbol->string", "符号转字符串 (symbol->string sym)"),
    ("string->symbol", "字符串转符号 (string->symbol str)"),
    ("convert/bin", "转换为二进制 (convert/bin signal width)"),
    ("bits->sint", "位向量转有符号整数 (bits->sint bits)"),
    ("string-append", "追加字符串 (string-append str...) — wal-rust 扩展"),
    // 波形操作
    ("load", "加载波形文件 (load filename [tid])"),
    ("unload", "卸载波形 (unload id)"),
    ("step", "步进时间索引 (step [n])"),
    ("alias", "创建信号别名 (alias name signal)"),
    ("unalias", "删除信号别名 (unalias name)"),
    ("signal-width", "获取信号位宽 (signal-width name)"),
    ("sample-at", "在指定时间采样 (sample-at signal time)"),
    ("reval", "相对求值 (reval signal offset)"),
    ("rel_eval", "相对求值函数形式 (rel_eval expr offset) — reval 的别名"),
    ("whenever", "当条件为真时求值 (whenever cond body)"),
    ("repl", "启动交互式 REPL (repl)"),
    ("loaded-traces", "已加载的波形列表 (loaded-traces)"),
    ("fold/signal", "叠加信号 (fold/signal f init signal)"),
    ("new-trace", "创建新波形 (new-trace)"),
    ("dump-trace", "导出波形 (dump-trace id)"),
    ("defsig", "定义虚拟信号 (defsig name expr)"),
    ("find", "查找满足条件的索引 (find cond)"),
    ("find/g", "全局查找 (find/g cond)"),
    ("trim-trace", "裁剪波形 (trim-trace start end)"),
    ("in-scope", "切换到指定作用域 (in-scope scope body)"),
    ("scoped", "在当前作用域中查找信号 (scoped name)"),
    ("resolve-scope", "解析作用域中的信号 (resolve-scope scope name)"),
    ("set-scope", "设置当前作用域 (set-scope scope)"),
    ("unset-scope", "取消当前作用域 (unset-scope)"),
    ("in-scopes", "在多个作用域中求值 (in-scopes scopes body)"),
    ("in-group", "切换到指定组 (in-group group body)"),
    ("groups", "获取所有组 (groups)"),
    ("in-groups", "在所有组中求值 (in-groups groups expr)"),
    ("all-scopes", "获取所有作用域 (all-scopes)"),
    // 特殊变量
    ("SIGNALS", "所有信号列表 (特殊变量)"),
    ("INDEX", "当前时间索引 (特殊变量)"),
    ("MAX-INDEX", "最大时间索引 (特殊变量)"),
    ("CS", "当前作用域 (特殊变量)"),
    ("CG", "当前组 (特殊变量) — Current Group"),
    ("LOCAL-SIGNALS", "当前作用域信号 (特殊变量)"),
    ("VIRTUAL-SIGNALS", "虚拟信号列表 (特殊变量)"),
    ("TRACE-FILE", "当前波形文件路径 (特殊变量)"),
    ("TRACE-NAME", "当前波形名称 (特殊变量)"),
    ("TS", "当前时间戳 (特殊变量)"),
    ("signals", "所有信号列表 (特殊变量) — SIGNALS 的小写别名"),
    ("index", "当前时间索引 (特殊变量) — INDEX 的小写别名"),
    ("max-index", "最大时间索引 (特殊变量) — MAX-INDEX 的小写别名"),
    ("ts", "当前时间戳 (特殊变量) — TS 的小写别名"),
    ("trace-name", "当前波形名称 (特殊变量) — TRACE-NAME 的小写别名"),
    ("trace-file", "当前波形文件路径 (特殊变量) — TRACE-FILE 的小写别名"),
    // IO
    ("print", "打印 (print args...)"),
    ("printf", "格式化打印 (printf format args...)"),
    ("exit", "退出程序 (exit [code])"),
];

pub const MACROS: &[(&str, &str)] = &[
    ("defun", "定义函数语法糖 (defun name [args] body)"),
    ("when", "当条件为真时执行 (when cond body...)"),
    ("unless", "当条件为假时执行 (unless cond body...)"),
    ("dowhile", "do-while 循环 (dowhile body... cond)"),
    ("until", "until 循环 (until cond body...)"),
    ("car", "取列表头部 (car xs)"),
    ("cdr", "取列表尾部 (cdr xs)"),
    ("cadr", "取列表第二个元素 (cadr xs)"),
    ("inc", "增量 (inc sym)"),
    ("dec", "减量 (dec sym)"),
    ("sum", "求和 (sum list)"),
    ("timeframe", "时间范围 (timeframe body)"),
    ("rising", "上升沿 (rising expr)"),
    ("falling", "下降沿 (falling expr)"),
    ("count", "计数 (count cond) — 多参数返回对应列表"),
    ("set!", "设置变量 (set! key value)"),
];

static ALL_COMPLETIONS: Lazy<Vec<CompletionItem>> = Lazy::new(|| {
    let mut items = Vec::new();

    for (name, detail) in OPERATORS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionKind::Operator,
            detail: Some(detail.to_string()),
            documentation: None,
        });
    }

    for (name, detail) in SPECIAL_FORMS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionKind::Keyword,
            detail: Some(detail.to_string()),
            documentation: None,
        });
    }

    for (name, detail) in BUILTIN_FUNCTIONS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionKind::Function,
            detail: Some(detail.to_string()),
            documentation: None,
        });
    }

    for (name, detail) in MACROS {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: CompletionKind::Function,
            detail: Some(detail.to_string()),
            documentation: None,
        });
    }

    items
});

#[allow(dead_code)]
pub fn get_all_completions() -> Vec<CompletionItem> {
    ALL_COMPLETIONS.clone()
}

pub fn get_all_completions_ref() -> &'static Vec<CompletionItem> {
    &ALL_COMPLETIONS
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_has_completions() {
        let items = get_all_completions();
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.label == "load"));
        assert!(items.iter().any(|i| i.label == "+"));
    }

    #[test]
    fn test_all_operators_present() {
        let items = get_all_completions();
        let labels: HashSet<&str> = items.iter().map(|i| i.label.as_str()).collect();
        for op in &["+", "-", "*", "/", "**", "=", "!=", ">", "<", ">=", "<=", "!", "&&", "||"] {
            assert!(labels.contains(op), "Missing operator completion: {}", op);
        }
    }

    #[test]
    fn test_key_special_forms_present() {
        let items = get_all_completions();
        let labels: HashSet<&str> = items.iter().map(|i| i.label.as_str()).collect();
        for sf in &["define", "let", "fn", "if", "cond", "case", "quote", "do",
                      "while", "list", "array", "eval-file"] {
            assert!(labels.contains(sf), "Missing special form completion: {}", sf);
        }
    }

    #[test]
    fn test_key_builtin_functions_present() {
        let items = get_all_completions();
        let labels: HashSet<&str> = items.iter().map(|i| i.label.as_str()).collect();
        for f in &["first", "second", "last", "rest", "length", "map", "fold",
                     "in", "zip", "max", "min", "average", "floor", "ceil", "round", "mod",
                     "abs", "load", "unload", "step", "get", "seta", "geta", "mapa",
                     "print", "printf", "exit", "atom?", "symbol?", "string?", "int?", "list?"] {
            assert!(labels.contains(f), "Missing builtin function completion: {}", f);
        }
    }

    #[test]
    fn test_known_symbols_consistency() {
        let items = get_all_completions();
        let labels: HashSet<&str> = items.iter().map(|i| i.label.as_str()).collect();

        for (name, _) in OPERATORS {
            assert!(labels.contains(name), "OPERATORS item '{}' not in completions", name);
        }
        for (name, _) in SPECIAL_FORMS {
            assert!(labels.contains(name), "SPECIAL_FORMS item '{}' not in completions", name);
        }
        for (name, _) in BUILTIN_FUNCTIONS {
            assert!(labels.contains(name), "BUILTIN_FUNCTIONS item '{}' not in completions", name);
        }
        for (name, _) in MACROS {
            assert!(labels.contains(name), "MACROS item '{}' not in completions", name);
        }
    }

    // ---- 补全过滤/排序测试 ----

    fn filter_by_prefix(prefix: &str) -> Vec<CompletionItem> {
        get_all_completions()
            .into_iter()
            .filter(|c| c.label.starts_with(prefix))
            .collect()
    }

    #[test]
    fn test_completion_filter_exact_match() {
        let results = filter_by_prefix("+");
        assert_eq!(results.len(), 1, "Only '+' should match prefix '+'");
        assert_eq!(results[0].label, "+");
    }

    #[test]
    fn test_completion_filter_prefix_l() {
        let results = filter_by_prefix("l");
        assert!(results.iter().any(|c| c.label == "load"));
        assert!(results.iter().any(|c| c.label == "let"));
        assert!(results.iter().any(|c| c.label == "length"));
        assert!(results.iter().any(|c| c.label == "list"));
        assert!(results.iter().any(|c| c.label == "list?"));
    }

    #[test]
    fn test_completion_filter_prefix_a() {
        let results = filter_by_prefix("a");
        assert!(results.iter().any(|c| c.label == "array"));
        assert!(results.iter().any(|c| c.label == "abs"));
        assert!(results.iter().any(|c| c.label == "atom?"));
        assert!(results.iter().any(|c| c.label == "average"));
        assert!(results.iter().any(|c| c.label == "all-scopes"));
    }

    #[test]
    fn test_completion_filter_prefix_d() {
        let results = filter_by_prefix("d");
        assert!(results.iter().any(|c| c.label == "define"));
        assert!(results.iter().any(|c| c.label == "defun"));
        assert!(results.iter().any(|c| c.label == "defined?"));
        assert!(results.iter().any(|c| c.label == "do"));
        assert!(results.iter().any(|c| c.label == "dela"));
    }

    #[test]
    fn test_completion_filter_prefix_s() {
        let results = filter_by_prefix("s");
        assert!(results.iter().any(|c| c.label == "set!"));
        assert!(results.iter().any(|c| c.label == "step"));
        assert!(results.iter().any(|c| c.label == "sum"));
        assert!(results.iter().any(|c| c.label == "slice"));
        assert!(results.iter().any(|c| c.label == "signal?"));
    }

    #[test]
    fn test_completion_filter_prefix_in() {
        let results = filter_by_prefix("in");
        assert!(results.iter().any(|c| c.label == "int?"));
        assert!(results.iter().any(|c| c.label == "in"));
    }

    #[test]
    fn test_completion_filter_empty_prefix_returns_all() {
        let all = get_all_completions();
        let filtered = filter_by_prefix("");
        assert_eq!(filtered.len(), all.len());
    }

    #[test]
    fn test_completion_filter_no_match() {
        let results = filter_by_prefix("zzz");
        assert!(results.is_empty(), "No completion should match 'zzz'");
    }

    #[test]
    fn test_completion_counts_by_kind() {
        let items = get_all_completions();
        let mut keyword_count = 0;
        let mut function_count = 0;
        let mut operator_count = 0;
        let mut variable_count = 0;
        for item in &items {
            match item.kind {
                CompletionKind::Keyword => keyword_count += 1,
                CompletionKind::Function => function_count += 1,
                CompletionKind::Operator => operator_count += 1,
                CompletionKind::Variable => variable_count += 1,
                _ => {}
            }
        }
        assert!(operator_count >= 10, "Expected >=10 operators, got {}", operator_count);
        assert!(keyword_count >= 15, "Expected >=15 keywords, got {}", keyword_count);
        assert!(function_count >= 60, "Expected >=60 functions, got {}", function_count);
        let total = keyword_count + function_count + operator_count + variable_count;
        assert!(total >= 80, "Expected >=80 total completions, got {}", total);
    }

    #[test]
    fn test_completion_filter_prefix_si() {
        let results = filter_by_prefix("si");
        assert!(results.iter().any(|c| c.label == "signal?"));
        assert!(results.iter().any(|c| c.label == "signal-width"));
    }

    #[test]
    fn test_completion_filter_prefix_co() {
        let results = filter_by_prefix("co");
        assert!(results.iter().any(|c| c.label == "cond"));
        assert!(results.iter().any(|c| c.label == "convert/bin"));
    }

    #[test]
    fn test_completion_filter_prefix_ma() {
        let results = filter_by_prefix("ma");
        assert!(results.iter().any(|c| c.label == "map"));
        assert!(results.iter().any(|c| c.label == "max"));
        assert!(results.iter().any(|c| c.label == "mapa"));
    }

    #[test]
    fn test_completion_label_uniqueness() {
        let items = get_all_completions();
        let mut labels: HashSet<String> = HashSet::new();
        for item in &items {
            labels.insert(item.label.clone());
        }
        assert_eq!(labels.len(), items.len(), "All labels must be unique");
    }

    #[test]
    fn test_special_variables_present() {
        let items = get_all_completions();
        let labels: HashSet<&str> = items.iter().map(|i| i.label.as_str()).collect();
        for v in &["SIGNALS", "INDEX", "MAX-INDEX", "CS", "CG", "LOCAL-SIGNALS",
                     "VIRTUAL-SIGNALS", "TRACE-FILE", "TRACE-NAME", "TS"] {
            assert!(labels.contains(v), "Missing special variable completion: {}", v);
        }
    }

    #[test]
    fn test_no_duplicate_labels() {
        let items = get_all_completions();
        let mut seen = HashSet::new();
        for item in &items {
            assert!(seen.insert(&item.label),
                "Duplicate completion label: {}", item.label);
        }
    }

    #[test]
    fn test_completion_items_have_valid_kinds() {
        let items = get_all_completions();
        for item in &items {
            assert!(!item.label.is_empty(), "Empty label in completion");
            // detail is optional but if present should not be empty
            if let Some(ref d) = item.detail {
                assert!(!d.is_empty(), "Empty detail for '{}'", item.label);
            }
        }
    }
}
