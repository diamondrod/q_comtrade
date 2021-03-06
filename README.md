# COMTARDE File Parser for kdb+

Power system writes out a lot of sample data in COMTRADE format. As these data are time-series data, kdb+ is suitable to analyze it once the data is loaded into database. This shared library provides q/kdb+ with an ability to parse three kinds of COMTRADE files, i.e., configuration file (`.cfg`), data file (`.dat`) and information file (`.inf`). Data can be loaded from both contents (string/bytes) and file path (symbol).

**Notes:**
- COMTRADE is using `<CR/LF>` as a delimiter. This means that the file format is Windows native.
- This library is implemented for COMTRADE version 1999. For specification, see [the document](http://smartgridcenter.tamu.edu/resume/pdf/comtrade91.pdf).

## Example

```q
q)// Load shared library
q)\l q/comtrade.q
q)// Load configuration file.
q)config: "\r\n" sv read0 `:files/sample_ascii.cfg;
q)// Deserialize config.
q)parsed_config: .comtrade.deserializeConfig config
q)parsed_config
station_name                     | `TestStation3
recording_device_id              | `
revision_year                    | 1999i
total_number_of_channels         | 63i
number_of_analog_channels        | 31i
number_of_status_channels        | 32i
analog_channel_index             | 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 ..
analog_channel_id                | `IA_G1`IB_G1`IC_G1`VA_G1`VB_G1`VC_G1`IA_G2..
analog_channel_phase             | `A`B`C`A`B`C`A`B`C`A`B`C`A`B`C`A`B`C`A`B`C..
circuit_component_being_monitored| `1`2`3`5`6`7`1`2`3`5`6`7`1`2`3`5`6`7`1`2`3..
channel_units                    | `A`A`A`V`V`V`A`A`A`V`V`V`A`A`A`V`V`V`A`A`A..
channel_multiplier               | 0.01983805 0.02008609 0.02013783 0.0494803..
channel_offset_adder             | 2.468354 8.588351 2.465757 -3.369059 8.144..
skew                             | 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 ..
minimum_value                    | -99999 -99999 -99999 -99999 -99999 -99999 ..
maximum_value                    | 99998 99998 99998 99998 99998 99998 99998 ..
primary_factor                   | 2500 2500 2500 6 6 6 2500 2500 2500 6 6 6 ..
secondary_factor                 | 5 5 5 0.1 0.1 0.1 5 5 5 0.1 0.1 0.1 5 5 5 ..
scaling_identifier               | "ppppppppppppppppppppppppppppppp"
status_channel_index             | 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 ..
status_channel_id                | `86_G1`87UA_G1`87UB_G1`87UC_G1`64S_G1`51_G..
status_channel_phase             | ````````````````````````````````
..
q)analog_items: 13 # 6 _ key parsed_config
q)analog_items
`analog_channel_index`analog_channel_id`analog_channel_phase`circuit_componen..
q)flip analog_items # parsed_config
analog_channel_index analog_channel_id analog_channel_phase circuit_component..
-----------------------------------------------------------------------------..
1                    IA_G1             A                    1                ..
2                    IB_G1             B                    2                ..
3                    IC_G1             C                    3                ..
4                    VA_G1             A                    5                ..
5                    VB_G1             B                    6                ..
6                    VC_G1             C                    7                ..
7                    IA_G2             A                    1                ..
8                    IB_G2             B                    2                ..
9                    IC_G2             C                    3                ..
10                   VA_G2             A                    5                ..
11                   VB_G2             B                    6                ..
12                   VC_G2             C                    7                ..
13                   IA_G3             A                    1                ..
14                   IB_G3             B                    2                ..
15                   IC_G3             C                    3                ..
16                   VA_G3             A                    5                ..
17                   VB_G3             B                    6                ..
18                   VC_G3             C                    7                ..
19                   IA_G4             A                    1                ..
20                   IB_G4             B                    2                ..
..
q)// Load data file
q)data: "\r\n" sv read0 `:files/sample_ascii.dat;
q)// Deserialize data.
q)parsed_data: .comtrade.deserializeData[data; parsed_config];
q)parsed_data
sample_number time                          analog_channel_0 analog_channel_1..
-----------------------------------------------------------------------------..
1             2007.06.25D19:13:57.789757000 93678            -71445          ..
2             2007.06.25D19:13:57.789931000 95165            -67662          ..
3             2007.06.25D19:13:57.790104000 95784            -63879          ..
4             2007.06.25D19:13:57.790278000 96652            -60097          ..
5             2007.06.25D19:13:57.790451000 96528            -54728          ..
6             2007.06.25D19:13:57.790625000 95660            -50335          ..
7             2007.06.25D19:13:57.790799000 95660            -46308          ..
8             2007.06.25D19:13:57.790972000 95041            -41061          ..
9             2007.06.25D19:13:57.791146000 93926            -36180          ..
10            2007.06.25D19:13:57.791320000 92934            -30445          ..
11            2007.06.25D19:13:57.791493000 91447            -25198          ..
12            2007.06.25D19:13:57.791667000 90580            -20439          ..
13            2007.06.25D19:13:57.791840000 89465            -15070          ..
14            2007.06.25D19:13:57.792014000 87730            -9823           ..
15            2007.06.25D19:13:57.792188000 85871            -4942           ..
16            2007.06.25D19:13:57.792361000 83021            -61             ..
17            2007.06.25D19:13:57.792535000 80171            4331            ..
18            2007.06.25D19:13:57.792708000 77693            8602            ..
19            2007.06.25D19:13:57.792882000 73976            14215           ..
20            2007.06.25D19:13:57.793056000 70010            18730           ..
..
q)// Load information (can parse independently)
q)info: "\r\n" sv read0 `:files/sample.inf
q)// Deserialize info.
q)parsed_info: .comtrade.deserializeInfo info;
q)parsed_info
Record_Information  | `Source`Record_Information`Location`max_current`min_cur..
Event_Information_#1| `Channel_number`max_value`min_value`max_sample_number`m..
Event_Information_#2| `Channel_number`max_value`min_value`max_sample_number`m..
File_Description    | `Station_Name`Recording_Device_ID`Revision_Year`Total_C..
Analog_Channel_#1   | `Channel_ID`Phase_ID`Monitored_Component`Channel_Units`..
Status_Channel_#1   | `Channel_ID`Phase_ID`Monitored_Component`Normal_State!(..
event_rec           | `recorder_type`trig_set`ch_type!(,,"1";(,"0";,"0";,"0";..
analog_rec_#1       | `op_limit`trg_over_val`trg_under_val`trg_roc`inverted!(..
```

or you can load data from file:

```q
q)// Deserialize config.
q)parsed_config: .comtrade.deserializeConfig `:files/sample_ascii_win.cfg
q)parsed_config
station_name                     | `TestStation3
recording_device_id              | `
revision_year                    | 1999i
total_number_of_channels         | 63i
number_of_analog_channels        | 31i
number_of_status_channels        | 32i
analog_channel_index             | 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 ..
analog_channel_id                | `IA_G1`IB_G1`IC_G1`VA_G1`VB_G1`VC_G1`IA_G2..
analog_channel_phase             | `A`B`C`A`B`C`A`B`C`A`B`C`A`B`C`A`B`C`A`B`C..
circuit_component_being_monitored| `1`2`3`5`6`7`1`2`3`5`6`7`1`2`3`5`6`7`1`2`3..
channel_units                    | `A`A`A`V`V`V`A`A`A`V`V`V`A`A`A`V`V`V`A`A`A..
channel_multiplier               | 0.01983805 0.02008609 0.02013783 0.0494803..
channel_offset_adder             | 2.468354 8.588351 2.465757 -3.369059 8.144..
skew                             | 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 ..
minimum_value                    | -99999 -99999 -99999 -99999 -99999 -99999 ..
maximum_value                    | 99998 99998 99998 99998 99998 99998 99998 ..
primary_factor                   | 2500 2500 2500 6 6 6 2500 2500 2500 6 6 6 ..
secondary_factor                 | 5 5 5 0.1 0.1 0.1 5 5 5 0.1 0.1 0.1 5 5 5 ..
scaling_identifier               | "ppppppppppppppppppppppppppppppp"
status_channel_index             | 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 ..
status_channel_id                | `86_G1`87UA_G1`87UB_G1`87UC_G1`64S_G1`51_G..
status_channel_phase             | ````````````````````````````````
..
q)// Deserialize data.
q)parsed_data: .comtrade.deserializeData[`:files/sample_ascii_win.dat; parsed_config];
q)parsed_data
sample_number time                          analog_channel_0 analog_channel_1..
-----------------------------------------------------------------------------..
1             2007.06.25D19:13:57.789757000 93678            -71445          ..
2             2007.06.25D19:13:57.789931000 95165            -67662          ..
3             2007.06.25D19:13:57.790104000 95784            -63879          ..
4             2007.06.25D19:13:57.790278000 96652            -60097          ..
5             2007.06.25D19:13:57.790451000 96528            -54728          ..
6             2007.06.25D19:13:57.790625000 95660            -50335          ..
7             2007.06.25D19:13:57.790799000 95660            -46308          ..
8             2007.06.25D19:13:57.790972000 95041            -41061          ..
9             2007.06.25D19:13:57.791146000 93926            -36180          ..
10            2007.06.25D19:13:57.791320000 92934            -30445          ..
11            2007.06.25D19:13:57.791493000 91447            -25198          ..
12            2007.06.25D19:13:57.791667000 90580            -20439          ..
13            2007.06.25D19:13:57.791840000 89465            -15070          ..
14            2007.06.25D19:13:57.792014000 87730            -9823           ..
15            2007.06.25D19:13:57.792188000 85871            -4942           ..
16            2007.06.25D19:13:57.792361000 83021            -61             ..
17            2007.06.25D19:13:57.792535000 80171            4331            ..
18            2007.06.25D19:13:57.792708000 77693            8602            ..
19            2007.06.25D19:13:57.792882000 73976            14215           ..
20            2007.06.25D19:13:57.793056000 70010            18730           ..
..
```

# Install

You can use `cargo` to build `libqcomtrade.so`.

```bash
comtrader]$ cargo build --release
```
