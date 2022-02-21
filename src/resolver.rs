use std::collections::BTreeMap;

use crate::{
    ast::{
        CodeExpression, Expression, ResolvedCodeExpression, ResolvedStatement, ResolvedVariable,
        Statement,
    },
    error::{ResolveError, ResolveErrorKind},
};

#[derive(PartialEq)]
enum ResolverState {
    Declared,
    Defined,
}

enum FunctionType {
    Function,
}

type ResolveResult<T> = Result<T, Vec<ResolveError>>;

pub struct Resolver {
    scopes: Vec<BTreeMap<String, ResolverState>>,
    current_function: Option<FunctionType>,
}

impl Resolver {
    #[allow(dead_code)]
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            current_function: None,
        }
    }

    #[allow(dead_code)]
    pub fn resolve(&mut self, ast: Vec<Statement>) -> ResolveResult<Vec<ResolvedStatement>> {
        self.resolve_block_statement(false, ast)
    }

    fn resolve_statement(&mut self, statement: Statement) -> ResolveResult<ResolvedStatement> {
        Ok(match statement {
            Statement::Expression(e) => ResolvedStatement::Expression(self.resolve_expr(e)?),
            Statement::Function(n, p, b) => {
                self.resolve_function_statement(n, p, b, FunctionType::Function)?
            }
            Statement::Print(e) => ResolvedStatement::Print(self.resolve_expr(e)?),
            Statement::Return(e) => {
                if self.current_function.is_none() {
                    return Err(vec![ResolveError {
                        // TODO
                        location: (0, 0),
                        value: ResolveErrorKind::TopLevelReturn,
                    }]);
                }
                ResolvedStatement::Return(e.map(|e| self.resolve_expr(e)).transpose()?)
            }
            Statement::Var(v, e) => self.resolve_var_statement(v, e)?,
            Statement::While(c, b) => self.resolve_while_statement(c, *b)?,
            Statement::Block(s) => ResolvedStatement::Block(self.resolve_block_statement(true, s)?),
            Statement::If(c, t, e) => self.resolve_if_statement(c, *t, e.map(|e| *e))?,
        })
    }

    fn resolve_function_statement(
        &mut self,
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        function_type: FunctionType,
    ) -> ResolveResult<ResolvedStatement> {
        self.declare(name.clone())?;
        self.define(name.clone());

        let mut previous_type = Some(function_type);
        std::mem::swap(&mut previous_type, &mut self.current_function);
        self.begin_scope();
        let res = (|| {
            for param in params.clone() {
                self.declare(param.clone())?;
                self.define(param);
            }
            self.resolve_block_statement(true, body)
        })();
        self.end_scope();
        std::mem::swap(&mut previous_type, &mut self.current_function);

        Ok(ResolvedStatement::Function(name, params, res?))
    }

    fn resolve_block_statement(
        &mut self,
        make_scope: bool,
        body: Vec<Statement>,
    ) -> ResolveResult<Vec<ResolvedStatement>> {
        if make_scope {
            self.begin_scope();
        }
        let mut statements = Vec::with_capacity(body.len());
        let mut errors = Vec::new();
        for statement in body {
            match self.resolve_statement(statement) {
                Ok(s) => statements.push(s),
                Err(e) => errors.extend(e),
            }
        }

        let res = if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        };
        if make_scope {
            self.end_scope();
        }
        res
    }

    fn resolve_if_statement(
        &mut self,
        condition: CodeExpression,
        then_branch: Statement,
        else_branch: Option<Statement>,
    ) -> ResolveResult<ResolvedStatement> {
        let condition = self.resolve_expr(condition)?;
        let then_b = self.resolve_statement(then_branch)?;
        let else_b = else_branch.map(|b| self.resolve_statement(b)).transpose()?;
        Ok(ResolvedStatement::If(
            condition,
            Box::new(then_b),
            else_b.map(Box::new),
        ))
    }

    fn resolve_var_statement(
        &mut self,
        name: String,
        expr: Option<CodeExpression>,
    ) -> ResolveResult<ResolvedStatement> {
        self.declare(name.clone())?;
        let expr = expr.map(|expr| self.resolve_expr(expr)).transpose()?;
        self.define(name.clone());
        Ok(ResolvedStatement::Var(name, expr))
    }

    fn resolve_while_statement(
        &mut self,
        condition: CodeExpression,
        body: Statement,
    ) -> ResolveResult<ResolvedStatement> {
        let condition = self.resolve_expr(condition)?;
        let body = self.resolve_statement(body)?;
        Ok(ResolvedStatement::While(condition, Box::new(body)))
    }

    pub fn resolve_expr(&mut self, expr: CodeExpression) -> ResolveResult<ResolvedCodeExpression> {
        let loc = expr.location;
        Ok(match expr.value {
            Expression::Binary(l, o, r) => ResolvedCodeExpression {
                location: loc,
                value: Expression::Binary(
                    Box::new(self.resolve_expr(*l)?),
                    o,
                    Box::new(self.resolve_expr(*r)?),
                ),
            },
            Expression::Call(f, p) => self.resolve_call_expr(loc, *f, p)?,
            Expression::Grouping(e) => ResolvedCodeExpression {
                location: loc,
                value: Expression::Grouping(Box::new(self.resolve_expr(*e)?)),
            },
            Expression::Literal(l) => ResolvedCodeExpression {
                location: loc,
                value: Expression::Literal(l),
            },
            Expression::Unary(o, r) => ResolvedCodeExpression {
                location: loc,
                value: Expression::Unary(o, Box::new(self.resolve_expr(*r)?)),
            },
            Expression::Variable(n) => self.resolve_variable_expr(loc, n)?,
            Expression::Assign(n, e) => self.resolve_assign_expr(loc, n, *e)?,
        })
    }

    fn resolve_call_expr(
        &mut self,
        location: (usize, usize),
        callee: CodeExpression,
        params: Vec<CodeExpression>,
    ) -> ResolveResult<ResolvedCodeExpression> {
        let callee = self.resolve_expr(callee)?;
        let params: Result<_, _> = params.into_iter().map(|p| self.resolve_expr(p)).collect();
        Ok(ResolvedCodeExpression {
            location,
            value: Expression::Call(Box::new(callee), params?),
        })
    }

    fn resolve_variable_expr(
        &mut self,
        location: (usize, usize),
        name: String,
    ) -> ResolveResult<ResolvedCodeExpression> {
        if self.scopes.last().and_then(|s| s.get(&name)) == Some(&ResolverState::Declared) {
            Err(vec![ResolveError {
                location,
                value: ResolveErrorKind::VariableOwnInitializer,
            }])
        } else {
            Ok(ResolvedCodeExpression {
                location,
                value: Expression::Variable(self.resolve_local(name)),
            })
        }
    }

    fn resolve_local(&mut self, name: String) -> ResolvedVariable {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name) {
                return ResolvedVariable {
                    name,
                    hops: Some(self.scopes.len() - 1 - i),
                };
            }
        }
        ResolvedVariable { name, hops: None }
    }

    fn resolve_assign_expr(
        &mut self,
        location: (usize, usize),
        name: String,
        expr: CodeExpression,
    ) -> ResolveResult<ResolvedCodeExpression> {
        let expr = self.resolve_expr(expr)?;
        let var = self.resolve_local(name);
        Ok(ResolvedCodeExpression {
            location,
            value: Expression::Assign(var, Box::new(expr)),
        })
    }

    // util

    fn declare(&mut self, name: String) -> ResolveResult<()> {
        if let Some(current) = self.scopes.last_mut() {
            if current.contains_key(&name) {
                return Err(vec![ResolveError {
                    // TODO: or maybe dont todo
                    location: (0, 0),
                    value: ResolveErrorKind::VariableRedeclaration,
                }]);
            }
            current.insert(name, ResolverState::Declared);
        }
        Ok(())
    }

    fn define(&mut self, name: String) {
        if let Some(current) = self.scopes.last_mut() {
            current.insert(name, ResolverState::Defined);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
