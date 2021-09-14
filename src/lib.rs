//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                            Load Libraries                            //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::fs::OpenOptions;
use std::io::{Read, BufReader};
use kdbplus::api::*;

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                          Global Variables                            //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// One day in nanoseconds.
const ONE_DAY_NANOS: i64 = 86_400_000_000_000;

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                               Macros                                 //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Parse [token_idx]th value in a vector of tokens into int or real. The parsed value is pushed
///  to the given q list object which is a [slice_idx]th element of a root list.
#[macro_export(crate)]
macro_rules! parse_token {
  [i32; $tokens:expr, $token_idx:expr, $slice:expr, $slice_idx:expr, $error:expr] => {
    match $tokens[$token_idx].parse::<i32>(){
      Ok(num) => {
        $slice[$slice_idx].push_raw(num).unwrap();
      },
      Err(_) => return Err($error)
    }
  };

  [f64; $tokens:expr, $token_idx:expr, $slice:expr, $slice_idx:expr, $error:expr] => {
    match $tokens[$token_idx].parse::<f64>(){
      Ok(num) => {
        $slice[$slice_idx].push_raw(num as f32).unwrap();
      },
      Err(_) => return Err($error)
    }
  };
}

/// Set string contents of a file or a value of a given argument to `string`.
#[macro_export(crate)]
macro_rules! set_string {
  ($data: expr, $string: expr, $contents_buffer: expr) => {
    if $data.get_type() == qtype::SYMBOL_ATOM{
      if let Err(error) = load_ascii_data($data, &mut $contents_buffer){
        return new_error(error);
      }
      else{
        $string=$contents_buffer.as_str();
      }
    }
    else{
      match $data.get_str(){
        Ok(string_) => {
          $string = string_;
        },
        Err(error) => {
          return new_error(error)
        }
      }
    }
  };
}

/// Set binary contents of a file or a value of a given argument to `bytes`.
#[macro_export(crate)]
macro_rules! set_bytes {
  ($data: expr, $bytes: expr, $contents_buffer: expr) => {
    if $data.get_type() == qtype::SYMBOL_ATOM{
      if let Err(error) = load_binary_data($data, &mut $contents_buffer){
        return new_error(error);
      }
      else{
        $bytes=$contents_buffer.as_mut_slice();
      }
    }
    else{
      $bytes=$data.as_mut_slice::<G>();
    }
  };
}

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                          Private Fucntions                           //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Read ASCII file into `String` buffer.
fn load_ascii_data(data: K, contents_buffer: &mut String) -> Result<(), &'static str>{
  // File path
  if let Some(path) = data.get_symbol().unwrap().strip_prefix(':'){
    if let Ok(file) = OpenOptions::new().read(true).write(false).create(false).open(path){
      let mut reader=BufReader::new(file);
      reader.read_to_string(contents_buffer).unwrap();
      Ok(())
    }
    else{
      return Err("no such file\0");
    }
  }
  else{
    return Err("invalid file name - missing ':'\0");
  }
}

/// Read binary file into `Vec<u8> buffer`.
fn load_binary_data(data: K, contents_buffer: &mut Vec<u8>) -> Result<(), &'static str>{
  if let Some(path) = data.get_symbol().unwrap().strip_prefix(':'){
    if let Ok(file) = OpenOptions::new().read(true).write(false).create(false).open(path){
      let mut reader=BufReader::new(file);
      reader.read_to_end(contents_buffer).unwrap();
      Ok(())
    }
    else{
      return Err("no such file\0");
    }
  }
  else{
    return Err("invalid file name - missing ':'\0");
  }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                            Load Modules                              //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

pub mod config;
pub mod data;
pub mod info;
