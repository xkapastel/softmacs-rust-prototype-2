// This file is a part of Softmacs.
// Copyright (C) 2018 Matthew Blount

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public
// License along with this program.  If not, see
// <https://www.gnu.org/licenses/.

use std::rc::Rc;
use std::collections::HashSet;
use super::Lisp;

#[derive(Debug, Clone)]
pub enum Error {
  Stub,
  Read,
  Time,
  Space,
  Type,
  Guard,
  Pointer,
}

type Result<T> = std::result::Result<T, Error>;

fn guard(flag: bool) -> Result<()> {
  if flag {
    return Ok(());
  }
  return Err(Error::Guard);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Gc {
  index: usize,
  timestamp: usize,
}

type RestFn = Fn(Gc, &mut V0) -> Result<Gc>;

#[derive(Clone)]
struct Symbol(Rc<str>);

#[derive(Clone)]
struct Pair {
  fst: Gc,
  snd: Gc,
  is_list: bool,
}

#[derive(Clone)]
enum Nat {
  Pair,
  Fst,
  Snd,
  Eval,
  Init,
  Shift,
  Reset,
  And,
  Or,
  Not,
}

#[derive(Clone)]
struct App(Gc);

#[derive(Clone)]
struct Abs {
  head: Gc,
  tail: Gc,
  lexical: Gc,
  dynamic: Gc,
}

#[derive(Clone)]
enum Proc {
  Nat(Nat),
  App(App),
  Abs(Abs),
}

#[derive(Clone)]
enum Object {
  Unit,
  Bool(bool),
  Symbol(Symbol),
  Pair(Pair),
  Proc(Proc),
}

#[derive(Clone)]
enum Node {
  None,
  Some(Object, usize),
  Mark(Object, usize),
}

#[derive(Clone)]
struct Heap {
  nodes: Vec<Node>,
  time: usize,
}

impl Object {
  fn is_unit(&self) -> bool {
    match self {
      &Object::Unit => true,
      _ => false,
    }
  }
}

impl Node {
  fn is_none(&self) -> bool {
    match self {
      Node::None => true,
      _ => false,
    }
  }

  fn is_some(&self) -> bool {
    match self {
      Node::Some(_, _) => true,
      _ => false,
    }
  }

  fn is_mark(&self) -> bool {
    match self {
      Node::Mark(_, _) => true,
      _ => false,
    }
  }
}

impl Heap {
  fn with_capacity(capacity: usize) -> Self {
    let mut nodes: Vec<Node> = Vec::with_capacity(capacity);
    for _ in 0..capacity {
      nodes.push(Node::None);
    }
    Heap {
      nodes: nodes,
      time: 0,
    }
  }

  fn put(&mut self, object: Object) -> Result<Gc> {
    for (index, node) in self.nodes.iter_mut().enumerate() {
      if !node.is_none() {
        continue;
      }
      *node = Node::Some(object, self.time);
      let pointer = Gc { index: index, timestamp: self.time };
      self.time += 1;
      return Ok(pointer);
    }
    return Err(Error::Space);
  }

  fn get(&self, pointer: Gc) -> Result<Object> {
    match &self.nodes[pointer.index] {
      &Node::Some(ref object, timestamp) | &Node::Mark(ref object, timestamp) => {
        if pointer.timestamp != timestamp {
          return Err(Error::Pointer);
        }
        return Ok(object.clone());
      }
      &Node::None => {
        return Err(Error::Pointer);
      }
    }
  }

  fn mark(&mut self, pointer: Gc) -> Result<()> {
    let mut seen = HashSet::new();
    let mut mark = vec![pointer];
    let mut to_mark = vec![];
    while !mark.is_empty() {
      match &self.nodes[pointer.index] {
        &Node::None => {
          return Err(Error::Pointer);
        }
        &Node::Some(ref object, timestamp) | &Node::Mark(ref object, timestamp) => {
          match object {
            &Object::Unit => {}
            &Object::Bool(_) => {}
            &Object::Symbol(_) => {}
            &Object::Pair(ref value) => {
              to_mark.push(value.fst);
              to_mark.push(value.snd);
            }
            &Object::Proc(ref proc) => {
              match proc {
                &Proc::Nat(_) => {}
                &Proc::App(ref value) => {
                  to_mark.push(value.0);
                }
                &Proc::Abs(ref value) => {
                  to_mark.push(value.head);
                  to_mark.push(value.tail);
                  to_mark.push(value.lexical);
                  to_mark.push(value.dynamic);
                }
              }
            }
          }
        }
      }
      for pointer in to_mark.iter() {
        if !seen.contains(pointer) {
          seen.insert(*pointer);
          mark.push(*pointer);
        }
      }
      to_mark.clear();
    }
    return Ok(());
  }

  fn sweep(&mut self) {
    let mut count = 0;
    for node in self.nodes.iter_mut() {
      match node {
        &mut Node::None => {}
        &mut Node::Some(ref object, timestamp) => {
          *node = Node::None;
          count += 1;
        }
        &mut Node::Mark(ref object, timestamp) => {
          *node = Node::Some(object.clone(), timestamp);
        }
      }
    }
    println!("[gc] deleted {} objects", count);
    self.time += 1;
  }
}

#[derive(Debug, Clone)]
enum Token {
  Lparen,
  Rparen,
  Space(Rc<str>),
  Symbol(Rc<str>),
}

fn tokenize(src: &Vec<char>) -> Vec<Token> {
  let mut index = 0;
  let mut tokens = vec![];
  while index < src.len() {
    let rune = src[index];
    match rune {
      '(' => {
        tokens.push(Token::Lparen);
        index += 1;
      }
      ')' => {
        tokens.push(Token::Rparen);
        index += 1;
      }
      ' ' | '\t' | '\r' | '\n' => {
        let mut buf = String::new();
        while index < src.len() {
          let rune = src[index];
          match rune {
            ' ' | '\t' | '\r' | '\n' => {
              buf.push(rune);
              index += 1;
            }
            _ => { break }
          }
        }
        let space = Rc::from(buf.as_str());
        tokens.push(Token::Space(space));
      }
      _ => {
        let mut buf = String::new();
        while index < src.len() {
          let rune = src[index];
          match rune {
            '(' | ')' | ' ' | '\t' | '\r' | '\n' => { break }
            _ => {
              buf.push(rune);
              index += 1;
            }
          }
        }
        let body = Rc::from(buf.as_str());
        let token = Token::Symbol(body);
        tokens.push(token);
      }
    }
  }
  return tokens;
}

fn parse(src: &Vec<Token>, lisp: &mut V0) -> Result<Vec<Gc>> {
  let mut index = 0;
  let mut pointers = vec![];
  let mut stack = vec![];
  while index < src.len() {
    match &src[index] {
      &Token::Lparen => {
        stack.push(pointers);
        pointers = vec![];
        index += 1;
      }
      &Token::Rparen => {
        match stack.pop() {
          Some(prev) => {
            let mut xs = lisp.unit()?;
            for pointer in pointers.iter().rev() {
              xs = lisp.pair(*pointer, xs)?;
            }
            pointers = prev;
            pointers.push(xs);
            index += 1;
          }
          None => {
            return Err(Error::Read);
          }
        }
      }
      &Token::Space(ref body) => {
        index += 1;
      }
      &Token::Symbol(ref body) => {
        let pointer;
        if body.starts_with("#") {
          match &**body {
            "#" => {
              pointer = lisp.unit()?;
            }
            "#t" => {
              pointer = lisp.t()?;
            }
            "#f" => {
              pointer = lisp.f()?;
            }
            _ => {
              return Err(Error::Read);
            }
          }
        } else {
          pointer = lisp.symbol(body.clone())?;
        }
        pointers.push(pointer);
        index += 1;
      }
    }
  }
  return Ok(pointers);
}

fn eval(
  value: Gc,
  env: Gc,
  lisp: &mut V0,
  rest: &RestFn) -> Result<Gc> {
  return Err(Error::Stub);
}

fn exec(
  value: Gc,
  env: Gc,
  lisp: &mut V0,
  rest: &RestFn) -> Result<Gc> {
  return Err(Error::Stub);
}

fn evlis(
  value: Gc,
  env: Gc,
  lisp: &mut V0,
  rest: &RestFn) -> Result<Gc> {
  return Err(Error::Stub);
}

fn apply(
  proc: Gc,
  value: Gc,
  env: Gc,
  lisp: &mut V0,
  rest: &RestFn) -> Result<Gc> {
  return Err(Error::Stub);
}

struct V0 {
  heap: Heap,
}

impl super::Lisp for V0 {
  type Value = Gc;
  type Error = Error;

  fn unit(&mut self) -> Result<Self::Value> {
    let object = Object::Unit;
    return self.heap.put(object);
  }

  fn t(&mut self) -> Result<Self::Value> {
    let object = Object::Bool(true);
    return self.heap.put(object);
  }

  fn f(&mut self) -> Result<Self::Value> {
    let object = Object::Bool(false);
    return self.heap.put(object);
  }

  fn symbol(
    &mut self,
    value: Rc<str>) -> Result<Self::Value> {
    let symbol = Symbol(value);
    let object = Object::Symbol(symbol);
    return self.heap.put(object);
  }

  fn pair(
    &mut self,
    fst: Self::Value,
    snd: Self::Value) -> Result<Self::Value> {
    let is_list: bool;
    match self.heap.get(snd)? {
      Object::Unit            => { is_list = true }
      Object::Pair(ref value) => { is_list = value.is_list }
      _                       => { is_list = false }
    }
    let pair = Pair { fst: fst, snd: snd, is_list: is_list };
    let object = Object::Pair(pair);
    return self.heap.put(object);
  }

  fn eval(
    &mut self,
    value: Self::Value,
    env: Self::Value) -> Result<Self::Value> {
    return eval(value, env, self, &|value, lisp| Ok(value));
  }

  fn read(
    &mut self,
    src: &str) -> Result<Vec<Self::Value>> {
    let runes = src.chars().collect();
    let tokens = tokenize(&runes);
    return parse(&tokens, self);
  }

  fn show(
    &self,
    pointer: Self::Value,
    buf: &mut String) -> Result<()> {
    match self.heap.get(pointer)? {
      Object::Unit => {
        buf.push_str("#");
      }
      Object::Bool(value) => {
        if value {
          buf.push_str("#t");
        } else {
          buf.push_str("#f");
        }
      }
      Object::Symbol(ref value) => {
        buf.push_str(&value.0);
      }
      Object::Pair(ref value) => {
        if !value.is_list {
          buf.push('(');
          self.show(value.fst, buf)?;
          buf.push_str(" * ");
          self.show(value.snd, buf)?;
          buf.push(')');
        } else {
          buf.push('(');
          let mut xs = pointer;
          while let Object::Pair(ref value) = self.heap.get(xs)? {
            self.show(value.fst, buf)?;
            if !self.heap.get(value.snd)?.is_unit() {
              buf.push(' ');
            }
            xs = value.snd;
          }
          guard(self.heap.get(xs)?.is_unit())?;
          buf.push(')');
        }
      }
      Object::Proc(_) => {
        buf.push_str("<procedure>");
      }
    }
    return Ok(());
  }
}

pub fn init(capacity: usize) -> impl super::Lisp<Error=Error> {
  V0 {
    heap: Heap::with_capacity(capacity),
  }
}
