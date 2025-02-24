use core::{borrow::Borrow, cell::RefCell, fmt::{Display, Formatter}, ops::{Add, Sub}};

use alloc::{format, rc::Rc, string::{String, ToString}, vec::Vec};
use super::ast::{Node, Program};

type VariableMap = Vec<(String, Option<RuntimeValue>)>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    id: String,
    params: Vec<Option<Rc<Node>>>,
    body: Option<Rc<Node>>,
}

impl Function {
    fn new (id: String, params: Vec<Option<Rc<Node>>>, body: Option<Rc<Node>>) -> Self {
        Self {
            id,
            params,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    variables: VariableMap,
    outer: Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn new(outer: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            variables: VariableMap::new(),
            outer,
        }
    }

    pub fn get_variable(&self, name: String) -> Option<RuntimeValue> {
        for variable in &self.variables {
            if variable.0 == name {
                return variable.1.clone();
            }
        }
        if let Some(env) = &self.outer {
            env.borrow_mut().get_variable(name)
        } else {
            None
        }
    }

    fn add_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        self.variables.push((name, value));
    }

    fn update_variable(&mut self, name: String, value: Option<RuntimeValue>) {
        for i in 0..self.variables.len() {
            if self.variables[i].0 == name {
                self.variables.remove(i);
                self.variables.push((name,value));
                return;
            }
        }
    }
    
}


#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Number(u64),
    StringLiteral(String),
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        let s = match self {
            RuntimeValue::Number(value) => format!("{}", value),
            RuntimeValue::StringLiteral(value) => value.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub struct JsRuntime {
    functions: Vec<Function>,
    env: Rc<RefCell<Environment>>
}

impl JsRuntime {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new(None)))
        }
    }

    pub fn execute(&mut self, program: &Program) {
        for node in program.body() {
            self.eval(&Some(node.clone()), self.env.clone());
        }
    }

    fn eval(
        &mut self, 
        node: &Option<Rc<Node>>,
        env: Rc<RefCell<Environment>>
    ) -> Option<RuntimeValue> {
        let node = match node {
            Some(n) => n,
            None => return None
        };

        match node.borrow() {
            Node::ExpressionStatement(expr) => return self.eval(&expr, env.clone()),
            Node::AdditiveExpression { operator, left, right } => {
                let left_value = match self.eval(&left, env.clone()) {
                    Some(value) => value,
                    None => return None
                };
                let right_value = match self.eval(&right, env.clone()) {
                    Some(value) => value,
                    None => return None
                };

                if operator == &'+' {
                    Some(left_value + right_value)
                } else if operator == &'-' {
                    Some(left_value - right_value)
                } else {
                    None
                }
            }
            Node::AssignmentExpression { 
                operator, 
                left, 
                right } => {
                    if operator != &'=' {
                        return None
                    }
                    if let Some(node) = left {
                        if let Node::Identifier(id) = node.borrow() {
                            let new_value = self.eval(&right, env.clone());
                            env.borrow_mut().update_variable(id.to_string(), new_value);
                            return None;
                        }
                    }
                    None
            }
            Node::MemberExpression { object: _, property: _ } => {
                None
            }
            Node::NumericLiteral(value) => Some(RuntimeValue::Number(*value)),
            Node::VariableDeclaration { declarations } => {
                for declaration in declarations {
                    self.eval(&declaration,  env.clone());
                }
                None
            }
            Node::VariableDeclarator { id, init } => {
                if let Some(node) = id {
                    if let Node::Identifier(id) = node.borrow() {
                        let init = self.eval(&init, env.clone());
                        env.borrow_mut().add_variable(id.to_string(), init);
                    }
                }
                None
            }
            Node::Identifier(name) => {
                match env.borrow_mut().get_variable(name.to_string()) {
                    Some(v) => Some(v),
                    None => Some(RuntimeValue::StringLiteral(name.to_string()))
                }
            }
            Node::StringLiteral(value) => {
                Some(RuntimeValue::StringLiteral(value.to_string()))
            }
            Node::BlockStatement { body } => {
                let mut result: Option<RuntimeValue> = None;
                for stmt in body {
                    result = self.eval(&stmt, env.clone());
                }
                result
            }
            Node::ReturnStatement { argument } => {
                return self.eval(&argument, env.clone());
            }
            Node::FunctionDeclaration { id, params, body } => {
                if let Some(RuntimeValue::StringLiteral(id)) = self.eval(&id, env.clone()) {
                    let cloned_body = match body {
                        Some(b) => Some(b.clone()),
                        None => None
                    };
                    self.functions.push(Function::new(id, params.to_vec(), cloned_body))
                }
                None
            }
            Node::CallExpression { callee, arguments } => {
                let new_env = Rc::new(RefCell::new(Environment::new(Some(env))));

                let calee_value = match self.eval(callee, new_env.clone()) {
                    Some(value) => value,
                    None => return None
                };

                let function = {
                    let mut f: Option<Function> = None;

                    for func in &self.functions {
                        if calee_value == RuntimeValue::StringLiteral(func.id.to_string()) {
                            f = Some(func.clone());
                        }
                    }

                    match f {
                        Some(f) => f,
                        None => panic!("function {:?} doen't exist", callee),
                    }
                };

                assert!(arguments.len() == function.params.len());
                for(i, item) in arguments.iter().enumerate() {
                    if let Some(RuntimeValue::StringLiteral(name)) = self.eval(&function.params[i], new_env.clone()) {
                        new_env.borrow_mut().add_variable(name, self.eval(item, new_env.clone()));
                    }
                }
                self.eval(&function.body.clone(), new_env.clone())
            }
        }   
    }

    
}


impl Add<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num + right_num);
        }
        RuntimeValue::StringLiteral(self.to_string() + &rhs.to_string())
    }
}

impl Sub<RuntimeValue> for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: RuntimeValue) -> RuntimeValue {
        if let (RuntimeValue::Number(left_num), RuntimeValue::Number(right_num)) = (&self, &rhs) {
            return RuntimeValue::Number(left_num - right_num);
        }
        return RuntimeValue::Number(u64::MIN);
    }
}


#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::renderer::js::{ast::JsParser, token::JsLexer};

    use super::{JsRuntime, RuntimeValue};

    

    #[test]
    fn test_num() {
        let input = "42".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(42))];

        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
        
    }


    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(3))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_sub_nums() {
        let input = "2 - 1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [Some(RuntimeValue::Number(1))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_assign_variable() {
        let input = "var foo=42;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None];
        let mut i = 0;
        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_reassign_variable() {
        let input = "var foo=42; foo=1; foo".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, None, Some(RuntimeValue::Number(1))];
        let mut i = 0;
        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_add_function_add_num() {
        let input = "function foo() { return 42; } foo()+1".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, Some(RuntimeValue::Number(43))];
        let mut i = 0;

        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }


    #[test]
    fn test_define_function_with_args() {
        let input = "function foo(a, b) { return a + b; } foo(1,2) + 3;".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, Some(RuntimeValue::Number(6))];
        let mut i = 0;
        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }

    #[test]
    fn test_local_variable() {
        let input = "var a=42; function foo() { var a=1; return a;} foo()+a".to_string();
        let lexer = JsLexer::new(input);
        let mut parser = JsParser::new(lexer);
        let ast = parser.parse_ast();
        let mut runtime = JsRuntime::new();
        let expected = [None, None, Some(RuntimeValue::Number(43))];
        let mut i = 0;
        for node in ast.body() {
            let result = runtime.eval(&Some(node.clone()), runtime.env.clone());
            assert_eq!(expected[i], result);
            i += 1;
        }
    }



}