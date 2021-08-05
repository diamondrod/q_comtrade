/
* @file comtrade.q
* @overview Define q functions to deserialize a COMTRADE files.
\

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                     Initial Setting                   //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

\l tests/test_helper_function.q
\l q/comtrade.q

//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//
//                         Test                          //
//+++++++++++++++++++++++++++++++++++++++++++++++++++++++//

//%% Load Answers %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

result_ascii_config: get `:tests/result_ascii_config;
result_ascii_data: get `:tests/result_ascii_data;
result_info: get `:tests/result_info;

//%% Tests %%//vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv/

ascii_config: "\r\n" sv read0 `:tests/test_ascii.cfg;
parsed_ascii_config: .comtrade.deserializeConfig ascii_config;
.test.ASSERT_EQ["ASCII config"; parsed_ascii_config; result_ascii_config];

ascii_data: "\r\n" sv read0 `:tests/test_ascii.dat;
parsed_ascii_data: .comtrade.deserializeData[ascii_data; parsed_ascii_config];
.test.ASSERT_EQ["ASCII data"; parsed_ascii_data; result_ascii_data];

binary_config: "\r\n" sv read0 `:tests/test_binary.cfg;
parsed_binary_config: .comtrade.deserializeConfig binary_config;
.test.ASSERT_EQ["binary config"; parsed_binary_config; @[parsed_ascii_config; `file_type; :; `binary]];

binary_data: read1 `:tests/test_binary.dat;
parsed_binary_data: .comtrade.deserializeData[binary_data; parsed_binary_config];
.test.ASSERT_EQ["binary data"; ![parsed_binary_data; (); 0b; enlist `time]; ![parsed_ascii_data; (); 0b; enlist `time]];

info: "\r\n" sv read0 `:tests/test.inf
parsed_info: .comtrade.deserializeInfo info
.test.ASSERT_EQ["info"; parsed_info; result_info];

.test.DISPLAY_RESULT[];
