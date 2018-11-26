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
use std::result::Result;
use std::fmt::Debug;

pub trait Lisp {
  type Value: Copy;
  type Error: Debug;
  fn unit(&mut self) -> Result<Self::Value, Self::Error>;
  fn t(&mut self) -> Result<Self::Value, Self::Error>;
  fn f(&mut self) -> Result<Self::Value, Self::Error>;
  fn pair(&mut self, fst: Self::Value, snd: Self::Value) -> Result<Self::Value, Self::Error>;
  fn symbol(&mut self, value: Rc<str>) -> Result<Self::Value, Self::Error>;
  fn read(&mut self, src: &str) -> Result<Vec<Self::Value>, Self::Error>;
  fn show(&self, value: Self::Value, buffer: &mut String) -> Result<(), Self::Error>;
}

pub mod v0;
