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

    /// Step the length timer of the channel.
    fn step_length(&mut self);
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

    /// Implementation of the custom wave channel.
    channel_three: ChannelThree,

    /// Implementation of the noise wave channel.
    channel_four: ChannelFour,

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

    /// The position the FS is currently in.
    frame_sequencer_position: u8,
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
            channel_two: ChannelTwo::default(),
            channel_three: ChannelThree::new(),
            channel_four: ChannelFour::new(),
            sample_clock: 0,
            buffer: Box::new([0.0; 1024]),
            buffer_position: 0,
            is_buffer_full: false,
            callback,
            frame_sequencer_position: 0,
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
            self.channel_three.tick_channel();
            self.channel_four.tick_channel();

            // Tick the frame sequencer. It generates clocks for the length,
            // envelope and sweep functions.
            if self.sample_clock % 8192 == 0 {
                self.sample_clock = 0;

                match self.frame_sequencer_position {
                    0 => {
                        self.channel_one.step_length();
                        self.channel_two.step_length();
                        self.channel_three.step_length();
                        self.channel_four.step_length();
                    }

                    2 => {
                        self.channel_one.step_length();
                        self.channel_two.step_length();
                        self.channel_three.step_length();
                        self.channel_four.step_length();
                    }

                    4 => {
                        self.channel_one.step_length();
                        self.channel_two.step_length();
                        self.channel_three.step_length();
                        self.channel_four.step_length();
                    }

                    6 => {
                        self.channel_one.step_length();
                        self.channel_two.step_length();
                        self.channel_three.step_length();
                        self.channel_four.step_length();
                    }

                    _ => {}
                }

                self.frame_sequencer_position = (self.frame_sequencer_position + 1) & 7;
            }

            // Each (CPU CLOCK / SAMPLE RATE) cycles one sample is generated
            // and pushed to the buffer.
            if self.sample_clock % ((CPU_CLOCK / SAMPLE_RATE) as u32) == 0 {
                self.buffer[self.buffer_position] = (self.left_volume as f32 / 7.0)
                    * ((if (self.nr51 & 0x80) != 0 {
                        self.channel_four.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x40) != 0 {
                        self.channel_three.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x20) != 0 {
                        self.channel_two.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x10) != 0 {
                        self.channel_one.get_amplitude()
                    } else {
                        0.0
                    }) / 4.0);

                self.buffer[self.buffer_position + 1] = (self.right_volume as f32 / 7.0)
                    * ((if (self.nr51 & 0x08) != 0 {
                        self.channel_four.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x04) != 0 {
                        self.channel_three.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x02) != 0 {
                        self.channel_two.get_amplitude()
                    } else {
                        0.0
                    } + if (self.nr51 & 0x01) != 0 {
                        self.channel_one.get_amplitude()
                    } else {
                        0.0
                    }) / 4.0);

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
            0xFF26 => {
                let mut nr52 = (self.enabled as u8) << 7;

                // Set the status bits appropriately.
                nr52 |= self.channel_one.channel_enabled as u8;
                nr52 |= (self.channel_two.channel_enabled as u8) << 1;
                nr52 |= (self.channel_three.channel_enabled as u8) << 2;
                nr52 |= (self.channel_four.channel_enabled as u8) << 3;

                nr52
            }

            // Channel 1 IO registers.
            0xFF10..=0xFF14 => self.channel_one.read_byte(addr),

            // Channel 2 IO registers.
            0xFF16..=0xFF19 => self.channel_two.read_byte(addr),

            // Channel 3 IO registers + Wave RAM.
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_three.read_byte(addr),

            // Channel 4 IO registers.
            0xFF20..=0xFF23 => self.channel_four.read_byte(addr),

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

            // Channel 3 IO registers + Wave RAM.
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_three.write_byte(addr, value),

            // Channel 4 IO registers.
            0xFF20..=0xFF23 => self.channel_four.write_byte(addr, value),

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

    /// Tells whether the channel itself it enabled.
    /// This can be only affected by the `length` parameter.
    channel_enabled: bool,
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
            channel_enabled: false,
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
            0xFF14 => {
                self.nr14 = value;

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }
            }

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
        if self.dac_enabled && self.channel_enabled {
            WAVE_DUTY[(self.nr11 >> 6) as usize][self.wave_position] as f32
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if ((self.nr14 >> 6) & 0x01) != 0 && (self.nr11 & 0b111111) > 0 {
            let timer = (self.nr11 & 0b111111) - 1;

            self.nr11 = (self.nr11 & 0b1100_0000) | timer;

            if timer == 0 {
                self.channel_enabled = false;
            }
        }
    }
}

/// Implementation of the square wave channel two with an envelope function.
#[derive(Default)]
pub struct ChannelTwo {
    /// Whether the channel DAC is enabled or not.
    dac_enabled: bool,

    /// Whether the channel itself is enabled.
    /// This can be only affected by a trigger event.
    channel_enabled: bool,

    /// This is equal to `(2048 - frequency) * 4`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u16,

    /// The position we are currently in the wave pattern duty.
    wave_position: usize,

    /// The wave pattern duty currently in use.
    duty_pattern: u8,

    /// The sound length counter. If this is >0 and bit 6 in NR24 is set
    /// then it is decremented with clocks from FS. If this then hits 0
    /// the sound channel is then disabled.
    length_counter: u8,

    /// Controls the envelope function for this channel.
    nr22: u8,

    /// The channel frequency value. This is controlled by NR23 and NR24.
    frequency: u16,

    /// Whether the length timer is enabled or not.
    length_enabled: bool,
}

impl Channel for ChannelTwo {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // There is no NR10 register.
            0xFF15 => 0xFF,

            // The length data is only write-only and when read
            // it is all set to 1s.
            0xFF16 => (self.duty_pattern << 6) | 0b0011_1111,

            // TODO - Implement Envelope.
            0xFF17 => self.nr22,

            // NR23 is a write only register.
            0xFF18 => 0xFF,

            0xFF19 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // There is no NR10 register.
            0xFF15 => {}

            0xFF16 => {
                self.duty_pattern = (value >> 6) & 0b11;

                // The length counter is calculated by the following formula,
                // `Length Counter = (64 - Length Data)`
                self.length_counter = 64 - (value & 0b0011_1111);
            }

            0xFF17 => {
                self.nr22 = value;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = (self.nr22 & 0b1111_1000) != 0;
            }

            0xFF18 => {
                // Update frequency with the lower eight bits.
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }

            0xFF19 => {
                // Update frequency with the upper three bits.
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }
            }

            _ => unreachable!(),
        }
    }

    /// Tick the channel by one T-cycle.
    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 4;

            // Wave position is wrapped, so when the position is >8 it's
            // wrapped back to 0.
            self.wave_position = (self.wave_position + 1) & 7;
        }

        self.frequency_timer -= 1;
    }

    /// Get the current amplitude of the channel.
    /// The only possible values of this are 0 or 1.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            WAVE_DUTY[self.duty_pattern as usize][self.wave_position] as f32
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;

            // The channel is disabled if the length counter is reset.
            if self.length_counter == 0 {
                self.channel_enabled = false;
            }
        }
    }
}

/// Implementation of the custom wave channel.
pub struct ChannelThree {
    /// If the channel DAC is enabled or not.
    dac_enabled: bool,

    /// Sound Length configuration register.
    nr31: u8,

    /// Output level configuration register.
    output_level: u8,

    /// The volume shift specifier.
    volume_shift: u8,

    /// Lower 8 bits of the channel frequency.
    nr33: u8,

    /// Contains some more control bits + higher three
    /// bits of channel frequency.
    nr34: u8,

    /// This is equal to `(2048 - frequency) * 2`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u32,

    /// Arbitrary 32 4-bit samples.
    wave_ram: Box<[u8; 0x10]>,

    /// The current sample being played in the wave ram.
    position: usize,

    /// Tells whether the channel itself it enabled.
    /// This can be only affected by the `length` parameter.
    channel_enabled: bool,
}

impl ChannelThree {
    pub fn new() -> Self {
        Self {
            dac_enabled: false,
            nr31: 0,
            output_level: 0,
            volume_shift: 0,
            nr33: 0,
            nr34: 0,
            frequency_timer: 0,
            wave_ram: Box::new([0; 0x10]),
            position: 0,
            channel_enabled: false,
        }
    }
}

impl Channel for ChannelThree {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => (self.dac_enabled as u8) << 7,
            0xFF1B => self.nr31,
            0xFF1C => self.output_level << 5,
            0xFF1D => self.nr33,
            0xFF1E => self.nr34,

            0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize],

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => {
                self.dac_enabled = (value >> 7) & 0b1 != 0;
            }
            0xFF1B => self.nr31 = value,
            0xFF1C => {
                self.output_level = (value & 0x60) >> 5;

                self.volume_shift = match self.output_level {
                    0b00 => 4,
                    0b01 => 0,
                    0b10 => 1,
                    0b11 => 2,

                    _ => unreachable!(),
                };
            }
            0xFF1D => self.nr33 = value,
            0xFF1E => {
                self.nr34 = value;

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }
            }

            0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize] = value,

            _ => unreachable!(),
        }
    }

    /// Tick the channel by one T-cycle.
    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `(2048 - frequency) * 4` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let frequency = (((self.nr34 & 0x07) as u32) << 8) | (self.nr33 as u32);

            self.frequency_timer = (2048 - frequency) * 2;

            // Wave position is wrapped, so when the position is >16 it's
            // wrapped back to 0.
            self.position = (self.position + 1) & 15;
        }

        self.frequency_timer -= 1;
    }

    /// Get the current amplitude of the channel.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled {
            let sample = ((self.wave_ram[self.position / 2])
                >> (if (self.position % 2) != 0 { 4 } else { 0 }))
                & 0x0F;

            ((sample >> self.volume_shift) as f32) / 7.5 - 1.0
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if ((self.nr34 >> 6) & 0x01) != 0 && (self.nr31 & 0b111111) > 0 {
            self.nr31 -= 1;

            if self.nr31 == 0 {
                self.channel_enabled = false;
            }
        }
    }
}

/// Implementation of the noise channel four.
pub struct ChannelFour {
    /// If the channel DAC is enabled or not.
    dac_enabled: bool,

    /// Contains the five bits of the length data.
    nr41: u8,

    /// Controls the envelope function.
    nr42: u8,

    /// The polynomial counter, used to control the RNG.
    nr43: u8,

    /// Control register, which has the trigger bit.
    nr44: u8,

    /// Tells whether the channel itself it enabled.
    /// This can be only affected by the `length` parameter.
    channel_enabled: bool,

    /// The linear feedback shift register (LFSR) generates a pseudo-random bit sequence.
    lfsr: u16,

    /// This is equal to `(2048 - frequency) * 2`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u32,
}

impl ChannelFour {
    pub fn new() -> Self {
        Self {
            dac_enabled: false,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            channel_enabled: false,
            lfsr: 0,
            frequency_timer: 0,
        }
    }
}

impl Channel for ChannelFour {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF20 => self.nr41,
            0xFF21 => self.nr42,
            0xFF22 => self.nr43,
            0xFF23 => self.nr44,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF20 => self.nr41 = value,
            0xFF21 => {
                self.nr42 = value;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = ((self.nr42 >> 3) & 0b11111) != 0;
            }
            0xFF22 => self.nr43 = value,
            0xFF23 => {
                self.nr44 = value;

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }
            }

            _ => unreachable!(),
        }
    }

    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `divisor_code << clockshift` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let divisor_code = (self.nr43 & 0x07) as u32;

            self.frequency_timer = (if divisor_code == 0 {
                8
            } else {
                divisor_code << 4
            }) << ((self.nr43 >> 4) as u32);

            let xor_result = (self.lfsr & 0b01) ^ ((self.lfsr & 0b10) >> 1);

            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

            if ((self.nr43 >> 3) & 0b01) != 0 {
                self.lfsr &= !(1 << 6);
                self.lfsr |= xor_result << 6;
            }
        }

        self.frequency_timer -= 1;
    }

    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            (!self.lfsr & 0b01) as f32
        } else {
            0.0
        }
    }

    fn step_length(&mut self) {
        if ((self.nr44 >> 6) & 0x01) != 0 && (self.nr41 & 0b111111) > 0 {
            self.nr41 -= 1;

            if self.nr41 == 0 {
                self.channel_enabled = false;
            }
        }
    }
}
