/
* @file read_from_file.q
* @overview Deserialize COMTRADE files from files. `.dat` file is encoded in ASCII format.
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

// Deserialize config.
parsed_config: .comtrade.deserializeConfig `:files/sample_ascii_win.cfg

// Deserialize data.
parsed_data: .comtrade.deserializeData[`:files/sample_ascii_win.dat; parsed_config];

// Deserialize info.
parsed_info: .comtrade.deserializeInfo `:files/sample_win.inf;
