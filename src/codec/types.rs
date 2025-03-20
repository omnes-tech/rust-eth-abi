use crate::codec::traits::BoxTrait;

#[derive(Debug)]
pub enum Value {
    Single(Box<dyn BoxTrait>, String),
    Collection(Vec<Value>),
}

impl Value {
    pub fn new(values: Vec<Value>) -> Self {
        Value::Collection(values)
    }

    pub fn get_i(&self, index: usize) -> &Self {
        match self {
            Value::Single(_, _) => &self,
            Value::Collection(values) => &values[index],
        }
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, Value::Collection(_))
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Single(value, type_str) => Value::Single(value.clone_box(), type_str.clone()),
            Value::Collection(values) => Value::Collection(values.clone()),
        }
    }
}

pub struct ValueBuilder {
    values: Vec<Value>,
}

impl ValueBuilder {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn add<T: BoxTrait + 'static>(&mut self, value: T) -> &mut Self {
        let eth_type = value.eth_type();
        self.values.push(create_value(value, eth_type.as_str()));
        self
    }

    pub fn add_array<T: BoxTrait + 'static>(&mut self, values: Vec<T>) -> &mut Self {
        let inner_values: Vec<Value> = if let Some(first) = values.first() {
            if first.as_any().is::<Vec<Box<dyn BoxTrait>>>() {
                // Handle array of tuples
                values
                    .into_iter()
                    .map(|v| {
                        if let Some(tuple) = v.as_any().downcast_ref::<Vec<Box<dyn BoxTrait>>>() {
                            let mut tuple_values = Vec::new();
                            for value in tuple {
                                if let Some(string_vec) =
                                    value.encode_codec().as_any().downcast_ref::<Vec<String>>()
                                {
                                    let string_values: Vec<Value> = string_vec
                                        .iter()
                                        .map(|s| {
                                            Value::Single(
                                                Box::new(s.clone()) as Box<dyn BoxTrait>,
                                                "string".to_string(),
                                            )
                                        })
                                        .collect();
                                    tuple_values.push(Value::Collection(string_values));
                                } else {
                                    let type_str = value.eth_type();
                                    tuple_values.push(Value::Single(value.clone_box(), type_str));
                                }
                            }
                            Value::Collection(tuple_values)
                        } else {
                            panic!("Expected tuple value")
                        }
                    })
                    .collect()
            } else {
                let type_str = values.first().unwrap().eth_type();
                // Handle regular arrays
                values
                    .into_iter()
                    .map(|v| Value::Single(Box::new(v) as Box<dyn BoxTrait>, type_str.clone()))
                    .collect()
            }
        } else {
            Vec::new()
        };

        self.values.push(Value::Collection(inner_values));
        self
    }

    pub fn add_tuple(&mut self, values: Vec<Box<dyn BoxTrait>>) -> &mut Self {
        let mut inner_values = Vec::new();
        for value in values {
            if let Some(string_vec) = value.as_any().downcast_ref::<Vec<String>>() {
                let string_values: Vec<Value> = string_vec
                    .iter()
                    .map(|s| {
                        Value::Single(
                            Box::new(s.clone()) as Box<dyn BoxTrait>,
                            "string".to_string(),
                        )
                    })
                    .collect();
                inner_values.push(Value::Collection(string_values));
            } else {
                let type_str = value.eth_type();
                inner_values.push(Value::Single(value, type_str));
            }
        }

        self.values.push(Value::Collection(inner_values));
        self
    }

    pub fn build(&self) -> Vec<Value> {
        self.values.clone()
    }
}

pub fn create_value<T: BoxTrait + 'static>(value: T, type_str: &str) -> Value {
    Value::Single(Box::new(value), type_str.to_string())
}

pub fn create_array_value<T: BoxTrait + 'static>(values: Vec<T>, element_type: &str) -> Value {
    Value::Collection(
        values
            .into_iter()
            .map(|v| create_value(v, element_type))
            .collect(),
    )
}
