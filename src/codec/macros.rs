#[macro_export]
macro_rules! build_values {
    // Base case for arrays (vec![...]) - creates a Collection
    (vec![$($inner:expr),* $(,)?]) => {{
        let inner_values = vec![
            $(Value::Single($inner, $inner.eth_type())),*
        ];
        Value::Collection(inner_values)
    }};

    // Base case for single values (non-vectors)
    ($value:expr) => {{
        Value::Single($value, $value.eth_type())
    }};

    // Case for multiple values at the top level - wraps them in a Vec
    ($($value:expr),* $(,)?) => {
        vec![
            $(build_values!($value)),*
        ]
    };
}
