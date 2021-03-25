use alloc::boxed::Box;

/// The rate at which samples are consumed by the audio
/// driver.
pub const SAMPLE_RATE: usize = 65536;

/// The size of the audio sample buffer.
pub const BUFFER_SIZE: usize = 1024;

/// The rate at which the CPU is ticked.
pub const CPU_CLOCK: usize = 4194304;

/// Table for all the defined wave duties.
const WAVE_DUTY: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75%
];

pub trait Channel {
    /// Read a byte from the specified address.
    fn read_byte(&self, addr: u16) -> u8;

    /// Write a byte to the specified address.
    fn write_byte(&mut self, addr: u16, value: u8);

    /// Tick the channel by 4 T-cycles.
    fn tick_channel(&mut self);

    /// Get the current amplitude of the channel.
    fn get_amplitude(&self) -> f32;
}

pub struct Apu {
    /// The volume value for the left channel.
    left_volume: u8,

    /// The volume value for the right channel.
    right_volume: u8,

    /// $FF25 - Controls which stereo channels, sound is outputted to.
    nr51: u8,

    /// APU enabled - Controls whether the APU is ticking.
    enabled: bool,

    /// Implementation of the square wave channel one with envelope and sweep function.
    channel_one: ChannelOne,

    /// Implementation of the square wave channel two with envelope function.
    channel_two: ChannelTwo,

    /// Used to clock FS and sample generation.
    sample_clock: u32,

    /// The audio buffer which contains 32-bit float samples.
    pub buffer: Box<[f32; BUFFER_SIZE]>,

    /// The position we are currently in the audio buffer.
    pub buffer_position: usize,

    /// Indicates if the sample buffer is full.
    pub is_buffer_full: bool,

    /// Audio callback which is called when the sample buffer is full.
    callback: Box<dyn Fn(&[f32])>,
}

impl Apu {
    /// Create a new `Apu` instance.
    pub fn new(callback: Box<dyn Fn(&[f32])>) -> Self {
        Self {
            left_volume: 0,
            right_volume: 0,
            nr51: 0,
            enabled: false,
            channel_one: ChannelOne::new(),
            channel_two: ChannelTwo::new(),
            sample_clock: 0,
            buffer: Box::new([0.0; 1024]),
            buffer_position: 0,
            is_buffer_full: false,
            callback,
        }
    }

    /// Tick the APU by 1 M-cycle.
    pub fn tick(&mut self) {
        for _ in 0..4 {
            // This clock is incremented every T-cycle.
            // This is used to clock the frame sequencer and
            // to generate samples.
            self.sample_clock = self.sample_clock.wrapping_add(1);

            // Tick all the connected channels.
            self.channel_one.tick_channel();
            self.channel_two.tick_channel();

            // The logic of the FS goes here, but currently it's just
            // a stub.
            if self.sample_clock % 8192 == 0 {
                self.sample_clock = 0;
            }

            // Each (CPU CLOCK / SAMPLE RATE) cycles one sample is generated
            // and pushed to the buffer.
            if self.sample_clock % ((CPU_CLOCK / SAMPLE_RATE) as u32) == 0 {
                self.buffer[self.buffer_position] = (self.left_volume as f32 / 7.0)
                    * ((if (self.nr51 & 0x20) != 0 {
                        self.channel_two.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x10) != 0 {
                        self.channel_one.get_amplitude()
                    } else {
                        0.0
                    }) / 2.0);

                self.buffer[self.buffer_position + 1] = (self.right_volume as f32 / 7.0)
                    * ((if (self.nr51 & 0x02) != 0 {
                        self.channel_two.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x01) != 0 {
                        self.channel_one.get_amplitude()
                    } else {
                        0.0
                    }) / 2.0);

                self.buffer_position += 2;
            }

            // Checks if the buffer is full and calls the provided callback.
            if self.buffer_position >= BUFFER_SIZE {
                (self.callback)(self.buffer.as_ref());

                // Reset the buffer position.
                self.buffer_position = 0;
            }
        }
    }

    /// Read a byte from the given address.
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // NR50 - Controls volume for the stereo channels.
            0xFF24 => (self.left_volume << 4) | self.right_volume,
            0xFF25 => self.nr51,
            0xFF26 => (self.enabled as u8) << 7,

            // Channel 1 IO registers.
            0xFF10..=0xFF14 => self.channel_one.read_byte(addr),

            // Channel 2 IO registers.
            0xFF16..=0xFF19 => self.channel_two.read_byte(addr),

            _ => unreachable!(),
        }
    }

    /// Write a byte to the given address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF24 => {
                self.left_volume = (value >> 4) & 0x07;
                self.right_volume = value & 0x07;
            }
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                self.enabled = (value >> 7) != 0;
            }

            // Channel 1 IO registers.
            0xFF10..=0xFF14 => self.channel_one.write_byte(addr, value),

            // Channel 2 IO registers.
            0xFF16..=0xFF19 => self.channel_two.write_byte(addr, value),

            _ => unreachable!(),
        }
    }
}

/// Implementation of the square wave channel one.
pub struct ChannelOne {
    /// Controls the sweep functions for this channel.
    nr10: u8,

    /// Contains the wave duty pattern, and an optional
    /// length data.
    nr11: u8,

    /// Controls envelope function for this channel.
    nr12: u8,

    /// Lower eight bits of the channel frequency.
    nr13: u8,

    /// Contains some more control bits + higher three
    /// bits of channel frequency.
    nr14: u8,

    /// This is equal to `(2048 - frequency) * 4`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u32,

    /// The position we are currently in the wave.
    wave_position: usize,

    /// Tells whether the channel's DAC is enabled or not.
    dac_enabled: bool,
}

impl ChannelOne {
    pub fn new() -> Self {
        Self {
            nr10: 0,
            nr11: 0,
            nr12: 0,
            nr13: 0,
            nr14: 0,
            frequency_timer: 0,
            wave_position: 0,
            dac_enabled: false,
        }
    }
}

impl Channel for ChannelOne {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.nr10,
            0xFF11 => self.nr11,
            0xFF12 => self.nr12,
            0xFF13 => self.nr13,
            0xFF14 => self.nr14,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => self.nr10 = value,
            0xFF11 => self.nr11 = value,
            0xFF12 => {
                self.nr12 = value;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = ((self.nr12 >> 3) & 0b11111) != 0;
            }
            0xFF13 => self.nr13 = value,
            0xFF14 => self.nr14 = value,

            _ => unreachable!(),
        }
    }

    /// Tick the channel by one T-cycle.
    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let frequency = (((self.nr14 & 0x07) as u32) << 8) | (self.nr13 as u32);

            self.frequency_timer = (2048 - frequency) * 4;

            // Wave position is wrapped, so when the position is >8 it's
            // wrapped back to 0.
            self.wave_position = (self.wave_position + 1) & 7;
        }

        self.frequency_timer -= 1;
    }

    /// Get the current amplitude of the channel.
    /// The only possible values of this are 0 or 1.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled {
            WAVE_DUTY[(self.nr11 >> 6) as usize][self.wave_position] as f32
        } else {
            0.0
        }
    }
}

/// Implementation of the square wave channel two.
pub struct ChannelTwo {
    /// Contains the wave duty pattern, and an optional
    /// length data.
    nr21: u8,

    /// Controls envelope function for this channel.
    nr22: u8,

    /// Lower eight bits of the channel frequency.
    nr23: u8,

    /// Contains some more control bits + higher three
    /// bits of channel frequency.
    nr24: u8,

    /// This is equal to `(2048 - frequency) * 4`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u32,

    /// The position we are currently in the wave.
    wave_position: usize,

    /// Tells whether the channel's DAC is enabled or not.
    dac_enabled: bool,
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
            dac_enabled: false,
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
            0xFF17 => {
                self.nr22 = value;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = ((self.nr22 >> 3) & 0b11111) != 0;
            }
            0xFF18 => self.nr23 = value,
            0xFF19 => self.nr24 = value,

            _ => unreachable!(),
        }
    }

    /// Tick the channel by one T-cycle.
    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let frequency = (((self.nr24 & 0x07) as u32) << 8) | (self.nr23 as u32);

            self.frequency_timer = (2048 - frequency) * 4;

            // Wave position is wrapped, so when the position is >8 it's
            // wrapped back to 0.
            self.wave_position = (self.wave_position + 1) & 7;
        }

        self.frequency_timer -= 1;
    }

    /// Get the current amplitude of the channel.
    /// The only possible values of this are 0 or 1.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled {
            WAVE_DUTY[(self.nr21 >> 6) as usize][self.wave_position] as f32
        } else {
            0.0
        }
    }
}
