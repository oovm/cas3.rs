extern crate peg;
use ordered_float;
use rustyline::{DefaultEditor, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::ops::{Deref, DerefMut};
use std::{fmt, path::Path};

peg::parser! {
    grammar expr_parser() for str {
        rule integer() -> Expr
            = n:$("-"? ['0'..='9']+ ) {? n.parse().map(Expr::Int).or(Err("integer")) }

        rule real() -> Expr
            = n:$("-"? ['0'..='9']* "." ['0'..='9']+ ) {? n.parse().map(Expr::Real).or(Err("real")) }

        rule symbol() -> Expr
            = s:$(['a'..='z' | 'A'..='Z' | '?' | '$'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' ]* ) { Expr::Sym(s.into()) }

        rule string() -> Expr
            = "\"" s:$((!['"'][_])* ) "\"" { Expr::Str(s.into()) }

        rule atom() -> Expr
            = real() / integer() / symbol() / string()

        rule list() -> Expr
            = "(" l:Expr() ** " " ")" { Expr::List(l) }

        pub rule Expr() -> Expr
            = atom() / list()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Expr {
    Int(i64),
    Real(ordered_float::NotNan<f64>),
    Sym(String),
    Str(String),
    List(Vec<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Real(r) => write!(f, "{}", r),
            Expr::Sym(s) => write!(f, "{}", s),
            Expr::Str(s) => write!(f, "\"{}\"", s),
            Expr::List(lst) => {
                let str_list: Vec<String> = lst.iter().map(|x| x.to_string()).collect();
                write!(f, "({})", str_list.join(" "))
            }
        }
    }
}

impl Deref for Expr {
    type Target = Vec<Expr>;

    fn deref(&self) -> &Self::Target {
        match self {
            Expr::List(vec) => vec,
            _ => panic!("Can only deref Expr::List"),
        }
    }
}

impl DerefMut for Expr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Expr::List(vec) => vec,
            _ => panic!("Can only deref Expr::List"),
        }
    }
}

pub fn sym(s: &str) -> Expr {
    Expr::Sym(s.to_string())
}

fn head(expr: &Expr) -> Expr {
    match expr {
        Expr::Int(_) => Expr::Sym("Int".to_string()),
        Expr::Real(_) => Expr::Sym("Real".to_string()),
        Expr::Sym(_) => Expr::Sym("Sym".to_string()),
        Expr::Str(_) => Expr::Sym("Str".to_string()),
        Expr::List(lst) => {
            if let Some(first) = lst.first() {
                first.clone()
            } else {
                println!("[ERROR]: empty list isnt allowed");
                Expr::Sym("GET_FUCKED".to_string())
            }
        }
    }
}

pub fn is_atom(expr: &Expr) -> bool {
    match expr {
        Expr::List(_) => false,
        _ => true,
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Context2 {
    vars: HashMap<Expr, TableEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TableEntry {
    own: Option<Expr>,
    down: Expr,
    sub: Expr,
}

impl TableEntry {
    pub fn new() -> Self {
        Self {
            own: None,
            down: Expr::List(vec![sym("list")]),
            sub: Expr::List(vec![sym("list")]),
        }
    }
}

pub fn get_ownvalue(ctx: &Context2, sym: Expr) -> Option<Expr> {
    let te = ctx.vars.get(&sym);
    if let Some(te) = te {
        let rule = te.own.clone();
        if let Some(rule) = rule {
            Some(rule[2].clone())
        } else {
            None
        }
        // for now, since I'm only allowing a single ownvalue maybe im not going to do the whole handling of HoldPattern[lhs] :> rhs
        // where i actually take sym and do sym /. OwnValues[sym]
        // apply_rule()
    } else {
        None
    }
}

pub fn get_downvalues(ctx: &Context2, sym: Expr) -> Option<Expr> {
    let te = ctx.vars.get(&sym);
    if let Some(te) = te {
        let rule = te.down.clone();
        if let Expr::List(rule) = rule {
            Some(Expr::List(rule.to_vec()))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_subvalues(ctx: &Context2, sym: Expr) -> Option<Expr> {
    let te = ctx.vars.get(&sym);
    if let Some(te) = te {
        let rule = te.sub.clone();
        if let Expr::List(rule) = rule {
            Some(Expr::List(rule.to_vec()))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn evaluate(stack: &mut Expr, ctx: &mut Context2, expr: &Expr) -> Expr {
    let mut ex = expr.clone();
    let mut last_ex = None;

    loop {
        if Some(&ex) == last_ex.as_ref() {
            // If the expression hasn't changed, break the loop.
            break;
        }

        last_ex = Some(ex.clone());

        match &ex {
            Expr::Int(_) | Expr::Real(_) | Expr::Str(_) => {
                break;
            }
            Expr::Sym(ref s) => {
                if let Some(rule) = get_ownvalue(ctx, sym(s)) {
                    ex = rule;
                } else {
                    break;
                }
            }
            Expr::List(ref ls) => {
                let h = ls.first().unwrap();
                let nh = evaluate(stack, ctx, h);
                let mut evaluated_args = vec![];

                for p in &ls[1..] {
                    evaluated_args.push(evaluate(stack, ctx, p));
                }

                // ex = match nh {
                //     // we dont need to panic here "abc"[foo] doesn't
                //     Expr::Int(_) | Expr::Real(_) | Expr::Str(_) => panic!("head must be a symbol"),
                //     Expr::Sym(ref s) => apply_downvalues(stack, ctx, nh, &evaluated_args),
                //     Expr::List(ref head_args) => apply_subvalues(stack, ctx, nh, &evaluated_args),
                // };
                if nh == sym("matchq") {
                    assert!(evaluated_args.len() == 2);
                } else {
                    ex = Expr::List(
                        std::iter::once(nh.clone())
                            .chain(evaluated_args.clone())
                            .collect(),
                    );
                }
            }
        }
    }

    ex
}

// pub fn startup(ctx: &mut Context, startup_path: &Path) -> Result<()> {
//     let file = File::open(startup_path)?;
//     let reader = BufReader::new(file);

//     for line in reader.lines() {
//         match line {
//             Ok(content) => {
//                 if let Ok(ex) = &expr_parser::Expr(&content) {
//                     let mut stack = Expr::List(vec![]);
//                     evaluate(&mut stack, ctx, ex);
//                 }
//             }
//             Err(error) => {
//                 eprintln!("Error reading a line: {:?}", error);
//             }
//         }
//     }

//     Ok(())
// }

pub fn run() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut ctx = Context2 {
        vars: HashMap::new(),
    };

    // startup(&mut ctx, Path::new("startup.sexp")).unwrap();

    let mut i = 0;

    loop {
        let prompt = format!("(In {})> ", i);
        let line = rl.readline(&prompt)?; // read
        rl.add_history_entry(line.as_str()).unwrap(); // history
        let ex = expr_parser::Expr(&line);
        match ex {
            Ok(expr) => {
                let mut stack = Expr::List(vec![]);
                let res = evaluate(&mut stack, &mut ctx, &expr);
                // println!("head: {}", head(&expr));

                // ins and outs (works but makes ctx printing too verbose, and its just not that useful rn )
                // let in_i = expr_parser::Expr(format!("(setd (In {i}) {})", expr).as_str()).unwrap();
                // evaluate(&mut ctx, &in_i);
                // let out_i =
                //     expr_parser::Expr(format!("(set (Out {i}) {})", expr).as_str()).unwrap();
                // evaluate(&mut ctx, &out_i);

                println!("(Out {i}): {}", res);
                i += 1;
            }
            Err(err) => println!("Failed to parse: {}", err),
        }
    } // loop
}

fn is_match(expr: &Expr, pattern_expr: &Expr, bindings: &mut HashMap<String, Expr>) -> bool {
    match (expr, pattern_expr) {
        (Expr::List(e_list), Expr::List(p_list)) => {
            if p_list.len() == 0 || e_list.len() != p_list.len() {
                return false;
            }
            for (e, p) in e_list.iter().zip(p_list.iter()) {
                if !is_match(e, p, bindings) {
                    return false;
                }
            }
            true
        }
        (_, Expr::List(p_list)) => {
            if let Expr::Sym(ref p_head) = p_list[0] {
                if p_head == "pattern" {
                    let name = p_list[1].clone().to_string();
                    let pattern = &p_list[2];
                    if let Some(existing_binding) = bindings.get(&name) {
                        return expr == existing_binding;
                    }
                    if is_match(expr, pattern, bindings) {
                        bindings.insert(name, expr.clone());
                        return true;
                    }
                } else if p_head == "blank" {
                    if p_list.len() == 2 {
                        println!("p_list: {:?}", p_list);
                        let required_head = &p_list[1];
                        if head(expr) == *required_head {
                            return true;
                        }
                    } else if p_list.len() == 1 {
                        return true;
                    }
                }
            }
            false
        }
        (Expr::Sym(e), Expr::Sym(p)) => e == p,
        (Expr::Int(e), Expr::Int(p)) => e == p,
        (Expr::Real(e), Expr::Real(p)) => e == p,
        (Expr::Str(e), Expr::Str(p)) => e == p,
        _ => false,
    }
}

fn main() -> Result<()> {
    // run()?;

    let mut bindings = HashMap::new();
    let expr = expr_parser::Expr("((k a) b)").unwrap();
    let expr = expr_parser::Expr("(plus 1 2)").unwrap();

    let pattern =
        expr_parser::Expr("((k (pattern x (blank Sym))) (pattern y (blank Sym)))").unwrap();

    let pattern = expr_parser::Expr("((k (pattern x (blank))) (pattern y (blank)))").unwrap();

    let pattern = expr_parser::Expr("(plus (blank) (blank))").unwrap();

    let does_match = is_match(&expr, &pattern, &mut bindings);
    println!("does_match: {} with bindings {:?}", does_match, bindings);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use expr_parser::Expr;
    use std::collections::HashMap;

    fn setup() -> HashMap<String, Expr> {
        HashMap::new()
    }

    #[test]
    fn test_blank_match() {
        let mut bindings = setup();
        let expr = expr_parser::Expr("1").unwrap();
        let pattern = expr_parser::Expr("(blank)").unwrap();
        assert!(is_match(&expr, &pattern, &mut bindings));
    }

    #[test]
    fn test_blank_int_match() {
        let mut bindings = setup();
        let expr = expr_parser::Expr("1").unwrap();
        let pattern = expr_parser::Expr("(blank Int)").unwrap();
        assert!(is_match(&expr, &pattern, &mut bindings));
    }

    #[test]
    fn test_pattern_blank_match() {
        let mut bindings = setup();
        let expr = expr_parser::Expr("1").unwrap();
        let pattern = expr_parser::Expr("(pattern x (blank))").unwrap();
        assert!(is_match(&expr, &pattern, &mut bindings));
    }

    #[test]
    fn test_pattern_blank_int_match() {
        let mut bindings = setup();
        let expr = expr_parser::Expr("1").unwrap();
        let pattern = expr_parser::Expr("(pattern x (blank Int))").unwrap();
        assert!(is_match(&expr, &pattern, &mut bindings));
    }

    #[test]
    fn test_pattern_blank_sym_no_match() {
        let mut bindings = setup();
        let expr = expr_parser::Expr("1").unwrap();
        let pattern = expr_parser::Expr("(pattern x (blank Sym))").unwrap();
        assert!(!is_match(&expr, &pattern, &mut bindings));
    }
}

/*
exprs/programs to make work
1.

(set x 1)
x
(set y 2)
(+ x y) => (+ 1 2). I don't think i want/need to implement arithmetic yet
2.
k[x_][y_] := x
(SetDelayed (k (pattern x (blank)) (pattern y (blank))) x)

f[x_] := {x, x^2, x^3}
f[y] # gives {x, x^2, x^3}
(SetDelayed (f (pattern x (blank)) (list x (pow x 2) (pow x 3)))

3.
(matchq x x) # true
(matchq x y) # false
(matchq x (pattern (blank))) # true
(matchq (list a) (pattern (blank))) # true

4. (most important right now)
SetDelayed[fib[Pattern[n, Blank[]]], Plus[fib[Plus[n, -1]], fib[Plus[n, -2]]]]]

(set (fib 1) (fib 0))
(set (fib 0) 1)
(set_delayed (fib (pattern n (blank))) (plus (fib (minus n 1)) (fib (minus n 2))))
(set_delayed (fib (pattern n (blank Int))) (plus (fib (minus n 1)) (fib (minus n 2))))
(fib 5)

okay so we make a new hashmap called DownValues that is a HashMap of symbol to list of exprs
this list of expr is all the downvalues.
so
f[x_][y_] := x
f[x_] := 1

f[x][y] # 1

k[x_][y_] := x
k[x][y] # x

so it does the recursive thing, doesn't find any pattern matching (k x), which it looks to find first, shown by the f example
then goes out to see if there is a more nested pattern that matches, which is the k example


one thing that mathematica does is it actually stores (fib 2, 3,... ) in the evaluation of fib(5)


notes:
currently this crashes the interpreter because it goes into an infinite loop (no fixed point)
(set (f) (f f)
(set (f f) (f))

f[x_] := x
f[1]

(setd (f (pattern x (blank))) (f x))
(f 1) so basically wh

(f (list 1))


------
TODO actually make testing

(set (a b) c)
(a b) == c
(set b 1)
(a b) == (a 1)

------
need to make
(set x (plus x 1))
crash the program
and
(setd x (plus x 1)) not
but (setd x (plus x 1)), (x) should crash the program



f[x_]:=g[y_]:=y
f[1] === Null # True
but note that if you try to call g before f, then g is undefined
so
ff[x_]:=gg[y_]:=y
gg[1] # gives gg[1]
but then
ff[1]
gg[1] # now gives 1

also note that
x=1
x=2
works because Set is HoldFirst


https://mathematica.stackexchange.com/questions/176732/can-a-symbol-have-more-than-one-ownvalue
i will keep TableEntry.own as a list expr, but since I am not going to do conditional evaluation, it will only have one element
if set manually by the user, through OwnValues[x] = ..., i panic if more than one

one interesting thing is how to set Set attributes to HoldFirst and Setd before calling Set and Setd.
maybe have to manually put in those DownValues of Attributes manually in rust and not in startup
can also just hardcode it in evaluate to never evaluate the first argument of Set and the rest


apply just replaces list with arg[1]
apply[f, {a, b, c}] # f[a, b, c]


*/
