/
* @file comtrade.q
* @overview Define q functions to deserialize a COMTRADE files.
\

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                     Initial Setting                   //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

LIBPATH_: $[
  `libqcomtrade.so in key `:target/debug; `:target/debug/libqcomtrade;
  `libqcomtrade.so in key `:target/release; `:target/release/libqcomtrade;
  `libqcomtrade
 ] 2:;

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                     Private Functions                 //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/
* @brief Deserialize the data file (`.dat`) of COMTRADE format into q table.
* @param data {variable}: 
*  - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
*  - string: File contents.
*  - list of byte: File contents
* @param num_analog_channel_ {int}: The number of analog channels.
* @param num_status_channel_ {int}: The number of status channels.
* @param critical_timestamp_ {bool}: Flag of whether timestamp is critical or not.
* @param first_data_time_ {timestamp}: Timestamp of the first data.
* @param timestamp_multiplication_factor_ {real}: Multiplication factor for timestamp in each record. Timestamp of each record is
*  `first_data_time` + timestamp * `timestamp_multiplication_factor_`.
* @param is_ascii {bool}: Flag of whether data is encoded in ASCI or binary.
\
.comtrade.deserializeData_imple: LIBPATH_ (`deserialize_comtrade_data; 7);

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                       Interface                       //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

/
* @brief Deserialize the configuration file (`.cfg`) of COMTRADE format into q dictionary.
* @param data {variable}: 
*  - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
*  - string: File contents.
\
.comtrade.deserializeConfig: LIBPATH_ (`deserialize_comtrade_config; 1);

/
* @brief Deserialize the data file (`.dat`) of COMTRADE format into q table.
* @param data {variable}: 
*  - symbol: File path which starts with `:`.
*  - string: File contents.
*  - list of byte: File contents
* @param config {dictionay}: Deserialized configuration with `.comtrade.deserializeConfig`.
\
.comtrade.deserializeData: {[data;config]
  .comtrade.deserializeData_imple[data;
    config `number_of_analog_channels;
    config `number_of_status_channels;
    $[config `number_of_sample_rates;
      0b;
      first config `sample_rates;
      0b;
      // Both number_of_sample_rates and sample_rates are 0
      1b
    ];
    config `first_data_time;
    config `timestamp_multiplication_factor;
    `ascii ~ config `file_type
  ]
 };

/
* @brief Deserialize the information file (`.inf`) of COMTRADE format into q dictionary.
* @param data {variable}: 
*  - symbol: File path which starts with `:`. Thsi file must be delimited <CR/LF>. i.e., the file must be in the Windows format.
*  - string: File contents.
\
.comtrade.deserializeInfo: LIBPATH_ (`deserialize_comtrade_info; 1);
