//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                            Load Libraries                            //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

use kdbplus::*;
use kdbplus::api::*;
use chrono::prelude::*;
use super::{parse_token, KDB_TIMESTAMP_OFFSET, load_ascii_data};

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                          Private Fucntions                           //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize 1st component (line) of `.cfg` file.
/// Format: "station_name,rec_dev_id,rev_year".
fn deserialize_comtrade_config_1(lines: &Vec<&str>, cursor: usize) -> Result<(K, K, usize), &'static str>{
  let tokens=lines[cursor].split(',').collect::<Vec<&str>>();
  if tokens.len() != 3{
    Err("the number of fields is fewer than expected -  line1\0")
  }
  else{
    let keys = new_list(qtype::SYMBOL_LIST, 3);
    let keys_slice=keys.as_mut_slice::<S>();
    keys_slice[0]=internalize(str_to_S!("station_name"));
    keys_slice[1]=internalize(str_to_S!("recording_device_id"));
    keys_slice[2]=internalize(str_to_S!("revision_year"));
    let values=new_list(qtype::COMPOUND_LIST, 3);
    values.as_mut_slice::<K>().copy_from_slice(&[new_symbol(tokens[0]), new_symbol(tokens[1]), new_int(tokens[2].parse::<i32>().unwrap_or_else(|_| 1991))]);
    Ok((keys, values, cursor + 1))
  }
}

/// Deserialize 2nd component (line) of `.cfg` file.
/// Format: "TT,##A,##D".
fn deserialize_comtrade_config_2(lines: &Vec<&str>, cursor: usize) -> Result<(K, K, i32, i32, usize), &'static str>{
  let keys = new_list(qtype::SYMBOL_LIST, 3);
  let keys_slice=keys.as_mut_slice::<S>();
  keys_slice[0]=internalize(str_to_S!("total_number_of_channels"));
  keys_slice[1]=internalize(str_to_S!("number_of_analog_channels"));
  keys_slice[2]=internalize(str_to_S!("number_of_status_channels"));

  let tokens;
  if let Some(line) = lines.get(cursor){
    tokens=line.split(',').collect::<Vec<&str>>();
  }
  else{
    return Err("early EOF\0");
  }
  if tokens.len() != 3{
    decrement_reference_count(keys);
    Err("the number of fields is fewer than expected -  line2\0")
  }
  else{
    let values=new_list(qtype::COMPOUND_LIST, 3);
    let values_slice=values.as_mut_slice::<K>();
    let num_analog_channel;
    let num_status_channel;
    match tokens[0].parse::<i32>(){
      Ok(num) => values_slice[0]=new_int(num),
      Err(_) => {
        decrement_reference_count(keys);
        decrement_reference_count(values);
        return Err("invalid total number of channels\0")
      }
    }
    // Trim last 'A'
    match tokens[1][..tokens[1].len()-1].parse::<i32>(){
      Ok(num) => {
        num_analog_channel=num;
        values_slice[1]=new_int(num)
      },
      Err(_) => {
        decrement_reference_count(keys);
        decrement_reference_count(values);
        return Err("invalid number of analog channels\0")
      }
    }
    // Trim last 'D'
    match tokens[2][..tokens[2].len()-1].parse::<i32>(){
      Ok(num) => {
        num_status_channel=num;
        values_slice[2]=new_int(num)
      },
      Err(_) => {
        decrement_reference_count(keys);
        decrement_reference_count(values);
        return Err("invalid number of status channels\0")
      }
    }
    Ok((keys, values, num_analog_channel, num_status_channel, cursor+1))
  }
}

/// Deserialize each line of 3rd component of `.cfg` file.
/// Format: "An,ch_id,ph,ccbm,uu,a,b,skew,min,max,primary,secondary,PS".
fn deserialize_comtrade_config_3_inner(line: &str, values: K) -> Result<(), &'static str>{
  let tokens=line.split(',').collect::<Vec<&str>>();
  if tokens.len() != 13{
    Err("the number of fields is fewer than expected -  component 3\0")
  }
  else{
    let values_slice=values.as_mut_slice::<K>();
    
    parse_token![i32; tokens, 0, values_slice, 0, "invalid analog channel index\0"];

    for i in 1..5{
      values_slice[i].push_symbol(tokens[i]).unwrap();
    }

    parse_token![f64; tokens, 5, values_slice, 5, "invalid channel multiplier\0"];
    parse_token![f64; tokens, 6, values_slice, 6, "invalid channel offset adder\0"];
    parse_token![f64; tokens, 7, values_slice, 7, "invalid channel skew\0"];
    parse_token![i32; tokens, 8, values_slice, 8, "invalid minimum value\0"];
    parse_token![i32; tokens, 9, values_slice, 9, "invalid maximum value\0"];
    parse_token![f64; tokens, 10, values_slice, 10, "invalid primary factor\0"];
    parse_token![f64; tokens, 11, values_slice, 11, "invalid secondary factor\0"];
    
    match tokens[12]{
      "p" | "P" => {
        values_slice[12].push_raw('p').unwrap();
      },
      "s" | "S" => {
        values_slice[12].push_raw('s').unwrap();
      },
      _ => return Err("invalid data scaling identifier\0")
    }
    
    Ok(())
  }
}

/// Deserialize 3rd line of `.cfg` file.
fn deserialize_comtrade_config_3(lines: &Vec<&str>, num_analog_channel: i32, cursor: usize) -> Result<(K, K, usize), &'static str>{
  
  if lines.len() < cursor + num_analog_channel as usize{
    // There are fewer lines than expected
    Err("early EOF\0")
  }
  else{
    let keys = new_list(qtype::SYMBOL_LIST, 13);
    let keys_slice=keys.as_mut_slice::<S>();
    keys_slice[0]=internalize(str_to_S!("analog_channel_index"));
    keys_slice[1]=internalize(str_to_S!("analog_channel_id"));
    keys_slice[2]=internalize(str_to_S!("analog_channel_phase"));
    keys_slice[3]=internalize(str_to_S!("circuit_component_being_monitored"));
    keys_slice[4]=internalize(str_to_S!("channel_units"));
    keys_slice[5]=internalize(str_to_S!("channel_multiplier"));
    keys_slice[6]=internalize(str_to_S!("channel_offset_adder"));
    keys_slice[7]=internalize(str_to_S!("skew"));
    keys_slice[8]=internalize(str_to_S!("minimum_value"));
    keys_slice[9]=internalize(str_to_S!("maximum_value"));
    keys_slice[10]=internalize(str_to_S!("primary_factor"));
    keys_slice[11]=internalize(str_to_S!("secondary_factor"));
    keys_slice[12]=internalize(str_to_S!("scaling_identifier"));

    let values = new_list(qtype::COMPOUND_LIST, 13);
    values.as_mut_slice::<K>().copy_from_slice(&[
      new_list(qtype::INT_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::REAL_LIST, 0_i64),
      new_list(qtype::REAL_LIST, 0_i64),
      new_list(qtype::REAL_LIST, 0_i64),
      new_list(qtype::INT_LIST, 0_i64),
      new_list(qtype::INT_LIST, 0_i64),
      new_list(qtype::REAL_LIST, 0_i64),
      new_list(qtype::REAL_LIST, 0_i64),
      // Use push
      new_string("")
    ]);

    let result=lines[cursor..(cursor+num_analog_channel as usize)].iter().map(|line|
      // Deserialize each line and append new values to corresponding lists.
      deserialize_comtrade_config_3_inner(*line, values)
    ).collect::<Result<Vec<_>, &'static str>>();
    
    match result{
      Ok(_) => Ok((keys, values, cursor+num_analog_channel as usize)),
      Err(error) => {
        decrement_reference_count(keys);
        decrement_reference_count(values);
        Err(error)
      }
    }
  }
  
}

/// Deserialize each line of 4th component of `.cfg` file.
/// Format: "Dn,ch_id,ph,ccbm,y".
fn deserialize_comtrade_config_4_inner(line: &str, values: K) -> Result<(), &'static str>{
  let tokens=line.split(',').collect::<Vec<&str>>();
  if tokens.len() != 5{
    Err("the number of fields is fewer than expected -  component 4\0")
  }
  else{
    let values_slice=values.as_mut_slice::<K>();

    parse_token![i32; tokens, 0, values_slice, 0, "invalid analog channel index\0"];

    for i in 1..4{
      values_slice[i].push_symbol(tokens[i]).unwrap();
    }

    match tokens[4]{
      "0" => {
        values_slice[4].push_raw(0_u8).unwrap();
      },
      "1" => {
        values_slice[4].push_raw(1_u8).unwrap();
      },
      _ => return Err("invalid channel state\0")
    }

    Ok(())
  }
}

/// Deserialize 4th line of `.cfg` file.
fn deserialize_comtrade_config_4(lines: &Vec<&str>, num_status_channel: i32, cursor: usize) -> Result<(K, K, usize), &'static str>{
  
  if lines.len() < cursor+num_status_channel as usize{
    // There are fewer lines than expected
    Err("early EOF\0")
  }
  else{
    let keys = new_list(qtype::SYMBOL_LIST, 5);
    let keys_slice=keys.as_mut_slice::<S>();
    keys_slice[0]=internalize(str_to_S!("status_channel_index"));
    keys_slice[1]=internalize(str_to_S!("status_channel_id"));
    keys_slice[2]=internalize(str_to_S!("status_channel_phase"));
    keys_slice[3]=internalize(str_to_S!("circuit_component_being_monitored"));
    keys_slice[4]=internalize(str_to_S!("channel_state"));

    let values = new_list(qtype::COMPOUND_LIST, 5);
    values.as_mut_slice::<K>().copy_from_slice(&[
      new_list(qtype::INT_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::SYMBOL_LIST, 0_i64),
      new_list(qtype::BOOL_LIST, 0_i64)
    ]);

    let result=lines[cursor..cursor+num_status_channel as usize].iter().map(|line|
      // Deserialize each line and append new values to corresponding lists.
      deserialize_comtrade_config_4_inner(*line, values)
    ).collect::<Result<Vec<_>, &'static str>>();
    
    match result{
      Ok(_) => Ok((keys, values, cursor + num_status_channel as usize)),
      Err(error) => {
        decrement_reference_count(keys);
        decrement_reference_count(values);
        Err(error)
      }
    }
  }
}

/// Deserialize 5th component (line) of `.cfg` file.
/// Format: "lf".
fn deserialize_comtrade_config_5(lines: &Vec<&str>, mut keys: K, mut values: K, cursor: usize) -> Result<usize, &'static str>{
  if let Some(line) = lines.get(cursor){
    keys.push_symbol("line_frequency").unwrap();
    values.push(new_real(line.parse::<f64>().unwrap_or_else(|_| qnull_base::F))).unwrap();
    Ok(cursor + 1)
  }
  else{
    Err("early EOF\0")
  }
}

/// Deserialize each line of sample rate in the 6th component of `.cfg` file.
/// "samp,endsamp"
fn deserialize_comtrade_config_6_inner(line: &str, values: K) -> Result<(), &'static str>{
  let tokens=line.split(',').collect::<Vec<&str>>();
  if tokens.len() != 2{
    Err("the number of fields is fewer than expected -  component 6\0")
  }
  else{
    let values_slice=values.as_mut_slice::<K>();
    parse_token![f64; tokens, 0, values_slice, 1, "invalid sample rate\0"];
    parse_token![i32; tokens, 1, values_slice, 2, "invalid last sample number\0"];
    Ok(())
  }
}

/// Deserialize 6th component of `.cfg` file.
/// "nrates"
/// "samp,endsamp"
/// ...
/// "samp,endsamp"
fn deserialize_comtrade_config_6(lines: &Vec<&str>, cursor: usize) -> Result<(K, K, usize), &'static str>{
  match lines.get(cursor){
    Some(line) => {
      match line.parse::<i32>(){
        Ok(num) if lines.len() >= cursor + 1 + num as usize=> {
          // Line of sample rate exits even if num is 0.
          // Length check is done further in the code below.

          let keys = new_list(qtype::SYMBOL_LIST, 3);
          let keys_slice = keys.as_mut_slice::<S>();
          keys_slice[0]=internalize(str_to_S!("number_of_sample_rates"));
          keys_slice[1]=internalize(str_to_S!("sample_rates"));
          keys_slice[2]=internalize(str_to_S!("last_sample_number"));

          let values=new_list(qtype::COMPOUND_LIST, 3);
          values.as_mut_slice::<K>().copy_from_slice(&[
            new_int(num),
            new_list(qtype::REAL_LIST, 0),
            new_list(qtype::INT_LIST, 0)
          ]);

          if num == 0 {
            if let Some(line) = lines.get(cursor+1){
              match deserialize_comtrade_config_6_inner(*line, values){
                Ok(_) => Ok((keys, values, cursor + 1 + 1)),
                Err(error) => Err(error)
              }
            }
            else{
              Err("early EOF\0")
            }
          }
          else{
            let result=lines[cursor+1..cursor+1+num as usize].iter().map(|line|{
              deserialize_comtrade_config_6_inner(*line, values)
            }).collect::<Result<Vec<_>, &'static str>>();

            match result{
              Ok(_) => Ok((keys, values, cursor + 1 + num as usize)),
              Err(error) => Err(error)
            }
          }
        },
        Ok(_) => Err("early EOF\0"),
        _ => Err("invalid number of sample rates\0")
      }
    },
    None => Err("early EOF\0")
  }
}

/// Deserialize each line of 7th component of `.cfg` file.
/// "dd/mm/yyyy,hh:mm:ss.ssssss"
fn deserialize_comtrade_config_7_inner(line: &str, error: &'static str) -> Result<i64, &'static str>{
  if line.len() == 26{
    match (line[0..2].parse::<u32>(), line[3..5].parse::<u32>(), line[6..10].parse::<i32>(), line[11..13].parse::<u32>(), line[14..16].parse::<u32>(), line[17..19].parse::<u32>(), line[20..26].parse::<u32>()){
      (Ok(day), Ok(month), Ok(year), Ok(hour), Ok(minute), Ok(second), Ok(micros)) => {
        Ok(Utc.ymd(year, month, day).and_hms_micro(hour, minute, second, micros).timestamp_nanos() - KDB_TIMESTAMP_OFFSET)
      },
      _ => Err(error)
    }
  }
  else{
    // Field is non-critical. Fill as null.
    Ok(qnull_base::J)
  }
  
}

/// Deserialize 7th component of `.cfg` file.
/// "dd/mm/yyyy,hh:mm:ss.ssssss"
/// "dd/mm/yyyy,hh:mm:ss.ssssss"
fn deserialize_comtrade_config_7(lines: &Vec<&str>, mut keys: K, mut values: K, cursor: usize) -> Result<usize, &'static str>{
  if lines.len() < cursor + 2{
    // There are fewer lines than expected
    Err("early EOF\0")
  }
  else{
    keys.push_symbol("first_data_time").unwrap();
    keys.push_symbol("event_time").unwrap();
    values.push(new_timestamp(deserialize_comtrade_config_7_inner(lines[cursor], "invalid first data time\0")?)).unwrap();
    values.push(new_timestamp(deserialize_comtrade_config_7_inner(lines[cursor+1], "invalid event time\0")?)).unwrap();
    Ok(cursor+2)
  }
}

/// Deserialize 8th component (line) of `.cfg` file.
/// Format: "ft".
fn deserialize_comtrade_config_8(lines: &Vec<&str>, cursor: usize) -> Result<(&'static str, usize), &'static str>{
  if let Some(line) = lines.get(cursor){
    match *line{
      "ASCII" | "ascii" => {
        Ok(("ascii", cursor+1))
      },
      "BINARY" | "binary" => {
        Ok(("binary", cursor+1))
      },
      _ => Err("invalid file type\0")
    }
  }
  else{
    Err("early EOF\0")
  }
}

/// Deserialize 8th component (line) of `.cfg` file.
/// Format: "ft".
fn deserialize_comtrade_config_9(lines: &Vec<&str>, mut keys: K, mut values: K, cursor: usize) -> Result<usize, &'static str>{
  if let Some(line) = lines.get(cursor){
    match line.parse::<f64>(){
      Ok(num) => {
        keys.push_symbol("timestamp_multiplication_factor").unwrap();
        values.push(new_real(num)).unwrap();
        Ok(cursor + 1)
      },
      Err(_) => Err("invalid timestamp multiplication factor\0")
    }
  }
  else{
    Err("early EOF\0")
  }
}

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                               Interface                              //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize the configuration file (`.cfg`) of COMTRADE format into q dictionary.
/// # Parameters
/// - `data`: 
///   - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
///   - string: File contents.
#[no_mangle]
pub extern "C" fn deserialize_comtrade_config(data: K) -> K{

  let string;
  let mut contents_buffer = String::new();
  // Load data into string.
  set_string!(data, string, contents_buffer);

  let lines=string.split_terminator("\r\n").collect::<Vec<&str>>();
  let mut keys;
  let mut values;
  let mut cursor = 0;

  match deserialize_comtrade_config_1(&lines, cursor){
    Ok((keys_, values_, cursor_)) => {
      cursor= cursor_;
      keys = keys_;
      values = values_;
    },
    Err(error) => return new_error(error)
  }

  let num_analog_channel;
  let num_status_channel;
  match deserialize_comtrade_config_2(&lines, cursor){
    Ok((keys_, values_, num_analog_channel_, num_status_channel_, cursor_)) => {
      cursor=cursor_;
      num_analog_channel=num_analog_channel_;
      num_status_channel=num_status_channel_;
      keys.append(keys_).unwrap();
      values.append(values_).unwrap();
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_3(&lines, num_analog_channel, cursor){
    Ok((keys_, values_, cursor_)) => {
      cursor=cursor_;
      keys.append(keys_).unwrap();
      values.append(values_).unwrap();
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_4(&lines, num_status_channel, cursor){
    Ok((keys_, values_, cursor_)) => {
      cursor=cursor_;
      keys.append(keys_).unwrap();
      values.append(values_).unwrap();
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_5(&lines, keys, values, cursor){
    Ok(cursor_) => cursor=cursor_,
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_6(&lines, cursor){
    Ok((keys_, values_, cursor_)) => {
      cursor=cursor_;
      keys.append(keys_).unwrap();
      values.append(values_).unwrap();
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_7(&lines, keys, values, cursor){
    Ok(cursor_) => {
      cursor=cursor_;
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_8(&lines, cursor){
    Ok((value, cursor_)) => {
      // Not sure why appending symbol inside the function leads to values leads to crash...
      keys.push_symbol("file_type").unwrap();
      values.push(new_symbol(value)).unwrap();
      cursor=cursor_;
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  match deserialize_comtrade_config_9(&lines, keys, values, cursor){
    Ok(cursor_) => {
      cursor=cursor_;
    },
    Err(error) => {
      decrement_reference_count(keys);
      decrement_reference_count(values);
      return new_error(error)
    }
  }

  if cursor != lines.len(){
    new_error("redundant line?\0")
  }
  else{
    new_dictionary(keys, values)
  }   
}
