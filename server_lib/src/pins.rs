use crate::Axis;

/// A stepper driver. The variant order is the order of every per-motor pin array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motor {
    Y1,
    Y2,
    X,
    Head,
}

impl Motor {
    pub const COUNT: usize = 4;

    pub fn index(self) -> usize {
        match self {
            Motor::Y1 => 0,
            Motor::Y2 => 1,
            Motor::X => 2,
            Motor::Head => 3,
        }
    }

    /// The motors that drive `axis`.
    pub fn of_axis(axis: Axis) -> &'static [Motor] {
        match axis {
            Axis::X => &[Motor::X],
            Axis::Y => &[Motor::Y1, Motor::Y2],
            Axis::Head => &[Motor::Head],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareConsts {
    pub pin_settings: PinSettings,
    pub hardware_profile: HardwareProfile,
}

/// Pin numbers per motor, indexed by [`Motor::index`].
#[derive(Debug, Clone, Copy)]
pub struct PinSettings {
    // pins
    pub dir_pins: [u8; Motor::COUNT],
    pub step_pins: [u8; Motor::COUNT],
    pub enable_pins: [u8; Motor::COUNT],
    pub micstep_pins: [[u8; Motor::COUNT]; 3],

    // micro stepping
    pub micstep_vals: [[bool; 3]; Motor::COUNT],
}

#[derive(Debug, Clone, Copy)]
pub struct HardwareProfile {
    // distance and speed to cm
    pub dist_per_step_axis_cm: f32,
    pub dist_per_step_head_cm: f32,
    pub head_travel_to_touch_cm: f32,
}

impl HardwareProfile {
    pub fn steps_for_cm_head(&self, cm: f32) -> i32 {
        (cm / self.dist_per_step_head_cm).round() as i32
    }
}

// MICRO STEPPING
/*
    MICSTEP 1	MICSTEP 2	MICSTEP 3	MICSTEP RESOLUTION
    --------------------------------------------
    Low	        Low		    Low		    (1)    Full step
    High	    Low		    Low		    (1/2)  Half step
    Low	        High	    Low		    (1/4)  Quarter step
    High	    High	    Low		    (1/8)  Eighth step
    High	    High	    High	    (1/16) Sixteenth step
*/

// TODO: get settings from file
pub static HARDWARE_CONSTS: HardwareConsts = HardwareConsts {
    pin_settings: PinSettings {
        dir_pins: [18, 4, 11, 16],
        step_pins: [15, 3, 7, 20],
        enable_pins: [14, 2, 8, 21],
        micstep_pins: [[22, 10, 12, 13], [27, 9, 6, 19], [17, 25, 5, 26]],

        micstep_vals: [
            [true, true, true],
            [true, true, true],
            [true, true, true],
            [true, true, false],
        ],
    },
    hardware_profile: HardwareProfile {
        dist_per_step_axis_cm: 0.013_993_56 / 16.0, // distance per step / microstepping factor for x and y axes
        dist_per_step_head_cm: 0.8 / (200.0 * 8.0), // 8mm travel per revolution / (200 steps per revolution * microstepping factor for head)

        head_travel_to_touch_cm: 0.6, // cm
    },
};
