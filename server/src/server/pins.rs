// PINS
pub static DIR_PINS: [i32; 4] = [18, 4, 11, 16];
pub static STEP_PINS: [i32; 4] = [15, 3, 7, 20];
pub static ENABLE_PINS: [i32; 4] = [14, 2, 8, 21];

pub static MICSTEP_1_PINS: [i32; 4] = [22, 10, 12, 13];
pub static MICSTEP_2_PINS: [i32; 4] = [27, 9, 6, 19];
pub static MICSTEP_3_PINS: [i32; 4] = [17, 25, 5, 26];

// DIRECTIONS
pub static DIR_VAL: [[char; 2]; 4] = [['1', '0'], ['1', '0'], ['1', '0'], ['0', '1']];

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
pub static MICSTEP_VALS: [[char; 3]; 4] = [
    ['1', '1', '1'],
    ['1', '1', '1'],
    ['1', '1', '1'],
    ['1', '1', '0'],
];
pub static MICSTEP_AXES: i32 = 16;
pub static MICSTEP_HEAD: i32 = 8;

// STEPS
pub static DIST_PER_STEP_AXIS_CM: f32 = 0.0139935599999 / MICSTEP_AXES as f32; // cm
pub static DIST_PER_STEP_HEAD_CM: f32 = 0.8 / (200.0 * MICSTEP_HEAD as f32); // cm (200 steps per revolution, 8mm travel per revolution)

pub static HEAD_TRAVEL_TO_TOUCH_CM: f32 = 0.6; // cm
pub static MAX_EXTRA_HEAD_TRAVEL_CM: f32 = 0.25; // cm
