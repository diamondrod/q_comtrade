//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                            Load Libraries                            //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

use std::convert::TryInto;
use bitvec::prelude::*;
use kdbplus::*;
use kdbplus::api::*;
use super::{parse_token, load_ascii_data, load_binary_data};

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                          Private Fucntions                           //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize each line of data (.dat) file written in ASCII format.
/// "n, timestamp, A1, A2,···Ak, D1, D2,···Dm"
fn deserialize_comtrade_data_inner_ascii(line: &str, values: K, num_analog_channel: i32, num_status_channel: i32, critical_timestamp: bool, first_data_time: i64, timestamp_multiplication_factor: f64) -> Result<(), &'static str>{
  let tokens=line.split(',').collect::<Vec<&str>>();
  if tokens.len() != num_analog_channel as usize + num_status_channel  as usize + 2{
    Err("the number of fields is fewer than expected\0")
  }
  else{
    let values_slice=values.as_mut_slice::<K>();

    // Deserialize sample number
    parse_token![i32; tokens, 0, values_slice, 0, "invalid sample number\0"];

    // Deserialize timestamp
    match tokens[1].parse::<i64>(){
      Ok(micros) => {
        values_slice[1].push_raw((timestamp_multiplication_factor * (1000 * micros) as f64) as i64 + first_data_time).unwrap();
      },
      Err(_) => {
        if critical_timestamp{
          return Err("invalid timestamp\0")
        }
        else{
          values_slice[1].push_raw(qnull_base::J).unwrap();
        }
      }
    }

    // Deserilize analog data
    for i in 2..2+num_analog_channel as usize{
      match tokens[i].parse::<i32>(){
        Ok(num) if num == 99999 => {
          values_slice[i].push_raw(qnull_base::I).unwrap();
        },
        Ok(num) => {
          values_slice[i].push_raw(num).unwrap();
        },
        Err(_) => return Err("invalid analog channel data\0")
      }
    }

    // Deserilize status data
    for i in 2+num_analog_channel as usize..tokens.len(){
      match tokens[i].parse::<i32>(){
        Ok(0) => {
          values_slice[i].push_raw(0).unwrap();
        },
        Ok(1) => {
          values_slice[i].push_raw(1).unwrap();
        },
        _ => return Err("invalid status channel data\0")
      }
    }

    Ok(())
  }
}

/// Deserialize each record of data (.dat) file written in a binary format.
/// sample number (4 bytes) + timestamp (4 bytes) + analog data (2 bytes) * num_ananalog + status data (2 * INT(num_status / 16 bits))
fn deserialize_comtrade_data_inner_binary(chunk: &[u8], mut cursor: usize, values: K, num_analog_channel: i32, num_status_channel: i32, critical_timestamp: bool, first_data_time: i64, timestamp_multiplication_factor: f64) -> Result<usize, &'static str>{
  
  if (chunk.len() % (4 * 2 + 2 * num_analog_channel as usize + 2 * (num_status_channel as f64 / 16_f64).ceil() as usize)) != 0{
    // Total length of bytes is not a multiple of single line length
    return Err("the number of fields is fewer than expected\0");
  }
  
  let values_slice=values.as_mut_slice::<K>();
  
  // Deserialize sample number
  values_slice[0].push_raw(i32::from_le_bytes(chunk[cursor..cursor+4].try_into().unwrap())).unwrap();
  cursor+=4;

  // Deserialize timestamp
  if chunk[cursor..cursor+4] == [0xFF_u8; 4]{
    if critical_timestamp{
      return Err("invalid timestamp\0")
    }
    else{
      values_slice[1].push_raw(qnull_base::J).unwrap();
    }
  }
  else{
    let num = i32::from_le_bytes(chunk[cursor..cursor+4].try_into().unwrap());
    values_slice[1].push_raw(first_data_time + ((1000 * num as i64) as f64 * timestamp_multiplication_factor) as i64).unwrap();
  }
  cursor+=4;

  // Deserilize analog data
  chunk[cursor..cursor+2*num_analog_channel as usize].chunks(2).enumerate().for_each(|(idx, data)|{
    if data == &[0x00_u8, 0x80]{
      values_slice[2+idx].push_raw(qnull_base::I).unwrap();
    }
    else{
      let num = i16::from_le_bytes(data.try_into().unwrap());
      values_slice[2+idx].push_raw(num as i32).unwrap();
    }
    cursor+=2;
  });

  // final block may not be complete 16 bits but padded.
  let complete_chunk_size = num_status_channel / 16;
  chunk[cursor..cursor + 2*complete_chunk_size as usize].chunks(2).enumerate().for_each(|(idx, data)|{
    // 16 channel data are stored in 2 bytes in Little Endian
    // 8th - 1st | 16th - 9th
    let offset = 16 * idx;
    let view = data.view_bits::<Msb0>();
    for i in 2 .. 10{
      // Higher bits
      // ex.) values_slice[2+num_analog_channel as usize + offset + 0].push_raw(view[7]).unwrap();
      // values_slice[2+num_analog_channel as usize + offset + 7].push_raw(view[0]).unwrap();
      values_slice[i + num_analog_channel as usize + offset].push_raw(view[9-i]).unwrap();
    }
    for i in 10..18{
      // Lower bits
      // ex.) values_slice[2+num_analog_channel as usize + offset + 8].push_raw(view[15]).unwrap();
      // values_slice[2+num_analog_channel as usize + offset + 15].push_raw(view[8]).unwrap();
      values_slice[i + num_analog_channel as usize + offset].push_raw(view[25-i]).unwrap();
    }
    cursor+=2;
  });
  
  if num_status_channel % 16 != 0{
    // Extra chunk exist
    let extra_chunk_size = num_status_channel as usize % 16;
    let view = chunk[cursor .. cursor+2].view_bits::<Msb0>();
    let mut column_offset = 2 + num_analog_channel as usize + 16 * complete_chunk_size as usize;
    if extra_chunk_size > 8{
      // 8th - 1st | 16th - 9th
      for i in 0..8{
        // Higher bits
        values_slice[column_offset+i].push_raw(view[7-i]).unwrap();
      }
      column_offset+=8;
      for i in 0 .. extra_chunk_size - 8{
        // Lower bits
        values_slice[column_offset+i].push_raw(view[15-i]).unwrap();
      }
    }
    else{
      // 8th - 1st | 16th - 9th
      for i in 0..extra_chunk_size{
        // Lower bits
        values_slice[column_offset+i].push_raw(view[7-i]).unwrap();
      }
    }
    cursor+=2;
  }
  

  Ok(cursor)

}

//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                               Interface                              //
//++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/// Deserialize the data file (`.dat`) of COMTRADE format into q table.
/// # Parameters
/// - `data`: 
///   - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
///   - string: File contents.
///   - list of byte: File contents
/// - `num_analog_channel_`: The number of analog channels.
/// - `num_status_channel_`: The number of status channels.
/// - `critical_timestamp_`: Flag of whether timestamp is critical or not.
/// - `first_data_time_`: Timestamp of the first data.
/// - `timestamp_multiplication_factor_`: Multiplication factor for timestamp in each record. Timestamp of each record is
///  `first_data_time` + timestamp * `timestamp_multiplication_factor_`.
/// - `is_ascii`: Flag of whether data is encoded in ASCI or binary.
#[no_mangle]
pub extern "C" fn deserialize_comtrade_data(data: K, num_analog_channel_: K, num_status_channel_: K, critical_timestamp_: K, first_data_time_: K, timestamp_multiplication_factor_: K, is_ascii_: K) -> K{

  let num_analog_channel = num_analog_channel_.get_int().unwrap();
  let num_status_channel = num_status_channel_.get_int().unwrap();
  let critical_timestamp = critical_timestamp_.get_bool().unwrap();
  let first_data_time = first_data_time_.get_long().unwrap();
  let timestamp_multiplication_factor = timestamp_multiplication_factor_.get_real().unwrap() as f64;
  let is_ascii = is_ascii_.get_bool().unwrap();

  // Prepare keys
  let keys=new_list(qtype::SYMBOL_LIST, (2 + num_analog_channel + num_status_channel) as J);
  let keys_slice=keys.as_mut_slice::<S>();
  keys_slice[0]=internalize(str_to_S!("sample_number"));
  keys_slice[1]=internalize(str_to_S!("time"));
  for i in 2..2+num_analog_channel as usize{
    keys_slice[i]=internalize(str_to_S!(format!("analog_channel_{}", i-2).as_str()));
  }
  for i in (2+num_analog_channel) as usize..(2+num_analog_channel+num_status_channel) as usize{
    keys_slice[i]=internalize(str_to_S!(format!("status_channel_{}", i-2-num_analog_channel as usize).as_str()));
  }

  // Prepare values
  let mut values=new_list(qtype::COMPOUND_LIST, 2);
  values.as_mut_slice::<K>().copy_from_slice(&[new_list(qtype::INT_LIST, 0), new_list(qtype::TIMESTAMP_LIST, 0)]);
  for _ in 0..num_analog_channel as usize{
    values.push(new_list(qtype::INT_LIST, 0)).unwrap();
  }
  for _ in 0..num_status_channel as usize{
    values.push(new_list(qtype::BOOL_LIST, 0)).unwrap();
  }

  if is_ascii{
    // Process ASCII data
    let string;
    let mut contents_buffer = String::new();
    if data.get_type() == qtype::SYMBOL_ATOM{
      if let Err(error) = load_ascii_data(data, &mut contents_buffer){
        return new_error(error);
      }
      else{
        string=contents_buffer.as_str();
      }
    }
    else{
      match data.get_str(){
        Ok(string_) => {
          string = string_;
        },
        Err(error) => {
          decrement_reference_count(keys);
          decrement_reference_count(values);
          return new_error(error)
        }
      }
    }

    let lines=string.split_terminator("\r\n").collect::<Vec<&str>>();
    // Deserilize each line in the data
    for i in 0..lines.len(){
      if let Err(error) = deserialize_comtrade_data_inner_ascii(lines[i], values, num_analog_channel, num_status_channel, critical_timestamp, first_data_time, timestamp_multiplication_factor){
        decrement_reference_count(keys);
        decrement_reference_count(values);
        return new_error(error);
      }
    }
    flip(new_dictionary(keys, values))
  }
  else{
    // Process binary data
    let bytes;
    let mut contents_buffer = Vec::new();
    // Load data into bytes.
    set_bytes!(data, bytes, contents_buffer);

    let total=bytes.len();
    let mut cursor=0;
    loop{
      match deserialize_comtrade_data_inner_binary(bytes, cursor, values, num_analog_channel, num_status_channel, critical_timestamp, first_data_time, timestamp_multiplication_factor){
        Ok(cursor_) => {
          if cursor_ < total{
            cursor = cursor_;
          }
          else{
            break;
          }
        },
        Err(error) => {
          decrement_reference_count(keys);
          decrement_reference_count(values);
          return new_error(error);
        }
      }
    }

    flip(new_dictionary(keys, values))
  }

}

