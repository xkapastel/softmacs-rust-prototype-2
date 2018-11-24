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

#[derive(Debug, Clone)]
enum Error {
  Stub,
  Read,
  Time,
  Space,
  Type,
  Assert,
  Pointer,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Gc {
  index: usize,
  timestamp: usize,
}

#[derive(Debug, Clone)]
enum Object {
  Null,
  Pair(Gc, Gc),
  List(Gc, Gc),
  Symbol(Rc<str>),
}

#[derive(Debug, Clone)]
enum Node {
  None,
  Some(Object, usize),
  Mark(Object, usize),
}

#[derive(Debug, Clone)]
struct Heap {
  nodes: Vec<Node>,
  time: usize,
}

impl Object {
  fn is_null(&self) -> bool {
    match self {
      &Object::Null => true,
      _ => false,
    }
  }

  fn is_list(&self) -> bool {
    match self {
      &Object::Null | &Object::List(_, _) => true,
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
    return Err(Error::Stub);
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
            &Object::Null => {}
            &Object::Symbol(_) => {}
            &Object::Pair(ref fst, ref snd) |
            &Object::List(ref fst, ref snd) => {
              to_mark.push(*fst);
              to_mark.push(*snd);
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

fn read(src: &str, heap: &mut Heap) -> Result<Vec<Gc>> {
  let runes = src.chars().collect();
  let tokens = tokenize(&runes);
  return parse(&tokens, heap);
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

fn parse(src: &Vec<Token>, heap: &mut Heap) -> Result<Vec<Gc>> {
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
            let mut xs = heap.put(Object::Null)?;
            for pointer in pointers.iter().rev() {
              xs = heap.put(Object::List(*pointer, xs))?;
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
        let pointer = heap.put(Object::Symbol(body.clone()))?;
        pointers.push(pointer);
        index += 1;
      }
    }
  }
  return Ok(pointers);
}

fn show(pointer: Gc, buf: &mut String, heap: &Heap) -> Result<()> {
  match heap.get(pointer)? {
    Object::Null => {
      buf.push_str("null");
    }
    Object::Symbol(value) => {
      buf.push_str(&value);
    }
    Object::Pair(fst, snd) => {
      buf.push('(');
      show(fst, buf, heap)?;
      buf.push_str(" * ");
      show(snd, buf, heap)?;
      buf.push(')');
    }
    Object::List(head, tail) => {
      let mut xs = pointer;
      buf.push('(');
      loop {
        match heap.get(xs)? {
          Object::Null => {
            buf.push(')');
            return Ok(());
          }
          Object::List(head, tail) => {
            show(head, buf, heap)?;
            if !heap.get(tail)?.is_null() {
              buf.push(' ');
            }
            xs = tail;
          }
          _ => {
            return Err(Error::Type);
          }
        }
      }
    }
  }
  return Ok(());
}

use std::io::Write;

fn main() {
  let mut source_buffer = String::new();
  let mut target_buffer = String::new();
  let mut heap = Heap::with_capacity(1024);
  let mut uid = 0;
  loop {
    print!("⊥@softmacs\n> ");
    source_buffer.clear();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut source_buffer).unwrap();
    let xs = read(&source_buffer, &mut heap).unwrap();
    for pointer in xs.iter() {
      target_buffer.clear();
      show(*pointer, &mut target_buffer, &heap).unwrap();
      println!("{} = {}", format!("${}", uid), &target_buffer);
      uid += 1;
    }
  }
}
