//! Commands
#![allow(unused)]

/// Command Software Reset
pub const SOFTWARE_RESET: u8 = 0x01;

/// Command Memory Access Control
pub const MEMORY_ACCESS_CONTROL: u8 = 0x36;

/// Command for Pixel Format Set
pub const PIXEL_FORMAT_SET: u8 = 0x3a;

/// Command for Sleep Mode On
pub const SLEEP_MODE_ON: u8 = 0x10;

/// Command for Sleep Mode Off
pub const SLEEP_MODE_OFF: u8 = 0x11;

/// Command for Invert Off
pub const INVERT_OFF: u8 = 0x20;

/// Command for Invert On
pub const INVERT_ON: u8 = 0x21;

/// Command for Display Off
pub const DISPLAY_OFF: u8 = 0x28;

/// Command for Display On
pub const DISPLAY_ON: u8 = 0x29;

/// Command Column Address Set
pub const COLUMN_ADDRESS_SET: u8 = 0x2a;

/// Command for Page Address Set
pub const PAGE_ADDRESS_SET: u8 = 0x2b;

/// Command for MemoryWrite
pub const MEMORY_WRITE: u8 = 0x2c;

/// Command for Vertical Scroll Define
pub const VERTICAL_SCROLL_DEFINE: u8 = 0x33;

/// Command for Vertical Scroll Address
pub const VERTICAL_SCROLL_ADDR: u8 = 0x37;

/// Command for Idle Mode Off
pub const IDLE_MODE_OFF: u8 = 0x38;

/// Command for Idle Mode On
pub const IDLE_MODE_ON: u8 = 0x39;

/// Command for Set Brightness
pub const SET_BRIGHTNESS: u8 = 0x51;

/// Command for Content Adaptive Brightness
pub const CONTENT_ADAPTIVE_BRIGHTNESS: u8 = 0x55;

/// Command for Normal Mode Frame Rate
pub const NORMAL_MODE_FRAME_RATE: u8 = 0xb1;

/// Command for Idle Mode Frame rate
pub const IDLE_MODE_FRAME_RATE: u8 = 0xb2;

/// Command for Power A
pub const POWER_A: u8 = 0xcf;

/// Command for Power B
pub const POWER_B: u8 = 0xcf;

/// Command for Power Sequence
pub const POWER_SEQ: u8 = 0xed;

/// Command for Driver Timing Control A
pub const DRIVER_TIMING_CONTROL_A: u8 = 0xe8;

/// Command for Driver Timing Control B
pub const DRIVER_TIMING_CONTROL_B: u8 = 0xea;

/// Command for Pump Ratio Control
pub const PUMP_RATIO_CONTROL: u8 = 0xf7;

/// Command for Power Control 1
pub const POWER_CONTROL_1: u8 = 0xc0;

/// Command for Power Control 2
pub const POWER_CONTROL_2: u8 = 0xc1;

/// Command for VCOM Control 1
pub const VCOM_CONTROL_1: u8 = 0xc5;

/// Command for VCOM Control 2
pub const VCOM_CONTROL_2: u8 = 0xc7;

/// Command for Frame r=Rate Control 1 (In normal mode/ Full colors)
pub const FRAME_RATE_CONTROL_1: u8 = 0xb1;

/// Command for Display Function Control
pub const DISPLAY_FUNCTION_CONTROL: u8 = 0xb6;

/// Command for Enable 3 gamma control
pub const ENABLE_3G: u8 = 0xf2;

/// Command for Gamma Set
pub const GAMMA_SET: u8 = 0x26;

/// Command for Postive Gamma Correction
pub const POSITIVE_GAMMA_CORRECTION: u8 = 0xe0;

/// Command for Negative Gamma Correction
pub const NEGATIVE_GAMMA_CORRECTION: u8 = 0xe1;
