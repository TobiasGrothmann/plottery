// pins order is [Y1, Y2, X, HEAD]
#[derive(Debug)]
pub struct PinSettings {
    // pins
    pub dir_pins: [i32; 4],
    pub step_pins: [i32; 4],
    pub enable_pins: [i32; 4],
    pub micstep_pins: [[i32; 4]; 3],

    // micro stepping
    pub micstep_vals: [[char; 3]; 4],
    pub micstep_axes: i32,
    pub micstep_head: i32,

    // distance and speed to cm
    pub dist_per_step_axis_cm: f32,
    pub dist_per_step_head_cm: f32,
    pub head_travel_to_touch_cm: f32,
    pub extra_head_travel_for_pressure_cm: f32,
}

impl PinSettings {
    pub fn steps_for_cm_axis(&self, cm: f32) -> i32 {
        (cm / self.dist_per_step_axis_cm).round() as i32
    }
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
pub static PIN_SETTINGS: PinSettings = PinSettings {
    dir_pins: [18, 4, 11, 16],
    step_pins: [15, 3, 7, 20],
    enable_pins: [14, 2, 8, 21],
    micstep_pins: [[22, 10, 12, 13], [27, 9, 6, 19], [17, 25, 5, 26]],

    micstep_vals: [
        ['1', '1', '1'],
        ['1', '1', '1'],
        ['1', '1', '1'],
        ['1', '1', '0'],
    ],
    micstep_axes: 16,
    micstep_head: 8,

    dist_per_step_axis_cm: 0.0139935599999 / 16.0, // cm
    dist_per_step_head_cm: 0.8 / (200.0 * 8.0), // cm (200 steps per revolution, 8mm travel per revolution)

    head_travel_to_touch_cm: 0.6,            // cm
    extra_head_travel_for_pressure_cm: 0.25, // cm
};
