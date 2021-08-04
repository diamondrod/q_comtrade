/
* @file read_binary.q
* @overview Deserialize COMTRADE files from their contents. `.dat` file is encoded in binary format.
\

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                     Initial Setting                   //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

\c 50 500

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                     Load Library                      //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

// Load transformer library
\l q/comtrade.q

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                    Deserialization                    //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

// Load configuration file.
config: "\r\n" sv read0 `:files/sample_binary.cfg;
// Deserialize config.
parsed_config: .comtrade.deserializeConfig config

// Load data file
data: read1 `:files/sample_binary.dat;
// Deserialize data.
parsed_data: .comtrade.deserializeData[data; parsed_config];

// Load information (can parse independently)
info: "\r\n" sv read0 `:files/sample.inf
// Deserialize info.
parsed_info: .comtrade.deserializeInfo info;
