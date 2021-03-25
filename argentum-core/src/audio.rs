use alloc::boxed::Box;

const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

pub trait Channel {
    /// Tick the channel by 4 T-cycles.
    fn tick_channel(&mut self);

    /// Get the current amplitude of the channel.
    fn get_amplitude(&self) -> f32;

    /// Read a byte from the specified address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte to the specified address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Set the channel's enabled attribute.
    fn set_enabled(&mut self, enabled: bool);
}

pub struct Apu {
    nr50: u8,
    nr51: u8,
    nr52: u8,

    ch2: ChannelTwo,

    pub buffer: Box<[f32; 2112]>,
    sample_clock: u32,
    buffer_pos: usize,
    pub is_full: bool,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            nr50: 0,
            nr51: 0,
            nr52: 0,
            ch2: ChannelTwo::new(),
            buffer: Box::new([0.0; 2112]),
            buffer_pos: 0,
            is_full: false,
            sample_clock: 0,
        }
    }

    pub fn tick(&mut self) {
        self.ch2.tick_channel();

        self.sample_clock += 4;

        if self.sample_clock >= 87 {
            self.sample_clock = 0;

            self.buffer[self.buffer_pos] = (((self.nr50 >> 4) & 0x07) as f32 / 7.0)
                * (if (self.nr51 & 0x20) != 0 {
                    self.ch2.get_amplitude()
                } else {
                    0.0
                });

            self.buffer[self.buffer_pos + 1] = ((self.nr50 & 0x07) as f32 / 7.0)
                * (if (self.nr51 & 0x02) != 0 {
                    self.ch2.get_amplitude()
                } else {
                    0.0
                });

            self.buffer_pos += 2;
        }

        if self.buffer_pos >= self.buffer.len() {
            self.is_full = true;
            self.buffer_pos = 0;
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => self.nr52,

            0xFF16..=0xFF19 => self.ch2.read_byte(addr),

            _ => unreachable!(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                self.nr52 = value;
                //log::info!("self.nr52 & 0x02 != 0 == {:?}", self.nr52 & 0x02 != 0);
                //self.ch2.set_enabled(self.nr52 & 0x02 != 0);
            }
            0xFF16..=0xFF19 => self.ch2.write_byte(addr, value),

            _ => unreachable!(),
        }
    }
}

pub struct ChannelTwo {
    // Sound IO registers for channel two.
    nr21: u8,
    nr22: u8,
    nr23: u8,
    nr24: u8,

    /// This is equal to `(2048 - frequency) * 4`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped.
    frequency_timer: u32,

    /// The position we are currently in the wave.
    wave_position: usize,

    /// Tells whether the channel is enabled or not.
    enabled: bool,
}

impl ChannelTwo {
    pub fn new() -> Self {
        Self {
            nr21: 0,
            nr22: 0,
            nr23: 0,
            nr24: 0,
            frequency_timer: 0,
            wave_position: 0,
            enabled: true,
        }
    }
}

impl Channel for ChannelTwo {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF16 => self.nr21,
            0xFF17 => self.nr22,
            0xFF18 => self.nr23,
            0xFF19 => self.nr24,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF16 => self.nr21 = value,
            0xFF17 => self.nr22 = value,
            0xFF18 => {
                self.nr23 = value;
                self.frequency_timer = (2048 - ((self.nr24 & 0x07) as u32 | self.nr23 as u32)) * 4;
            }
            0xFF19 => {
                self.nr24 = value;
                self.frequency_timer = (2048 - ((self.nr24 & 0x07) as u32 | self.nr23 as u32)) * 4;
            }

            _ => unreachable!(),
        }
    }

    fn tick_channel(&mut self) {
        for _ in 0..4 {
            self.frequency_timer = self.frequency_timer.saturating_sub(1);

            // Step wave generation and reload the frequency timer.
            if self.frequency_timer == 0 {
                self.frequency_timer = (2048 - ((self.nr24 & 0x07) as u32 | self.nr23 as u32)) * 4;
                self.wave_position = (self.wave_position + 1) & 7;
            }
        }
    }

    fn get_amplitude(&self) -> f32 {
        if self.enabled {
            //log::info!("here1");
            WAVE_DUTY[(self.nr21 >> 6) as usize][self.wave_position] as f32
        } else {
            0.0
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
