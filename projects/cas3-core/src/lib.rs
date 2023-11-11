// extern crate cairo;
extern crate peg;

use std::{
    ops::{Add, Mul},
};
use std::{fmt, path::Path};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

// use rug::Integer;
// use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use ordered_float::{self, NotNan};
// use cairo::{Context, SvgSurface};

#[cfg(test)]
mod tests;
mod errors;

peg::parser! {
    pub grammar expr_parser() for str {
        rule comment()
            = "(*" (!"*)" [_])* "*)"

        rule whitespace() = ([' ' | '\t' | '\n' | '\r'] / comment())* // Allow whitespace or comments

        rule integer() -> Expr
            = n:$("-"? ['0'..='9']+ ) {? n.parse().map(Expr::Int).or(Err("integer")) }

        rule real() -> Expr
            = n:$("-"? ['0'..='9']* "." ['0'..='9']* ) {? n.parse().map(Expr::Real).or(Err("real")) }

        rule symbol() -> Expr
            = s:$(['a'..='z' | 'A'..='Z' | '?' | '$'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' ]* ) { Expr::Sym(s.into()) }

        rule string() -> Expr
            = "\"" s:$((!['"'][_])* ) "\"" { Expr::Str(s.into()) }

        rule atom() -> Expr
            = real() / integer() / symbol() / string()

        rule list() -> Expr
            = "(" l:Expr() ** whitespace() ")" { Expr::List(l) }

        pub rule Expr() -> Expr
            = whitespace() e:(atom() / list()) whitespace() { e }

        pub rule expressions() -> Vec<Expr>
            = whitespace() e:Expr() ** whitespace() { e }
    }
}

fn parse(s: &str) -> Expr {
    expr_parser::Expr(s).unwrap()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Expr {
    Int(num_bigint::BigInt),
    // Int(Integer),
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
            e => panic!("Can only deref Expr::List. ex:{}", e),
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

fn list(strs: Vec<&str>) -> Expr {
    Expr::List(strs.iter().map(|s| sym(s)).collect::<Vec<_>>())
}

fn liste(es: Vec<Expr>) -> Expr {
    Expr::List(es)
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

fn length(expr: &Expr) -> Expr {
    match expr {
        Expr::List(es) => Expr::Int((es.len() - 1).into()),
        _ => Expr::Int(0.into()),
    }
}

// (a b c) -> (b c)
// fn rest(expr: &Expr) -> Expr {}

pub fn is_atom(expr: &Expr) -> bool {
    match expr {
        Expr::List(_) => false,
        _ => true,
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Cas3VM {
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
            down: Expr::List(vec![sym("List")]),
            sub: Expr::List(vec![sym("List")]),
        }
    }
}

pub fn get_ownvalue(ctx: &Cas3VM, sym: Expr) -> Option<Expr> {
    // println!("ctx: {:?}. sym: {}", ctx, sym);
    let te = ctx.vars.get(&sym);
    if let Some(te) = te {
        let rule = te.own.clone();
        return rule;
        // for now, since I'm only allowing a single ownvalue maybe im not going to do the whole handling of HoldPattern[lhs] :> rhs
        // where i actually take sym and do sym /. OwnValues[sym]
        // apply_rule()
    } else {
        None
    }
}

/// i hate this function 
fn unpack_mat(ex: Expr) -> Option<Vec<Vec<(f64, f64, f64)>>> {
    let mut result: Vec<Vec<(f64, f64, f64)>> = vec![];

    if let Expr::List(outer_list) = ex {
        // println!("outerr elem: {:?}", outer_list[0]);
        for outer_elem in &outer_list[1..] {
            if let Expr::List(inner_list) = outer_elem {
                let mut inner_vec: Vec<(f64, f64, f64)> = vec![];
                // println!("inner elem: {:?}", inner_list[0]);
                for inner_elem in &inner_list[1..] {
                    if let Expr::List(tuple_list) = inner_elem {
                        // println!("tuple_list: {:?}", tuple_list);
                        if tuple_list.len() == 4 {
                            if let (Expr::Real(a), Expr::Real(b), Expr::Real(c)) =
                                (&tuple_list[1], &tuple_list[2], &tuple_list[3])
                            {
                                inner_vec.push((a.into_inner(), b.into_inner(), c.into_inner()));
                            } else {
                                return None; // one of the elements was not a Real
                            }
                        } else {
                            return None; // tuple_list did not contain exactly 3 elements
                        }
                    } else {
                        return None; // inner_elem was not a List
                    }
                }
                result.push(inner_vec);
            } else {
                return None; // outer_elem was not a List
            }
        }
        Some(result)
    } else {
        None // ex was not a List
    }
}

// fn create_svg_from_colors(colors: Vec<Vec<(f64, f64, f64)>>, filename: &str, scale_factor: i32) {
//     let rows = colors.len();
//     let cols = colors[0].len(); // Assuming all rows have the same length
//
//     let width = cols as i32 * scale_factor;
//     let height = rows as i32 * scale_factor;
//
//     // Create SvgSurface
//     let surface = SvgSurface::new(width as f64, height as f64, Some(filename))
//         .expect("Couldn't create SVG surface");
//
//     // Create Context for drawing
//     let cr = Context::new(&surface).expect("Couldn't create Cairo context");
//
//     // Scale the coordinate system
//     cr.scale(scale_factor as f64, scale_factor as f64);
//
//     // Loop through each cell to draw rectangles with specified colors
//     for (i, row) in colors.iter().enumerate() {
//         for (j, &(r, g, b)) in row.iter().enumerate() {
//             cr.rectangle(j as f64, i as f64, 1.0, 1.0);
//             cr.set_source_rgb(r, g, b);
//             cr.fill().expect("Failed to fill rectangle");
//         }
//     }
//
//     // Finish drawing
//     cr.show_page().expect("Failed to save SVG");
// }

// fn export()

// are we guaranteed that we have a list here?
// can evaluated_args be empty
// nh can be List too
pub fn internal_functions_apply(
    stack: &mut Expr,
    ctx: &mut Cas3VM,
    nh: Expr,
    evaluated_args: Vec<Expr>,
) -> Expr {
    let reconstructed_ex = Expr::List(
        std::iter::once(nh.clone())
            .chain(evaluated_args.clone().to_owned())
            .collect(),
    );

    if nh == sym("matchq") {
        if evaluated_args.len() != 2 {
            println!("matchq takes 2 arguments");
            return sym("$Failed");
        }
        return Expr::Sym(format!(
            "{}",
            my_match(
                evaluated_args[0].clone(),
                evaluated_args[1].clone(),
                &vec![],
                &mut HashMap::new(),
                &mut HashMap::new(),
            )
        ));
    } else if nh == sym("sameq") {
        // println!("in sameq: evaluated_args: {:?}", evaluated_args);
        let first_arg = &evaluated_args[0];
        let all_same = evaluated_args.iter().all(|arg| arg == first_arg);
        return Expr::Sym(format!("{}", all_same));
    } else if nh == sym("replace") {
        return replace(&evaluated_args[0], &evaluated_args[1]);
    } else if nh == sym("replace_all") {
        return replace_all(&evaluated_args[0], &evaluated_args[1]);
    } else if nh == sym("rr") || nh == sym("replace_repeated") {
        if evaluated_args.len() != 2 {
            println!("replace_repeated takes 2 arguments");
            return sym("$Failed");
        }
        return replace_repeated(&evaluated_args[0], &evaluated_args[1]);
    } else if nh == sym("head") {
        return head(&evaluated_args[0]);
    } else if nh == sym("parse") {
        match evaluated_args[0] {
            Expr::Str(ref s) => {
                let pex = expr_parser::Expr(s);
                match pex {
                    Ok(expr) => return expr,
                    Err(err) => {
                        println!("Failed to parse: {}", err);
                        return sym("$Failed");
                    }
                }
            }
            _ => {
                println!("parse takes a string");
                return sym("$Failed");
            }
        }
    } else if nh == sym("set") {
        // array part setting notes:
        // (set l (Table false 9))
        // simple single assignment case
        // (set (Part l 1) 3)

        // this is the more advanced case setting multiple at the same time
        // (set (Part l (list 1 2 3)) (Table true 3))

        // println!("evaluated_args: {:?}", evaluated_args);
        let lhs = &evaluated_args[0];
        let rhs = &evaluated_args[1];

        match lhs {
            // ownvalue
            Expr::Sym(ref s) => {
                // pretty sure this needs to be fixed to check if te already exists
                let mut te = TableEntry::new();
                te.own = Some(evaluated_args[1].clone());
                ctx.vars.insert(sym(s), te);
                return evaluated_args[1].clone();
            }
            // this is the down/subvalue case
            Expr::List(ls) => {
                let lhs_h = &ls[0];
                match lhs_h {
                    // lhs_h is the tag in this downvalue assignment
                    Expr::Sym(_) => {
                        // todo: implementing list inplace modification
                        // if lhs_h == &sym("Part") {
                        //     let part_lhs = &ls[1];
                        //     let part_rhs = &ls[2];
                        //     assert_eq!(length(rhs), length(part_rhs));
                        // }
                        //given
                        // (set (f (pattern x (blank))) x)
                        // we end up pushing
                        // (rule_delayed (holdpattern expr[1]) expr[2])
                        // (rule_delayed (holdpattern evaluated_args[0]) evaluated_args[1])
                        // onto the downvalues of h (which is expected to have head list)
                        let te: &mut TableEntry = ctx
                            .vars
                            .entry(lhs_h.clone())
                            .or_insert_with(TableEntry::new);
                        let dv_str = format!("(rule_delayed (hold_pattern {lhs}) {rhs})");
                        let dv = expr_parser::Expr(&dv_str).unwrap();

                        // NOTE! here we aren't inserting in the right order, where we look for more specific
                        // definitions and insert them first. so user has to do the right order themselves
                        // at the moment
                        te.down.push(dv);
                        return rhs.clone();
                    }
                    // subvalue
                    Expr::List(_) => {
                        todo!("subvalues")
                    }
                    _ => {
                        let h = head(lhs_h);
                        println!("Tag {h} in {lhs} is Protected");
                        return rhs.clone();
                    }
                }
            }
            _ => {
                println!("set takes a symbol or list, got {}", lhs);
                return sym("$Failed");
            }
        }
    } else if nh == sym("setd") {
        // println!("evaluated_args: {:?}", evaluated_args);
        let lhs = &evaluated_args[0];
        match lhs {
            Expr::Sym(ref s) => {
                // pretty sure this needs to be fixed to check if te already exists
                let mut te = TableEntry::new();
                te.own = Some(evaluated_args[1].clone());
                ctx.vars.insert(sym(s), te);
                return sym("Null");
            }
            // this is the down/subvalue case
            Expr::List(ls) => {
                let lhs_h = &ls[0];
                match lhs_h {
                    // lhs_h is the tag in this downvalue assignment
                    Expr::Sym(_) => {
                        let rhs = &evaluated_args[1];
                        // onto the downvalues of h (which is expected to have head list)
                        let te: &mut TableEntry = ctx
                            .vars
                            .entry(lhs_h.clone())
                            .or_insert_with(TableEntry::new);
                        let dv_str = format!("(rule_delayed (hold_pattern {lhs}) {rhs})");
                        let dv = expr_parser::Expr(&dv_str).unwrap();

                        // NOTE! here we aren't inserting in the right order, where we look for more specific
                        // definitions and insert them first. so user has to do the right order themselves
                        // at the moment
                        te.down.push(dv);
                        return sym("Null");
                    }
                    // subvalue
                    Expr::List(_) => {
                        todo!("subvalues")
                    }
                    _ => panic!("hi"),
                }
            }
            _ => {
                println!("set takes a symbol or list, got {}", lhs);
                return sym("$Failed");
            }
        }
    } else if nh == sym("own_values") {
        ctx.vars
            .get(&evaluated_args[0])
            .unwrap() // there is a bug here
            .own
            .clone()
            .unwrap()
        // todo!()
    } else if nh == sym("down_values") {
        ctx.vars.get(&evaluated_args[0]).unwrap().down.clone()
    } else if nh == sym("sub_values") {
        todo!()
    } else if nh == sym("clear") {
        match &evaluated_args[0] {
            Expr::Sym(_) => {
                if let Some(te) = ctx.vars.get_mut(&evaluated_args[0]) {
                    te.own = None;
                    te.down = Expr::List(vec![sym("List")]);
                    te.sub = Expr::List(vec![sym("List")]);
                }
                return sym("Null");
            }
            _ => {
                println!("set takes a symbol");
                return sym("$Failed");
            }
        }
    } else if nh == sym("Plus") {
        match (&evaluated_args[0], &evaluated_args[1]) {
            (Expr::Int(a), Expr::Int(b)) => Expr::Int(a.add(b).into()),
            // see issue about 3.0 printing as `3`
            // (Expr::Real(a), Expr::Real(b)) => Expr::Real(a + b),
            _ => {
                let reconstructed_ex = Expr::List(
                    std::iter::once(nh.clone())
                        .chain(evaluated_args.clone().to_owned())
                        .collect(),
                );
                return reconstructed_ex;
            }
        }
    } else if nh == sym("Times") {
        match (&evaluated_args[0], &evaluated_args[1]) {
            (Expr::Int(a), Expr::Int(b)) => Expr::Int(a.mul(b).into()),
            _ => {
                return reconstructed_ex;
            }
        }
    } else if nh == sym("Part") {
        match &evaluated_args[0] {
            Expr::List(ls) => match &evaluated_args[1] {
                Expr::Int(i) => {
                    let i = i.to_isize().unwrap();
                    if i < 0 || i >= ls.len() as isize {
                        println!("Part: index {} out of range", i);
                        return reconstructed_ex;
                    }
                    return ls[i as usize].clone();
                }
                Expr::List(indices) => {
                    let mut results = vec![sym("List")];
                    for index in indices[1..].iter() {
                        match index {
                            Expr::Int(i) => {
                                let i = i.to_isize().unwrap();
                                if i < 0 || i >= ls.len() as isize {
                                    println!("Part: index {} out of range", i);
                                    return reconstructed_ex;
                                }
                                results.push(ls[i as usize].clone());
                            }
                            _ => {
                                // println!("Part: each index in the list must be an integer");
                                return reconstructed_ex;
                            }
                        }
                    }
                    return Expr::List(results);
                }
                _ => {
                    // println!("Part: index must be an integer or a list of integers");
                    return reconstructed_ex;
                }
            },
            _ => return reconstructed_ex,
        }
    } else if nh == sym("Length") {
        return length(&evaluated_args[0]);
    } else if nh == sym("Get") {
        if let Expr::Str(p) = &evaluated_args[0] {
            let res = ctx.run_file(p);
            if let Ok(res) = res {
                return res;
            } else {
                return sym("$Failed");
            }
        } else {
            println!("Get takes an Expr::String");
            return sym("$Failed");
        }
    } else if nh == sym("Map") {
        // todo level spec
        // honestly i was hoping i could do this in cas3, not builtin but just to get things going

        let mut res = liste(vec![head(&evaluated_args[1])]);
        let f = &evaluated_args[0];
        let mapargs = &evaluated_args[1];

        for (_i, arg) in mapargs[1..].iter().enumerate() {
            let fi = Expr::List(vec![f.clone(), arg.clone()]);
            res.push(fi);
        }
        return res;
    } else if nh == sym("NestList") {
        let f = &evaluated_args[0];
        let x = &evaluated_args[1];
        let n = &evaluated_args[2];
        let mut res = list(vec!["List"]);
        res.push(x.clone());
        if let Expr::Int(count) = n {
            for _i in 0..count.to_i64().unwrap() {
                let fi = evaluate(
                    stack,
                    ctx,
                    &Expr::List(vec![f.clone(), res.last().unwrap().clone()]),
                );
                res.push(fi);
            }
            return res;
        } else {
            return reconstructed_ex;
        }
    } else if nh == sym("Table") {
        let table_body = &evaluated_args[0];

        // todo: test if this works implemented in cas3 code
        if evaluated_args.len() == 1 {
            return table_body.clone();
        }
        let spec = evaluate(stack, ctx, &evaluated_args[1].clone());

        if evaluated_args.len() == 2 {
            // if int, we copy n times
            if let Expr::Int(n) = &spec {
                return Expr::List(
                    std::iter::once(sym("List"))
                        .chain((0..n.to_i64().unwrap()).map(|_| evaluated_args[0].clone()))
                        .collect(),
                );
            } else if let Expr::List(ls) = &spec {
                // this is the place where we want to make helpers for
                // the "standard Wolfram Language iteration specification"

                let mut res = Expr::List(vec![sym("List")]);
                let var = &ls[1];

                // this specific case is {i, imax}
                if ls.len() == 3 {
                    if ls[0] != sym("List") {
                        dbg!("invalid range specification. need a list");
                        // return reconstructed_ex;
                    }
                    if let Expr::Int(imax) = &ls[2] {
                        for i in 1..=imax.to_i64().unwrap() {
                            let mut e_i = Expr::List(vec![sym("replace_all")]);
                            e_i.push(table_body.clone());
                            let local_rule =
                                Expr::List(vec![sym("rule"), var.clone(), Expr::Int(i.into())]); // (rule var iter)
                            e_i.push(local_rule);
                            res.push(e_i);
                        }
                        return res;
                    } else if let Expr::List(vals) = &ls[2] {
                        // this is the sequence case where you just
                        // Table[expr,{i,{i1,i2,…}}]

                        if head(&ls[2]) != sym("List") {
                            dbg!(
                                "invalid range specification. need a list of values, gave {}",
                                &ls[2]
                            );
                            // return reconstructed_ex;
                        } else {
                            for val in &vals[1..] {
                                let mut e_i = Expr::List(vec![sym("replace_all")]);
                                e_i.push(table_body.clone());
                                let local_rule =
                                    Expr::List(vec![sym("rule"), var.clone(), val.clone()]); // (rule var iter)
                                e_i.push(local_rule);
                                res.push(e_i);
                            }
                            return res;
                        }
                    } else {
                        dbg!("need an int or list for imax, reals not supported in iteration specification yet");
                        return sym("$Failed");
                    }
                } else if ls.len() == 4 {
                    // this is {i, imin, imax}
                    if let (Expr::Int(imin), Expr::Int(imax)) = (&ls[2], &ls[3]) {
                        for i in imin.to_i64().unwrap()..=imax.to_i64().unwrap() {
                            let mut e_i = Expr::List(vec![sym("replace_all")]);
                            e_i.push(table_body.clone());
                            let local_rule =
                                Expr::List(vec![sym("rule"), var.clone(), Expr::Int(i.into())]); // (rule var iter)
                            e_i.push(local_rule);
                            res.push(e_i);
                        }
                        return res;
                    } else {
                        // this is the sequence case where you just
                        // Table[expr,{i,{i1,i2,…}}]
                        dbg!("need an int or list for imax, reals not supported in iteration specification yet");
                        return sym("$Failed");
                    }
                } else if ls.len() == 5 {
                    // this is {i, imin, imax, di}
                    // this is {i, imin, imax}
                    if let [Expr::Int(imin), Expr::Int(imax), Expr::Int(di)] = &ls[2..] {
                        let rng = imin.to_i64().unwrap()..=imax.to_i64().unwrap();
                        let iter = rng.step_by(di.to_i64().unwrap() as usize);
                        for i in iter {
                            let mut e_i = Expr::List(vec![sym("replace_all")]);
                            e_i.push(table_body.clone());
                            let local_rule =
                                Expr::List(vec![sym("rule"), var.clone(), Expr::Int(i.into())]); // (rule var iter)
                            e_i.push(local_rule);
                            res.push(e_i);
                        }
                        return res;
                    } else {
                        // this is the sequence case where you just
                        // Table[expr,{i,{i1,i2,…}}]
                        dbg!("need an int or list for imax, reals not supported in iteration specification yet");
                        return sym("$Failed");
                    }
                }
            }
        }

        let range_lists = &evaluated_args[1..]; //.clone().reverse();
        // Table[ f[i,j], {i, imin, imax}, {j, jmin, jmax}]
        // Table[Table[f[i,j], {j, jmin, jmax}], {i, imin, imax}]
        // let mut ex = Expr::List(vec![sym("Table")]);
        // ex.push(table_body.clone());

        let mut nested_table = table_body.clone();
        for range in range_lists.iter().rev() {
            let mut new_table = Expr::List(vec![sym("Table"), nested_table.clone()]);
            new_table = match &mut new_table {
                Expr::List(ref mut v) => {
                    v.push(range.clone());
                    new_table.clone()
                }
                _ => panic!("Unexpected expression type"),
            };
            nested_table = new_table;
        }
        return nested_table;
    } else if nh == sym("Join") {
        if !matches!(&evaluated_args[0], Expr::List(ls)) {
            println!("Join joins lists dummy!");
            return reconstructed_ex;
        }

        let ha = head(&evaluated_args[0]);

        let mut res = vec![ha.clone()];
        for e in evaluated_args {
            if ha != head(&e) {
                println!("Join: heads of arguments are not all the same");
                return reconstructed_ex;
            }
            if let Expr::List(ls) = e {
                res.append(&mut ls[1..].to_vec());
            } else {
                //fix
                return sym("Failed");
            }
        }
        return Expr::List(res);
    } else if nh == sym("Timing") {
        let t1 = Instant::now();
        let res = evaluate(stack, ctx, &evaluated_args[0]);
        let dt = t1.elapsed(); // Capture the elapsed time

        // Convert duration to seconds
        let elapsed_seconds = dt.as_secs() as f64 + dt.subsec_nanos() as f64 * 1e-9;
        // NotNan
        Expr::List(vec![
            sym("List"),
            Expr::Real(NotNan::new(elapsed_seconds).unwrap()),
            res,
        ])
    } else if nh == sym("Export") {
        let dst = &evaluated_args[0];
        let ex = &evaluated_args[1];
        // println!("ex: {:?}", ex);
        let m = unpack_mat(ex.clone()).unwrap();
        let filename = match dst {
            Expr::Str(s) => s,
            _ => {
                println!("Export: first argument must be a string");
                return sym("$Failed");
            }
        };
        // create_svg_from_colors(m, filename, 50);
        return sym("Null");
    } else {
        return Expr::List(
            std::iter::once(nh.clone())
                .chain(evaluated_args.clone().to_owned())
                .collect(),
        );
    }
}

pub fn evaluate(stack: &mut Expr, ctx: &mut Cas3VM, expr: &Expr) -> Expr {
    let mut ex = expr.clone();
    let mut last_ex = None;

    loop {
        if Some(&ex) == last_ex.as_ref() {
            // If the expression hasn't changed, break the loop.
            break;
        }
        // println!("evaluating: {}", ex);

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
                let h;
                if let Some(sh) = ls.first() {
                    h = sh;
                } else {
                    println!("Expr::List needs a head");
                    return sym("$Failed");
                }
                // step 5
                let mut nh = evaluate(stack, ctx, h);

                // step 6
                // the use of a separate stack here is questionable
                // also to note that when we use "contains" on it, we also do a comparison against the head ("List"), which is wrong
                // but for practical purposes shouldn't cause a problem

                // the most important next step here is making sure that (attrs set) works correctly
                // note that something here causes SO

                // a potential problem here is that nh is not necesarily an atom
                // nh can be Expr::List

                // 3a (we might be able to just replace_all((attrs h) (down_values attrs)))
                // nvm 3a is less easy to see that a match wastn found

                // println!("hi");

                // 1. assume h::Sym
                // 2. get dvs of attr
                // 3. find a matching rule in dvs to (attrs h)
                // if no matching rule found, return "(List)"

                let mut nh_attrs = Expr::List(vec![sym("List")]);
                // #16 - this is what we need to speed up. ideally bypass the pattern matcher somehow
                // we know/can assume we are looking up (attrs SYM)
                if let Expr::Sym(_) = nh.clone() {
                    let te = ctx.vars.entry(sym("attrs")).or_insert_with(TableEntry::new);
                    // (down_values attrs)
                    let dvs = &te.down;
                    let attr_expr =
                        expr_parser::Expr(&format!("(attrs {})", nh).to_string()).unwrap();
                    // println!("attr_expr: {}", attr_expr);
                    if let Expr::List(_ls) = dvs {
                        // dv expected to be (rule_delayed (hold_pattern lhs) rhs)
                        for dv in &_ls[1..] {
                            let mut pos_map = HashMap::new();
                            let mut named_map = HashMap::new();
                            let pos = vec![];
                            if my_match(
                                attr_expr.clone(),
                                dv[1].clone(),
                                &pos,
                                &mut pos_map,
                                &mut named_map,
                            ) {
                                // println!("found attributes match for {} -> {}", nh, dv);
                                nh_attrs = replace(&attr_expr, dv);
                                break; // Exit the loop once a match is found
                            }
                        }
                    } else {
                        panic!("down_values must be a list");
                    }
                }

                // println!("nh_attrs: {:?}", nh_attrs);
                // assert!(head(&nh_attrs) == sym("List"));
                if nh_attrs.contains(&sym("HoldAllComplete")) {
                    // skip to 14
                    todo!();
                }

                // step 7
                let mut evaluated_args = vec![];

                // hold_mask entry with a zero means "don't hold"
                let mut hold_mask = vec![false; ls.len() - 1];

                // idk if it should be else ifs
                if nh_attrs.contains(&sym("HoldAll")) {
                    hold_mask.fill(true);
                }
                if nh_attrs.contains(&sym("HoldFirst")) {
                    hold_mask[0] = true;
                }
                if nh_attrs.contains(&sym("HoldRest")) {
                    hold_mask[1..].fill(true);
                }
                // println!("hold_mask: {:?}", hold_mask);

                for (i, p) in ls[1..].iter().enumerate() {
                    if hold_mask[i] {
                        evaluated_args.push(p.clone());
                    } else {
                        let ev = evaluate(stack, ctx, p);

                        evaluated_args.push(ev);
                    }
                }

                if !nh_attrs.contains(&sym("SequenceHold")) {
                    let mut arg_idx = 0;
                    while arg_idx < evaluated_args.len() {
                        if head(&evaluated_args[arg_idx]) == sym("Sequence") {
                            let seq_args: Vec<_> = evaluated_args[arg_idx][1..].to_vec();
                            evaluated_args.splice(arg_idx..arg_idx + 1, seq_args);
                        } else {
                            arg_idx += 1;
                        }
                    }
                }
                let reconstructed_ex = Expr::List(
                    std::iter::once(nh.clone())
                        .chain(evaluated_args.clone().to_owned())
                        .collect(),
                );
                // println!("reconstructed_ex: {}", reconstructed_ex);

                // step 14: apply user defined downvalues and subvalues
                let exprime = match nh.clone() {
                    // we dont need to panic here "abc"[foo] doesn't
                    Expr::Int(_) | Expr::Real(_) | Expr::Str(_) => {
                        // note: WL doesn't give note in this case
                        println!("head must be a symbol, got {nh}");
                        return reconstructed_ex;
                    }
                    // this is the down_value case, bcause the head
                    Expr::Sym(_) => {
                        let te = ctx.vars.entry(nh.clone()).or_insert_with(TableEntry::new);
                        let dvs = &te.down;
                        // println!("looking for user defined down_values for {} -> {}", nh, dvs);

                        // should this be replace_all? or replace_repeated?

                        let exprime = replace_all(&reconstructed_ex, dvs);
                        // println!("before: {}", reconstructed_ex);
                        // println!("after: {}", exprime);
                        exprime
                    }
                    // subvalue
                    Expr::List(_) => reconstructed_ex.clone(),
                };

                // im not sure if this is correct, but it seems necesary,
                // if we found a matching downvalue rule, then we need to re-evaluate the expression after replacement
                if ex != exprime {
                    ex = exprime;
                    continue;
                }

                // note now that ex is not necesarily a List anymore
                // so if we still have a list, then we do step 15, and apply internal down/subvalues

                match ex {
                    Expr::List(_) => {}
                    _ => continue,
                }
                nh = head(&ex);
                evaluated_args = ex[1..].to_vec();
                // this corresponds to step 15 in Wagner's main eval loop section
                // where we apply internal/builtin down and subvalues
                ex = internal_functions_apply(stack, ctx, nh, evaluated_args);
            }
        }
    }
    // println!("exiting evaluate: {}", ex);
    ex
}

fn named_rebuild_all(expr: Expr, map: &HashMap<Expr, Expr>) -> Expr {
    // First, check if the entire expression exists in the map and replace it if it does
    if let Some(replacement) = map.get(&expr) {
        return replacement.clone();
    }

    // If the expression is not in the map, proceed with the recursion
    match expr {
        Expr::List(list) => {
            // Recursively rebuild all sub-expressions in the list
            let new_list: Vec<Expr> = list
                .into_iter()
                .map(|e| named_rebuild_all(e, map))
                .collect();
            Expr::List(new_list)
        }
        _ => expr,
    }
}

// this adjusts lookups to the position map based on the sequences found in the current list
fn final_pos_map_rebuild(pos: Vec<usize>, pat: Expr, pos_map: &HashMap<Vec<usize>, Expr>) -> Expr {
    // println!("pos: {:?}, pat: {}", pos, pat);
    if let Some(replacement) = pos_map.get(&pos) {
        return replacement.clone();
    }

    match pat {
        Expr::List(es) => {
            let mut new_es = vec![];
            // note in the case of blank_null_seq, this can be negative
            let mut offset: isize = 0;
            for (i, e) in es.iter().enumerate() {
                let mut new_pos = pos.clone();
                let pos_in_list = i as isize + offset;
                // println!("{i}, pos_in_list: {} offset:{offset}", pos_in_list);
                new_pos.push(pos_in_list as usize);
                let new_e = final_pos_map_rebuild(new_pos, e.clone(), pos_map);
                if head(&new_e) == sym("Sequence") {
                    offset += new_e.len() as isize - 2; // its -2 becasue 1 for the head and 1 for the original (blank_seq) object
                }
                // so pos[0..i-1] is the position of the current list
                // we want to then look for all the position keys that have thsi as a prefix and adjust them
                // but i think we only want to adjust on a final rebuild but not in the middle.
                // when we are at the end of a list we are confirming the map works but haven't rebuilt anything, so our pat is still the original
                // println!("{i}, new_e: {} offest: {offset}", new_e);
                new_es.push(new_e);
            }
            Expr::List(new_es)
        }
        _ => pat,
    }
}

/// to fix pos_map. as you iterate es, and you keep track of the offset for the current list.
/// then you want to adjust lookups into the posmap based on the position - offset.
/// the reason we still need this is for intermediate rebuilds where we have already spliced former sequences
fn pos_map_rebuild(pos: Vec<usize>, pat: Expr, pos_map: &HashMap<Vec<usize>, Expr>) -> Expr {
    if let Some(replacement) = pos_map.get(&pos) {
        return replacement.clone();
    }

    match pat {
        Expr::List(es) => {
            let mut new_es = vec![];
            // note in the case of blank_null_seq, this can be negative
            for (i, e) in es.iter().enumerate() {
                let mut new_pos = pos.clone();
                new_pos.push(i);
                let new_e = pos_map_rebuild(new_pos, e.clone(), pos_map);
                new_es.push(new_e);
            }
            Expr::List(new_es)
        }
        _ => pat,
    }
}

fn splice_sequences(expr: Expr) -> Expr {
    match expr {
        Expr::List(mut list) => {
            let mut i = 0;
            while i < list.len() {
                list[i] = splice_sequences(list[i].clone());
                i += 1;
            }

            let mut new_list = Vec::new();
            let mut i = 0;
            while i < list.len() {
                let item = list[i].clone();
                if let Expr::List(ref sublist) = item {
                    if let Some(Expr::Sym(head)) = sublist.first() {
                        if head == "Sequence" {
                            new_list.extend_from_slice(&sublist[1..]);
                            i += 1;
                            continue;
                        }
                    }
                }
                new_list.push(item);
                i += 1;
            }
            Expr::List(new_list)
        }
        _ => expr,
    }
}

fn rebuild_and_splice(
    pat: Expr,
    pos: &Vec<usize>,
    pos_map: &HashMap<Vec<usize>, Expr>,
    named_map: &HashMap<Expr, Expr>,
) -> Expr {
    splice_sequences(named_rebuild_all(
        pos_map_rebuild(pos.clone(), pat, pos_map),
        named_map,
    ))
}

fn final_rebuild_and_splice(
    pat: Expr,
    pos: &Vec<usize>,
    pos_map: &HashMap<Vec<usize>, Expr>,
    named_map: &HashMap<Expr, Expr>,
) -> Expr {
    splice_sequences(named_rebuild_all(
        final_pos_map_rebuild(pos.clone(), pat, pos_map),
        named_map,
    ))
}

// we assume that p has a blank object for a head
fn is_blank_match(e: Expr, p: Expr) -> bool {
    if let Expr::List(ps) = p {
        if ps.len() == 2 {
            let p_head = &ps[1];
            if p_head == &head(&e) {
                true
            } else {
                false
            }
        } else {
            true
        }
    } else {
        panic!("is_blank_match needs a list for p")
    }
}

fn my_match(
    ex: Expr,
    mut pat: Expr,
    pos: &Vec<usize>,
    pos_map: &mut HashMap<Vec<usize>, Expr>,
    named_map: &mut HashMap<Expr, Expr>,
) -> bool {
    let pattern_expr = pat.clone();
    if head(&pattern_expr) == sym("hold_pattern") {
        pat = pattern_expr[1].clone();
    }
    // if head(&pat) != sym("attrs") && pat != sym("attrs") && ex != sym("matchq") {
    //     println!("M: {pos:?} | {ex} | {pat} | {pos_map:?} | {named_map:?}");
    // }

    let pat_syms = vec![sym("blank"), sym("blank_seq"), sym("blank_null_seq")];
    if head(&pat) == sym("Alternatives") {
        for p in &pat[1..] {
            if my_match(ex.clone(), p.clone(), pos, pos_map, named_map) {
                pos_map.insert(pos.clone(), p.clone());
                return true;
            }
        }
        return false;
    }

    match (ex.clone(), pat.clone()) {
        (Expr::List(es), Expr::List(ps)) => {
            // this first block determines if the pattern matches the entire list
            if ps[0] == sym("pattern") {
                if let Some(from_map) = named_map.get(&pat) {
                    return &ex == from_map;
                } else {
                    if is_blank_match(ex.clone(), ps[2].clone()) {
                        named_map.insert(pat.clone(), ex.clone());
                        return true;
                    }
                }
            } else if pat_syms.contains(&ps[0]) {
                if is_blank_match(ex.clone(), pat.clone()) {
                    pos_map.insert(pos.clone(), ex);
                    return true;
                }
            }

            let mut new_pos = pos.clone();
            new_pos.push(0); // we are at the head
            if !my_match(es[0].clone(), ps[0].clone(), &new_pos, pos_map, named_map) {
                return false;
            }

            'outer: for (i, pi) in ps.iter().enumerate().skip(1) {
                let mut new_pos = pos.clone();
                new_pos.push(i); // we are at the head
                if head(pi) == sym("pattern") {
                    // println!("in pattern pi ");
                    if let Some(_from_map) = named_map.get(&pi) {
                        // here is an example that contradicts the below i think
                        //  (setd (Times (pattern x (blank Sym)) (pattern x (blank Sym))) (Pow x 2))
                        println!("we should have rebuilt to remove this i think");
                    }
                    let b = &pi[2];
                    let bt = &b[0];
                    // let p_name = &pi[1];
                    if bt == &sym("blank_seq") {
                        for j in 1..=es[1..].len() {
                            let mut elts = vec![sym("Sequence")];
                            // im pretty sure this is not needed
                            if i + j > es.len() {
                                // println!("breaking news!");
                                break 'outer;
                            }
                            for seq_e in &es[i..i + j] {
                                if b.len() == 2 {
                                    let b_head = &b[1];
                                    if b_head != &head(seq_e) {
                                        break;
                                    }
                                }
                                elts.push(seq_e.clone());
                            }
                            let seq = liste(elts);
                            named_map.insert(pi.clone(), seq.clone());

                            let new_pat = rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);
                            // println!("new_pat in bs: at iter {j} {new_pat} {seq}");
                            if my_match(ex.clone(), new_pat, pos, pos_map, named_map) {
                                break 'outer;
                            }
                        }
                    } else if bt == &sym("blank_null_seq") {
                        for j in 0..=es[1..].len() {
                            let mut elts = vec![sym("Sequence")];
                            // im pretty sure this is not needed
                            if i + j > es.len() {
                                // println!("breaking news!");
                                break 'outer;
                            }
                            for seq_e in &es[i..i + j] {
                                if b.len() == 2 {
                                    let b_head = &b[1];
                                    if b_head != &head(seq_e) {
                                        break;
                                    }
                                }
                                elts.push(seq_e.clone());
                            }
                            let seq = liste(elts);
                            named_map.insert(pi.clone(), seq.clone());

                            let new_pat = rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);
                            // println!("new_pat in bs: at iter {j} {new_pat} {seq}");
                            if my_match(ex.clone(), new_pat, pos, pos_map, named_map) {
                                break 'outer;
                            }
                        }
                    } else {
                        if i >= es.len() {
                            break 'outer;
                        }
                        // named blank case
                        if !my_match(es[i].clone(), ps[i].clone(), &new_pos, pos_map, named_map) {
                            break 'outer;
                        }
                    }
                } else if head(pi) == sym("blank_seq") {
                    for j in 1..=es[1..].len() {
                        let mut elts = vec![sym("Sequence")];
                        // im pretty sure this is not needed
                        if i + j > es.len() {
                            // println!("breaking news!");
                            break 'outer;
                        }
                        for seq_e in &es[i..i + j] {
                            if pi.len() == 2 {
                                let b_head = &pi[1];
                                if b_head != &head(seq_e) {
                                    break;
                                }
                            }
                            elts.push(seq_e.clone());
                        }
                        let seq = liste(elts);
                        pos_map.insert(new_pos.clone(), seq.clone());

                        let new_pat = rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);
                        // println!("new_pat in bs: at iter {j} {new_pat}");
                        let mut copy = pos_map.clone();
                        // this is to avoid double application of a pos rule
                        copy.remove(&new_pos);
                        // if my_match(ex.clone(), pat.clone(), pos, &mut copy) {
                        if my_match(ex.clone(), new_pat, pos, &mut copy, named_map) {
                            pos_map.clear();
                            pos_map.extend(copy);

                            pos_map.insert(new_pos.clone(), seq.clone());

                            break 'outer;
                        } else {
                            // break 'outer;
                            // i think we need to revert pos_map to whatever it was before this my_match call
                        }
                    }
                } else if head(pi) == sym("blank_null_seq") {
                    for j in 0..=es[1..].len() {
                        let mut elts = vec![sym("Sequence")];
                        // im pretty sure this is not needed
                        if i + j > es.len() {
                            println!("breaking news!");
                            break 'outer;
                        }
                        for seq_e in &es[i..i + j] {
                            if pi.len() == 2 {
                                let b_head = &pi[1];
                                if b_head != &head(seq_e) {
                                    break;
                                }
                            }
                            elts.push(seq_e.clone());
                        }
                        let seq = liste(elts);
                        pos_map.insert(new_pos.clone(), seq.clone());

                        let new_pat = rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);
                        // println!("new_pat in bs: at iter {j} {new_pat}");
                        let mut copy = pos_map.clone();
                        // this is to avoid double application of a pos rule
                        copy.remove(&new_pos);
                        // if my_match(ex.clone(), pat.clone(), pos, &mut copy) {
                        if my_match(ex.clone(), new_pat, pos, &mut copy, named_map) {
                            pos_map.clear();
                            pos_map.extend(copy);

                            pos_map.insert(new_pos.clone(), seq.clone());

                            break 'outer;
                        } else {
                            // break 'outer;
                            // i think we need to revert pos_map to whatever it was before this my_match call
                        }
                    }
                } else {
                    if i >= es.len() {
                        break 'outer;
                    }
                    if !my_match(es[i].clone(), ps[i].clone(), &new_pos, pos_map, named_map) {
                        break 'outer;
                    }
                }
            }
            let final_pat = final_rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);
            // let final_pat = rebuild_and_splice(pat.clone(), &pos, pos_map, named_map);

            // todo remove these conditions when we fast path attribute lookup
            // if head(&pat) != sym("attrs") && pat != sym("attrs") {
            //     println!("final comparison: POS: {pos:?} | PAT: {pat} | NEW_PAT: {final_pat} | EX: {ex} || pos {pos_map:?} || named {named_map:?}");
            // }
            if final_pat == ex {
                return true;
            }
            false
        }
        (_, Expr::List(ps)) => {
            if ps[0] == sym("pattern") {
                if let Some(from_map) = named_map.get(&pat) {
                    return &ex == from_map;
                } else {
                    if is_blank_match(ex.clone(), ps[2].clone()) {
                        named_map.insert(pat.clone(), ex.clone());
                        true
                    } else {
                        false
                    }
                }
            } else if pat_syms.contains(&ps[0]) {
                if is_blank_match(ex.clone(), pat.clone()) {
                    pos_map.insert(pos.clone(), ex);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => ex == pat,
    }
}

pub fn bindings_to_rules(bindings: &HashMap<String, Expr>) -> Expr {
    let mut rules = Expr::List(vec![sym("List")]);
    for (name, binding) in bindings.clone() {
        rules.push(Expr::List(vec![sym("rule"), sym(&name), binding.clone()]));
    }
    rules
}

pub fn pat_bindings_to_rules(bindings: &HashMap<Expr, Expr>) -> Expr {
    let mut rules = Expr::List(vec![sym("List")]);
    for (pat, binding) in bindings.clone() {
        if let Expr::List(ps) = pat {
            let p_name = &ps[1]; // (pattern x (blank))
            rules.push(Expr::List(vec![
                sym("rule"),
                p_name.clone(),
                binding.clone(),
            ]));
        }
    }
    rules
}

pub fn norm_rules(rules: &Expr) -> Vec<Expr> {
    if head(rules) == sym("rule") || head(rules) == sym("rule_delayed") {
        return vec![rules.clone()];
    } else {
        assert_eq!(head(rules), sym("List"));
        return rules.clone()[1..].to_vec();
    };
}

pub fn replace(expr: &Expr, rules: &Expr) -> Expr {
    let rules_list = norm_rules(rules);

    for rule in rules_list {
        let pos = vec![];
        let mut pos_map = HashMap::new();
        let mut named_map = HashMap::new();
        assert!(head(&rule) == sym("rule") || head(&rule) == sym("rule_delayed"));
        if my_match(
            expr.clone(),
            rule[1].clone(),
            &pos,
            &mut pos_map,
            &mut named_map,
        ) {
            let mut new_expr = rule[2].clone();
            new_expr = replace_all(&new_expr, &pat_bindings_to_rules(&named_map));

            return new_expr;
        }
    }
    expr.clone()
}

pub fn replace_all(expr: &Expr, rules: &Expr) -> Expr {
    let rules_list = norm_rules(rules);
    for rule in rules_list {
        let pos = vec![];
        let mut pos_map = HashMap::new();
        let mut named_map = HashMap::new();
        assert!(head(&rule) == sym("rule") || head(&rule) == sym("rule_delayed"));
        if my_match(
            expr.clone(),
            rule[1].clone(),
            &pos,
            &mut pos_map,
            &mut named_map,
        ) {
            return replace(expr, &rule);
        }
    }

    match expr {
        Expr::List(list) => {
            let new_list: Vec<Expr> = list
                .iter()
                .map(|sub_expr| replace_all(sub_expr, rules))
                .collect();
            Expr::List(new_list)
        }
        _ => replace(expr, rules),
    }
}

pub fn replace_repeated(expr: &Expr, rules: &Expr) -> Expr {
    let mut current_expr = expr.clone();
    let mut i = 0;
    loop {
        let new_expr = replace_all(&current_expr, rules);
        if new_expr == current_expr {
            break;
        }
        current_expr = new_expr;
        i += 1;
        if i > 1 << 16 {
            println!("replace_repeated, iteration limit 1<<16 reached");
            break;
        }
    }
    current_expr
}

pub fn startup_attrs(ctx: &mut Cas3VM) {
    let attrs_te = ctx.vars.entry(sym("attrs")).or_insert_with(TableEntry::new);
    let mut exs = vec![
        format!("(rule_delayed (hold_pattern (attrs hold_pattern)) (list HoldAll))"),
        format!("(rule_delayed (hold_pattern (attrs attrs)) (list HoldAll))"),
        format!("(rule_delayed (hold_pattern (attrs rule_delayed)) (list HoldRest SequenceHold))"),
        format!("(rule_delayed (hold_pattern (attrs set)) (list HoldFirst SequenceHold))"),
        format!("(rule_delayed (hold_pattern (attrs down_values)) (list HoldAll))"),
    ]
        .iter_mut()
        .map(|s| expr_parser::Expr(&s).unwrap())
        .collect();
    attrs_te.down.append(&mut exs);
}

impl Cas3VM {
    pub fn run_file<P: AsRef<Path>>(&mut self, filepath: P) -> std::io::Result<Expr> {
        let filepath = filepath.as_ref();
        // let file = File::open(filepath)?;
        // let reader = BufReader::new(file);
        let file_contents = std::fs::read_to_string(filepath)?;
        // i dont love this because it's ambigious whether or not something failed in reading the file or sth
        // or if the last expr in the file was a setd or something that returns a Null
        println!("Running file: {}", filepath.display());
        let mut res = sym("Null");
        let exprs = expr_parser::expressions(&file_contents).unwrap();
        // for line in reader.lines() {
        for expr in exprs {
            // match line {
            // Ok(content) => {
            //     if content.starts_with("//") || content.starts_with(";") || content.is_empty() {
            //         continue;
            //     }
            // if let Ok(ex) = &expr_parser::Expr(&content) {
            let mut stack = Expr::List(vec![]);
            res = evaluate(&mut stack, self, &expr);
            // } else {
            // eprintln!("Error parsing a line: {:?}", content);
            // }
        }
        // Err(error) => {
        // eprintln!("Error reading a line: {:?}", error);
        // }
        // }
        // }

        Ok(res)
    }
}




pub fn evalparse(s: &str) -> Expr {
    let ex = expr_parser::Expr(s);
    match ex {
        Ok(expr) => {
            let mut ctx = Cas3VM {
                vars: HashMap::new(),
            };
            let mut stack = Expr::List(vec![]);
            evaluate(&mut stack, &mut ctx, &expr)
        }
        Err(err) => panic!("Failed to parse: {s}: {err}"),
    }
}

pub fn ctx_evalparse(ctx: &mut Cas3VM, s: &str) -> Expr {
    let ex = expr_parser::Expr(s);
    match ex {
        Ok(expr) => {
            let mut stack = Expr::List(vec![]);
            evaluate(&mut stack, ctx, &expr)
        }
        Err(err) => panic!("Failed to parse: {s}: {err}"),
    }
}

