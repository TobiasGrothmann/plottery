static DIR_PINS: [i32; 4] = [18, 4, 11, 16];
static STEP_PINS: [i32; 4] = [15, 3, 7, 20];
static ENABLE_PINS: [i32; 4] = [14, 2, 8, 21];

static MS1_PINS: [i32; 4] = [22, 10, 12, 13];
static MS2_PINS: [i32; 4] = [27, 9, 6, 19];
static MS3_PINS: [i32; 4] = [17, 25, 5, 26];

static DIR_VAL: [[char; 2]; 4] = [['1', '0'], ['1', '0'], ['1', '0'], ['0', '1']];

/*
    MS1		MS2		MS3		Microstep Resolution
    --------------------------------------------
    Low		Low		Low		Full step
    High	Low		Low		Half step
    Low		High	Low		Quarter step
    High	High	Low		Eighth step
    High	High	High	Sixteenth step
*/
static MS_VAL: [[char; 3]; 4] = [
    ['1', '1', '1'],
    ['1', '1', '1'],
    ['1', '1', '1'],
    ['1', '1', '0'],
];
static MS: i32 = 16;
static HEAD_MS: i32 = 8;

static DIST_PER_STEP_AXIS: f32 = 0.0139935599999 / MS as f32; // cm
static DIST_PER_STEP_HEAD: f32 = 0.8 / (200.0 * HEAD_MS as f32); // cm (200 steps per revolution, 8mm travel per revolution)

static HEAD_TRAVEL_TO_TOUCH: f32 = 0.6; // cm
static MAX_EXTRA_HEAD_TRAVEL: f32 = 0.25; // cm
