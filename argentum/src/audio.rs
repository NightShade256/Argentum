/// The rate at which samples are consumed by the audio
/// driver.
pub const SAMPLE_RATE: usize = 48000;

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
    apu_enabled: bool,

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

    /// Stub
    left_vin: bool,

    /// Stub
    right_vin: bool,
}

impl Apu {
    /// Create a new `Apu` instance.
    pub fn new(callback: Box<dyn Fn(&[f32])>) -> Self {
        Self {
            left_volume: 0,
            right_volume: 0,
            nr51: 0,
            apu_enabled: false,
            channel_one: ChannelOne::default(),
            channel_two: ChannelTwo::default(),
            channel_three: ChannelThree::default(),
            channel_four: ChannelFour::default(),
            sample_clock: 0,
            buffer: Box::new([0.0; 1024]),
            buffer_position: 0,
            is_buffer_full: false,
            callback,
            frame_sequencer_position: 0,
            left_vin: false,
            right_vin: false,
        }
    }

    /// Tick the APU by 1 M-cycle.
    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
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
                        self.channel_one.step_sweep();
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
                        self.channel_one.step_sweep();
                    }

                    7 => {
                        self.channel_one.step_volume();
                        self.channel_two.step_volume();
                        self.channel_four.step_volume();
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
            0xFF24 => {
                (if self.left_vin { 0b1000_0000 } else { 0 })
                    | (self.left_volume << 4)
                    | (if self.right_vin { 0b0000_1000 } else { 0 })
                    | self.right_volume
            }

            0xFF25 => self.nr51,

            0xFF26 => {
                let mut nr52 = ((self.apu_enabled as u8) << 7) | 0x70;

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
            0xFF15..=0xFF19 => self.channel_two.read_byte(addr),

            // Channel 3 IO registers + Wave RAM.
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_three.read_byte(addr),

            // Channel 4 IO registers.
            0xFF1F..=0xFF23 => self.channel_four.read_byte(addr),

            _ => unreachable!(),
        }
    }

    /// Write a byte to the given address.
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        if !(self.apu_enabled
            || addr == 0xFF26
            || (0xFF30..=0xFF3F).contains(&addr)
            || [0xFF11, 0xFF16, 0xFF1B, 0xFF20].contains(&addr))
        {
            return;
        }

        let value = if !self.apu_enabled && [0xFF11, 0xFF16, 0xFF20].contains(&addr) {
            value & 0b0011_1111
        } else {
            value
        };

        match addr {
            0xFF24 => {
                self.left_volume = (value >> 4) & 0x07;
                self.right_volume = value & 0x07;

                self.left_vin = (value & 0b1000_0000) != 0;
                self.right_vin = (value & 0b0000_1000) != 0;
            }

            0xFF25 => self.nr51 = value,

            0xFF26 => {
                let enabled = (value >> 7) != 0;

                if !enabled && self.apu_enabled {
                    for addr in 0xFF10..=0xFF25 {
                        self.write_byte(addr, 0x00);
                    }

                    self.apu_enabled = false;
                } else if enabled && !self.apu_enabled {
                    self.apu_enabled = true;

                    self.frame_sequencer_position = 0;

                    self.channel_one.wave_position = 0;
                    self.channel_two.wave_position = 0;
                    self.channel_three.wave_position = 0;
                }
            }

            // Channel 1 IO registers.
            0xFF10..=0xFF14 => self.channel_one.write_byte(addr, value),

            // Channel 2 IO registers.
            0xFF15..=0xFF19 => self.channel_two.write_byte(addr, value),

            // Channel 3 IO registers + Wave RAM.
            0xFF1A..=0xFF1E | 0xFF30..=0xFF3F => self.channel_three.write_byte(addr, value),

            // Channel 4 IO registers.
            0xFF1F..=0xFF23 => self.channel_four.write_byte(addr, value),

            _ => unreachable!(),
        }
    }
}

/// Implementation of the square wave channel one with envelope and sweep.
#[derive(Default)]
pub struct ChannelOne {
    /// Tells whether the channel's DAC is enabled or not.
    dac_enabled: bool,

    /// Tells whether the channel itself it enabled.
    /// This can be only affected by a trigger event.
    channel_enabled: bool,

    /// This is equal to `(2048 - frequency) * 4`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u16,

    /// The position we are currently in the wave.
    wave_position: usize,

    /// Sweep Time, after this time a new frequency is calculated.
    sweep_period: u8,

    /// Is the sweep incrementing or decrementing in nature.
    sweep_is_decrementing: bool,

    /// The amount by which the frequency is changed.
    sweep_amount: u8,

    /// The amount of sweep steps the channels has received through the
    /// FS.
    sweep_period_timer: u8,

    /// If the sweep function is enabled or not?
    sweep_enabled: bool,

    /// Stores the previous calculated frequency, depending upon some
    /// conditions.
    shadow_frequency: u16,

    /// The wave pattern duty currently in use.
    duty_pattern: u8,

    /// The sound length counter. If this is >0 and bit 6 in NR24 is set
    /// then it is decremented with clocks from FS. If this then hits 0
    /// the sound channel is then disabled.
    length_counter: u8,

    /// The channel frequency value. This is controlled by NR23 and NR24.
    frequency: u16,

    /// Whether the length timer is enabled or not.
    length_enabled: bool,

    /// The initial volume of the envelope function.
    initial_volume: u8,

    /// Whether the envelope is incrementing or decrementing in nature.
    is_incrementing: bool,

    /// The amount of volume steps through the FS for volume to
    /// change.
    period: u8,

    /// The amount of volume steps the channels has received through the
    /// FS.
    period_timer: u8,

    /// The current volume of the channel.
    current_volume: u8,
}

impl ChannelOne {
    /// Steps the envelope function.
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing)
                    || (self.current_volume > 0 && !self.is_incrementing)
                {
                    if self.is_incrementing {
                        self.current_volume += 1;
                    } else {
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }

    /// Steps the sweep function.
    pub fn step_sweep(&mut self) {
        if self.sweep_period_timer > 0 {
            self.sweep_period_timer -= 1;
        }

        if self.sweep_period_timer == 0 {
            self.sweep_period_timer = if self.sweep_period > 0 {
                self.sweep_period
            } else {
                8
            };

            if self.sweep_enabled && self.sweep_period > 0 {
                let new_frequency = self.calculate_frequency();

                if new_frequency <= 2047 && self.sweep_amount > 0 {
                    self.frequency = new_frequency;
                    self.shadow_frequency = new_frequency;

                    self.calculate_frequency();
                }
            }
        }
    }

    /// Calculate the new frequency, and perform the overflow check.
    fn calculate_frequency(&mut self) -> u16 {
        let mut new_frequency = self.shadow_frequency >> self.sweep_amount;

        new_frequency = if self.sweep_is_decrementing {
            self.shadow_frequency - new_frequency
        } else {
            self.shadow_frequency + new_frequency
        };

        if new_frequency > 2047 {
            self.channel_enabled = false;
        }

        new_frequency
    }
}

impl Channel for ChannelOne {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => {
                (self.sweep_period << 4)
                    | (if self.sweep_is_decrementing {
                        0x08
                    } else {
                        0x00
                    })
                    | self.sweep_amount
                    | 0x80
            }

            0xFF11 => (self.duty_pattern << 6) | 0b0011_1111,

            0xFF12 => {
                (self.initial_volume << 4)
                    | (if self.is_incrementing { 0x08 } else { 0x00 })
                    | self.period
            }

            0xFF13 => 0xFF,

            0xFF14 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => {
                // Update the sweep function parameters.
                self.sweep_is_decrementing = (value & 0x08) != 0;
                self.sweep_period = value >> 4;
                self.sweep_amount = value & 0x07;
            }

            0xFF11 => {
                self.duty_pattern = (value >> 6) & 0b11;

                // The length counter is calculated by the following formula,
                // `Length Counter = (64 - Length Data)`
                self.length_counter = 64 - (value & 0b0011_1111);
            }
            0xFF12 => {
                // Update the envelope function parameters.
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = ((value >> 3) & 0b11111) != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }

            0xFF13 => {
                // Update frequency with the lower eight bits.
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }

            0xFF14 => {
                // Update frequency with the upper three bits.
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64.
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;

                    // Trigger the envelope function.
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;

                    // Trigger the sweep function.
                    self.shadow_frequency = self.frequency;

                    // Sweep period of 0 is treated as 8 for some reason.
                    self.sweep_period_timer = if self.sweep_period > 0 {
                        self.sweep_period
                    } else {
                        8
                    };

                    self.sweep_enabled = self.sweep_period > 0 || self.sweep_amount > 0;

                    if self.sweep_amount > 0 {
                        self.calculate_frequency();
                    }
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
            let input = WAVE_DUTY[self.duty_pattern as usize][self.wave_position] as f32
                * self.current_volume as f32;

            (input / 7.5) - 1.0
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

    /// The channel frequency value. This is controlled by NR23 and NR24.
    frequency: u16,

    /// Whether the length timer is enabled or not.
    length_enabled: bool,

    /// The initial volume of the envelope function.
    initial_volume: u8,

    /// Whether the envelope is incrementing or decrementing in nature.
    is_incrementing: bool,

    /// The amount of volume steps through the FS for volume to
    /// change.
    period: u8,

    /// The amount of volume steps the channels has received through the
    /// FS.
    period_timer: u8,

    /// The current volume of the channel.
    current_volume: u8,
}

impl ChannelTwo {
    /// Steps the envelope function.
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing)
                    || (self.current_volume > 0 && !self.is_incrementing)
                {
                    if self.is_incrementing {
                        self.current_volume += 1;
                    } else {
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }
}

impl Channel for ChannelTwo {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // There is no NR10 register.
            0xFF15 => 0xFF,

            // The length data is only write-only and when read
            // it is all set to 1s.
            0xFF16 => (self.duty_pattern << 6) | 0b0011_1111,

            0xFF17 => {
                (self.initial_volume << 4)
                    | (if self.is_incrementing { 0x08 } else { 0x00 })
                    | self.period
            }

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
                // Update the envelope function parameters.
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = (value & 0b1111_1000) != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }

            0xFF18 => {
                // Update frequency with the lower eight bits.
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }

            0xFF19 => {
                // Update frequency with the upper three bits.
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64.
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;

                    // Envelope is triggered.
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;
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
            let input = WAVE_DUTY[self.duty_pattern as usize][self.wave_position] as f32
                * self.current_volume as f32;

            (input / 7.5) - 1.0
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
#[derive(Default)]
pub struct ChannelThree {
    /// Whether the channel DAC is enabled or not.
    dac_enabled: bool,

    /// Whether the channel itself it enabled.
    /// This can be only affected by a trigger event.
    channel_enabled: bool,

    /// This is equal to `(2048 - frequency) * 2`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u16,

    /// The current sample being played in the wave ram.
    wave_position: usize,

    /// The sound length counter. If this is >0 and bit 6 in NR24 is set
    /// then it is decremented with clocks from FS. If this then hits 0
    /// the sound channel is then disabled.
    length_counter: u16,

    /// Output level configuration register.
    /// Sets the volume shift for the wave data.
    output_level: u8,

    /// The volume shift computed from the output level
    /// register.
    volume_shift: u8,

    /// The channel frequency value. This is controlled by NR23 and NR24.
    frequency: u16,

    /// Whether the length timer is enabled or not.
    length_enabled: bool,

    /// Arbitrary 32 4-bit samples.
    wave_ram: Box<[u8; 0x10]>,
}

impl Channel for ChannelThree {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => ((self.dac_enabled as u8) << 7) | 0x7F,
            0xFF1B => 0xFF,
            0xFF1C => (self.output_level << 5) | 0x9F,
            0xFF1D => 0xFF,
            0xFF1E => ((self.length_enabled as u8) << 6) | 0b1011_1111,

            0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize],

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => {
                self.dac_enabled = (value >> 7) & 0b1 != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }

            0xFF1B => {
                self.length_counter = 256 - value as u16;
            }

            0xFF1C => {
                self.output_level = (value >> 5) & 0b11;

                self.volume_shift = match self.output_level {
                    0b00 => 4,
                    0b01 => 0,
                    0b10 => 1,
                    0b11 => 2,

                    _ => unreachable!(),
                };
            }

            0xFF1D => {
                // Update frequency with the lower eight bits.
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }

            0xFF1E => {
                // Update frequency with the upper three bits.
                self.frequency = (self.frequency & 0xFF) | (((value & 0x07) as u16) << 8);

                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64.
                if self.length_counter == 0 {
                    self.length_counter = 256;
                }

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
        // `(2048 - frequency) * 2` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 2;

            // Wave position is wrapped, so when the position is >32 it's
            // wrapped back to 0.
            self.wave_position = (self.wave_position + 1) & 31;
        }

        self.frequency_timer -= 1;
    }

    /// Get the current amplitude of the channel.
    fn get_amplitude(&self) -> f32 {
        if self.dac_enabled {
            let sample = ((self.wave_ram[self.wave_position / 2])
                >> (if (self.wave_position & 1) != 0 { 4 } else { 0 }))
                & 0x0F;

            (((sample >> self.volume_shift) as f32) / 7.5) - 1.0
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

/// Implementation of the noise channel four.
#[derive(Default)]
pub struct ChannelFour {
    /// Whether the channel DAC is enabled or not.
    dac_enabled: bool,

    /// Tells whether the channel itself it enabled.
    /// This can be only affected by the `length` parameter.
    channel_enabled: bool,

    /// This is equal to `(2048 - frequency) * 2`
    /// This timer is decremented every T-cycle.
    /// When this timer reaches 0, wave generation is stepped, and
    /// it is reloaded.
    frequency_timer: u16,

    /// The linear feedback shift register (LFSR) generates a pseudo-random bit sequence.
    lfsr: u16,

    /// The sound length counter. If this is >0 and bit 6 in NR24 is set
    /// then it is decremented with clocks from FS. If this then hits 0
    /// the sound channel is then disabled.
    length_counter: u8,

    /// The polynomial counter, used to control the RNG.
    nr43: u8,

    /// Whether the length timer is enabled or not.
    length_enabled: bool,

    /// The initial volume of the envelope function.
    initial_volume: u8,

    /// Whether the envelope is incrementing or decrementing in nature.
    is_incrementing: bool,

    /// The amount of volume steps through the FS for volume to
    /// change.
    period: u8,

    /// The amount of volume steps the channels has received through the
    /// FS.
    period_timer: u8,

    /// The current volume of the channel.
    current_volume: u8,
}

impl ChannelFour {
    /// Steps the envelope function.
    pub fn step_volume(&mut self) {
        if self.period != 0 {
            if self.period_timer > 0 {
                self.period_timer -= 1;
            }

            if self.period_timer == 0 {
                self.period_timer = self.period;

                if (self.current_volume < 0xF && self.is_incrementing)
                    || (self.current_volume > 0 && !self.is_incrementing)
                {
                    if self.is_incrementing {
                        self.current_volume += 1;
                    } else {
                        self.current_volume -= 1;
                    }
                }
            }
        }
    }
}

impl Channel for ChannelFour {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // NR40 does not exist.
            0xFF1F => 0xFF,

            0xFF20 => 0xFF,

            0xFF21 => {
                (self.initial_volume << 4)
                    | (if self.is_incrementing { 0x08 } else { 0x00 })
                    | self.period
            }

            0xFF22 => self.nr43,

            0xFF23 => ((self.length_enabled as u8) << 6) | 0b1011_1111,

            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // NR40 does not exist.
            0xFF1F => {}

            0xFF20 => {
                self.length_counter = 64 - (value & 0b0011_1111);
            }

            0xFF21 => {
                // Update the envelope function parameters.
                self.is_incrementing = (value & 0x08) != 0;
                self.initial_volume = value >> 4;
                self.period = value & 0x07;

                // Check DAC status. If top 5 bits are reset
                // DAC will be disabled.
                self.dac_enabled = (value & 0b1111_1000) != 0;

                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }

            0xFF22 => self.nr43 = value,

            0xFF23 => {
                self.length_enabled = ((value >> 6) & 0x01) != 0;

                // If length counter is zero reload it with 64.
                if self.length_counter == 0 {
                    self.length_counter = 64;
                }

                // Restart the channel iff DAC is enabled and trigger is set.
                let trigger = (value >> 7) != 0;

                if trigger && self.dac_enabled {
                    self.channel_enabled = true;
                }

                if trigger {
                    // On trigger event all bits of LFSR are turned on.
                    self.lfsr = 0x7FFF;

                    // Envelope is triggered.
                    self.period_timer = self.period;
                    self.current_volume = self.initial_volume;
                }
            }

            _ => unreachable!(),
        }
    }

    fn tick_channel(&mut self) {
        // If the frequency timer decrement to 0, it is reloaded with the formula
        // `divisor_code << clock_shift` and wave position is advanced by one.
        if self.frequency_timer == 0 {
            let divisor_code = (self.nr43 & 0x07) as u16;

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
            let input = (!self.lfsr & 0b01) as f32 * self.current_volume as f32;

            (input / 7.5) - 1.0
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
