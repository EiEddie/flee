use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::time::Duration;
use std::io::{self, Read};

pub fn read_from_serial(port_name: &str, baud_rate: u32) -> Result<String, String> {
    // 配置串口设置
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_secs(1)) // 设置超时为1秒
        .data_bits(DataBits::Eight)      // 数据位8位
        .parity(Parity::None)            // 无校验位
        .stop_bits(StopBits::One)        // 停止位1
        .flow_control(FlowControl::None) // 无流控
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    let mut serial_buf: Vec<u8> = vec![0; 1024]; // 用于存放读取的串口数据
    match port.read(serial_buf.as_mut_slice()) {
        Ok(bytes_read) => {
            let data = String::from_utf8_lossy(&serial_buf[..bytes_read]);
            Ok(data.to_string()) // 返回读取到的数据
        },
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => Err("Timeout reading from serial port".to_string()),
        Err(e) => Err(format!("Failed to read from serial port: {}", e)),
    }
}

// 解析数据 "0012ppm 25^c" => (12, 25)，烟雾浓度和温度之间中间只有一个空格
pub fn parse_data(data: &str) -> Result<(i32, i32), String> {
    let parts: Vec<&str> = data.split_whitespace().collect();

    if parts.len() != 2 {
        return Err("Invalid data format".to_string());
    }

    // 解析烟雾浓度，例如 "0012ppm"
    let smoke_str = parts[0].trim_end_matches("ppm");
    let smoke_value = smoke_str.parse::<i32>()
        .map_err(|_| "Failed to parse smoke value".to_string())?;

    // 解析温度，例如 "25^c"
    let temp_str = parts[1].trim_end_matches("^c");
    let temp_value = temp_str.parse::<i32>()
        .map_err(|_| "Failed to parse temperature value".to_string())?;

    Ok((smoke_value, temp_value))
}
