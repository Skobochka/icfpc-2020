use std::collections::{BTreeMap,BTreeSet};
use std::io::{BufReader,BufRead};

use common::code::*;
use common::vm::{
    interpret::*,
    Env,
};


fn repeat<'s>(map: &'s BTreeMap<String,Vec<String>>, key: &str, v: &mut Vec<String>, excl: &BTreeSet<String>) -> Result<(),String> {
    println!("Key: {}",key);
    for k in map.get(key).ok_or(format!("No key: {}",key))?.iter() {
        if !excl.contains(k) {
            if let Some(':') = k.chars().next() { repeat(map,k,v,excl)?; }
            else { v.push(k.to_string()); }
        } else {
            v.push(k.to_string());
        }
    }
    Ok(())
}

fn repeat_once<'s>(map: &'s BTreeMap<String,Vec<String>>, key: &str) -> Result<(),String> {
    for k in map.get(key).ok_or(format!("No key: {}",key))?.iter() {
        if k == key { return Err("Recursion".to_string()); }
    }
    Ok(())
}

fn map_v(map: &BTreeMap<String,Vec<String>>, keys: Vec<String>) -> Result<Vec<String>,String> {
    let mut v = Vec::new();
    for k in keys {
        if let Some(':') = k.chars().next() {
            for k in map.get(&k).ok_or(format!("No key: {}",k))?.iter() {
                v.push(k.to_string());
            }
        } else { v.push(k.to_string()); }
    }
    Ok(v)
}

#[derive(Debug,Clone)]
pub enum Token {
    Op(Op),
    Rewrite(String),
    Unknown(String),
}

#[derive(Debug,Clone)]
pub struct Tokens (pub Vec<Token>);
impl Tokens {
    pub fn from(v: &Vec<String>) -> Tokens {
        let mut new_v = Vec::new();
        for s in v {
            if let Ok(num) = i64::from_str_radix(s,10) {
                new_v.push(Token::Op(Op::Const(Const::EncodedNumber(match num >= 0 {
                    true => EncodedNumber{
                        number: Number::Positive(PositiveNumber{ value: num as usize }),
                        modulation: Modulation::Demodulated,
                    },
                    false => EncodedNumber {
                        number: Number::Negative(NegativeNumber{ value: num as isize }),
                        modulation: Modulation::Demodulated,
                    },
                }))));
                continue
            }
            match &s[..] {
                "ap" => new_v.push(Token::Op(Op::App)),
                "b" => new_v.push(Token::Op(Op::Const(Const::Fun(Fun::B)))),
                "s" => new_v.push(Token::Op(Op::Const(Const::Fun(Fun::S)))),
                "c" => new_v.push(Token::Op(Op::Const(Const::Fun(Fun::C)))),
                "cons" => new_v.push(Token::Op(Op::Const(Const::Fun(Fun::Cons)))),
                "nil" => new_v.push(Token::Op(Op::Const(Const::Fun(Fun::Nil)))),
                s @_ if s.starts_with(":") => new_v.push(Token::Rewrite(s.to_string())),
                _ => new_v.push(Token::Unknown(s.to_string())),
            }
        }
        Tokens(new_v)
    }
    fn try_ops(&self) -> Result<Ops,Token> {
        let mut new_v = Vec::new();
        for t in &self.0 {
            match t {
                Token::Op(o) => new_v.push(o.clone()),
                t @ _ => return Err(t.clone()),
            }
        }
        Ok(Ops(new_v))
    }
}

fn main() -> Result<(),String> {
    let key = ":1111";
    let row = "ap ap b ap b ap cons 2 ap ap c ap ap b b cons ap ap c cons nil".split(" ").map(|s|s.to_string()).collect::<Vec<_>>();
    let tokens = Tokens::from(&row);
    //println!("Tokens: {:?}",tokens);
    match tokens.try_ops() {
        Ok(ops) => {
            let vm = Interpreter{};
            let mut env = Env{};
            println!("Ops:    {:?}",ops);
            match vm.build_tree(ops) {
                Ok(tree) => {
                    match vm.eval(tree,&mut env) {
                        Ok(ops) => println!("Result: {:?}",ops),
                        Err(e) => println!("Try eval error: {:?}",e),
                    }
                },
                Err(e) => println!("Try tree error: {:?}",e),
            }
        },
        Err(t) => println!("Try ops error on: {:?}",t),
    }

    Ok(())
}

/*
fn main() -> Result<(),String> {
    let mut map = BTreeMap::new();
    for ln in BufReader::new(std::io::stdin()).lines() {
        let s = ln.map_err(|e| format!("{:?}",e))?;
        let mut v = s.split("=");
        let key = match v.next() {
            Some(s) => s.trim(),
            None => return Err("Zero split".to_string()),
        };
        let value = match v.next() {
            Some(s) => s.trim(),
            None => return Err("One split".to_string()),
        };
        map.insert(key.to_string(),value.split(" ").filter_map(|s| match s.trim() {
            "" => None,
            s @ _ => Some(s.to_string()),
        }).collect::<Vec<_>>());
    }


    //repeat(&map,"galaxy",&mut v,&excl)?;
    //println!("{:?}",v);
    /*for (k,v) in &map {
        if let Err(_) = repeat_once(&map,k) {
            print!("{:?}, ",k);
        }
}*/
    let key = ":1111";
    let tokens = Tokens::from(map.get(key).ok_or(format!("No key: {}",key))?);
    //println!("Tokens: {:?}",tokens);
    match tokens.try_ops() {
        Ok(ops) => {
            let vm = Interpreter{};
            let mut env = Env{};
            println!("Ops:    {:?}",ops);
            match vm.build_tree(ops) {
                Ok(tree) => {
                    match vm.eval(tree,&mut env) {
                        Ok(ops) => println!("Result: {:?}",ops),
                        Err(e) => println!("Try eval error: {:?}",e),
                    }
                },
                Err(e) => println!("Try tree error: {:?}",e),
            }
        },
        Err(t) => println!("Try ops error on: {:?}",t),
    }

    Ok(())
}
*/
