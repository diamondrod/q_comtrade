//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                            Load Libraries                            //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

use kdbplus::*;
use kdbplus::api::*;
use super::load_ascii_data;

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                          Private Fucntions                           //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize a section of the information file (.inf) of COMTRADE format.
fn deserialize_comtrade_info_section(line: &str, mut keys: K) -> Result<K, &'static str>{
  // Starting with '[' is assured by the routing of the top level function.
  if let Some(line) = line[1..line.len()].strip_suffix(']'){
    if let Some(space) = line.find(' '){
      let key = line[space+1..line.len()].replace(' ', "_");
      let new_key_location=keys.push_symbol(key.as_str()).unwrap();
      Ok(new_key_location)
    }
    else{
      Err("invalid section - missing space after 'Public' or other header?\0")
    }
  }
  else{
    Err("invalid section - missing ']'\0")
  }
}

/// Deserialize an entry line of the information file (.inf) of COMTRADE format.
fn deserialize_comtrade_info_entry(line: &str, mut entry_keys: K, mut entry_values: K) -> Result<(K, K), &'static str>{
  if let Some((key, values)) = line.split_once('='){
    let new_key_location=entry_keys.push_symbol(key).unwrap();
    let mut this_entry_values=new_list(qtype::COMPOUND_LIST, 0);
    values.split(',').collect::<Vec<&str>>().into_iter().for_each(|value|{
      this_entry_values.push(new_string(value)).unwrap();
    });
    let new_value_location=entry_values.push(this_entry_values).unwrap();
    Ok((new_key_location, new_value_location))
  }
  else{
    Err("invalid entry - missing '='\0")
  }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                               Interface                              //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize the information file (`.inf`) of COMTRADE format into q dictionary.
/// # Parameters
/// - `data`: 
///   - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
///   - string: File contents.
#[no_mangle]
pub extern "C" fn deserialize_comtrade_info(data: K) -> K{
  let string;
  let mut contents_buffer = String::new();
  // Load data into string.
  set_string!(data, string, contents_buffer);

  // inf fle can include empty lines
  let lines= string.split_terminator("\r\n").filter(|line| *line != "").collect::<Vec<&str>>();
  let mut keys = new_list(qtype::SYMBOL_LIST, 0);
  let mut values = new_list(qtype::COMPOUND_LIST, 0);
  let mut cursor = 0;
  while cursor != lines.len(){
    match deserialize_comtrade_info_section(lines[cursor], keys){
      Ok(new_key_location) => {
        // Can be reallocated after push
        keys = new_key_location;
        cursor+=1;
        let mut entry_keys=new_list(qtype::SYMBOL_LIST, 0);
        let mut entry_values=new_list(qtype::COMPOUND_LIST, 0);
        loop{
          if (cursor != lines.len()) && (!lines[cursor].starts_with('[')){
            match deserialize_comtrade_info_entry(lines[cursor], entry_keys, entry_values){
              Ok((new_key_location, new_value_location)) => {
                // Can be reallocated after push
                entry_keys=new_key_location;
                entry_values=new_value_location;
                cursor+=1;
              },
              Err(error) => {
                return new_error(error);
              }
            }
          }
          else{
            // Append a dictionary of entries as a section value.
            values.push(new_dictionary(entry_keys, entry_values)).unwrap();
            // Go to new section
            break;
          }
        }
      },
      Err(error) => {
        return new_error(error);
      }
    }
  }

  new_dictionary(keys, values)
}
