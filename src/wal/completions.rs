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
];

pub const SPECIAL_FORMS: &[(&str, &str)] = &[
    ("define", "定义变量或函数 (define name value)"),
    ("set", "设置变量值 (set name value)"),
    ("let", "局部绑定 (let ([x 1]) body)"),
    ("fn", "定义函数 (fn [args] body)"),
    ("defmacro", "定义宏 (defmacro args body)"),
    ("if", "条件表达式 (if cond then else)"),
    ("while", "循环 (while cond body)"),
    ("for", "for 循环 (for [x list] body)"),
    ("for/list", "列表推导 (for/list [x xs] body) — collect results into list"),
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
    ("filter", "过滤列表 (filter pred list)"),
    ("sort", "排序列表 (sort list)"),
    ("reverse", "反转列表 (reverse list)"),
    ("in", "成员检查 (in x xs) — returns true if x is element of xs"),
    ("append", "追加元素到列表 (append list x)"),
    ("zip", "合并两个列表 (zip list1 list2)"),
    ("range", "生成范围列表 (range start end)"),
    ("slice", "切片 (slice list start end)"),
    ("get", "获取元素 (get list index)"),
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
    // 类型转换
    ("string->int", "字符串转整数 (string->int str)"),
    ("int->string", "整数转字符串 (int->string n)"),
    ("symbol->string", "符号转字符串 (symbol->string sym)"),
    ("string->symbol", "字符串转符号 (string->symbol str)"),
    ("convert/bin", "转换为二进制 (convert/bin signal width)"),
    ("bits->sint", "位向量转有符号整数 (bits->sint bits)"),
    // 波形操作
    ("load", "加载波形文件 (load filename [tid])"),
    ("unload", "卸载波形 (unload id)"),
    ("step", "步进时间索引 (step [n])"),
    ("alias", "创建信号别名 (alias name signal)"),
    ("unalias", "删除信号别名 (unalias name)"),
    ("signal-width", "获取信号位宽 (signal-width name)"),
    ("sample-at", "在指定时间采样 (sample-at signal time)"),
    ("reval", "相对求值 (reval signal offset)"),
    ("whenever", "当条件为真时求值 (whenever cond body)"),
    ("find", "查找满足条件的索引 (find cond)"),
    ("find/g", "全局查找 (find/g cond)"),
    ("trim-trace", "裁剪波形 (trim-trace start end)"),
    ("in-scope", "切换到指定作用域 (in-scope scope body)"),
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
    ("LOCAL-SIGNALS", "当前作用域信号 (特殊变量)"),
    ("VIRTUAL-SIGNALS", "虚拟信号列表 (特殊变量)"),
    ("TRACE-FILE", "当前波形文件路径 (特殊变量)"),
    ("TRACE-NAME", "当前波形名称 (特殊变量)"),
    ("TS", "当前时间戳 (特殊变量)"),
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
    ("step-until", "步进直到条件满足 (step-until cond)"),
    ("step-while", "步进当条件满足 (step-while cond)"),
    ("always", "总是执行 (always body...)"),
    ("defunm", "设置命名的宏 (defunm name [args] body)"),
    ("car", "取列表头部 (car xs)"),
    ("cdr", "取列表尾部 (cdr xs)"),
    ("cadr", "取列表第二个元素 (cadr xs)"),
    ("partition", "分区 (partition pred xs)"),
    ("inc-define", "增量定义 (inc-define sym)"),
    ("inc", "增量 (inc sym)"),
    ("dec", "减量 (dec sym)"),
    ("sum", "求和 (sum list)"),
    ("timeframe", "时间范围 (timeframe body)"),
    ("rising", "上升沿 (rising expr)"),
    ("falling", "下降沿 (falling expr)"),
    ("unstable", "不稳定信号 (unstable expr)"),
    ("stable", "稳定信号 (stable expr)"),
    ("signed", "有符号信号 (signed signal)"),
    ("count", "计数 (count cond)"),
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

pub fn get_all_completions() -> Vec<CompletionItem> {
    ALL_COMPLETIONS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_completions() {
        let items = get_all_completions();
        assert!(!items.is_empty());
        assert!(items.iter().any(|i| i.label == "load"));
        assert!(items.iter().any(|i| i.label == "+"));
    }
}
