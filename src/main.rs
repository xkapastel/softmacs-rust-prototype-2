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

extern crate softmacs;

use std::io::Write;
use softmacs::Lisp;

fn main() {
  let mut source_buffer = String::new();
  let mut target_buffer = String::new();
  let mut lisp = softmacs::v0::init(1024);
  let mut uid = 0;
  loop {
    print!("⊥@softmacs\n> ");
    source_buffer.clear();
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut source_buffer).unwrap();
    let xs = lisp.read(&source_buffer).unwrap();
    for pointer in xs.iter() {
      target_buffer.clear();
      lisp.show(*pointer, &mut target_buffer).unwrap();
      println!("{} = {}", format!("${}", uid), &target_buffer);
      uid += 1;
    }
  }
}
