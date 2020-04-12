## hrs3300-core 

A rust no_std driver for the 
Nanjing TianYiHeXin Electronics Company 
HRS300 heart rate sensor device. 
This driver was originally developed for the PineTime smart watch.

The available documentation for the HRS3300 is very limited, 
and the only known way to process the raw sensor data into
heart rate measurements is to use a closed-source library
provided by the vendor.  However, this crate will attempt
an open source implementation of the data smoothing and
processing to produce a heart rate measurement.

## Status
This is work-in-progress
- [ ] Debug build
- [ ] Release build
- [ ] Blocking mode read of raw sensor data
- [ ] Smoothing / filtering of sensor data
- [ ] Processing sensor data into heart rate measurement
- [ ] Example for running on PineTime hardware
- [ ] CI
- [ ] Documentation

