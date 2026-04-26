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

    docs.insert(
        "load".to_string(),
        FunctionDoc {
            name: "load".to_string(),
            signature: "(load filename [tid])".to_string(),
            description:
                "加载波形文件并注册到 WAL 运行时。支持的格式包括 VCD、CSV 和 FST（需要 pylibfst）。"
                    .to_string(),
            example: Some("(load \"counter.fst\")".to_string()),
        },
    );

    docs.insert(
        "step".to_string(),
        FunctionDoc {
            name: "step".to_string(),
            signature: "(step [n])".to_string(),
            description: "步进时间索引 n 个单位。默认步进 1。如果到达波形末尾返回 #f。".to_string(),
            example: Some("(step 1) ;; 步进 1\n(step -1) ;; 后退 1".to_string()),
        },
    );

    docs.insert(
        "define".to_string(),
        FunctionDoc {
            name: "define".to_string(),
            signature: "(define name value)".to_string(),
            description: "在全局作用域定义一个变量或函数。".to_string(),
            example: Some("(define x 42)\n(define add (fn [a b] (+ a b)))".to_string()),
        },
    );

    docs.insert(
        "fn".to_string(),
        FunctionDoc {
            name: "fn".to_string(),
            signature: "(fn [args] body...)".to_string(),
            description: "定义一个匿名函数。args 是参数列表，body 是函数体。".to_string(),
            example: Some("(fn [x y] (+ x y))".to_string()),
        },
    );

    docs.insert(
        "if".to_string(),
        FunctionDoc {
            name: "if".to_string(),
            signature: "(if condition then else)".to_string(),
            description: "条件表达式。如果 condition 为真（非 #f、非 0、非空列表）则求值 then，否则求值 else。".to_string(),
            example: Some("(if (> x 0) (print \"positive\") (print \"non-positive\"))".to_string()),
        },
    );

    docs.insert(
        "while".to_string(),
        FunctionDoc {
            name: "while".to_string(),
            signature: "(while condition body...)".to_string(),
            description: "循环求值 body 直到 condition 为假。".to_string(),
            example: Some(
                "(while (step 1)\n  (when (= INDEX 100)\n    (print \"Found!\")))".to_string(),
            ),
        },
    );

    docs.insert(
        "print".to_string(),
        FunctionDoc {
            name: "print".to_string(),
            signature: "(print args...)".to_string(),
            description: "打印所有参数到标准输出。".to_string(),
            example: Some("(print \"Index: \" INDEX)".to_string()),
        },
    );

    docs.insert(
        "get".to_string(),
        FunctionDoc {
            name: "get".to_string(),
            signature: "(get signal)".to_string(),
            description: "获取当前时间索引下信号的值。".to_string(),
            example: Some("(get tb.clk)".to_string()),
        },
    );

    docs.insert(
        "signal?".to_string(),
        FunctionDoc {
            name: "signal?".to_string(),
            signature: "(signal? name)".to_string(),
            description: "检查名称是否为已加载波形中的信号。".to_string(),
            example: Some("(signal? \"tb.clk\")".to_string()),
        },
    );

    docs.insert(
        "find".to_string(),
        FunctionDoc {
            name: "find".to_string(),
            signature: "(find condition)".to_string(),
            description: "返回所有满足 condition 的时间索引列表。".to_string(),
            example: Some("(find (= tb.overflow 1))".to_string()),
        },
    );

    docs
});

pub fn get_function_docs() -> &'static HashMap<String, FunctionDoc> {
    &FUNCTION_DOCS
}

pub fn get_doc(name: &str) -> Option<FunctionDoc> {
    FUNCTION_DOCS.get(name).cloned()
}
