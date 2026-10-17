# Reliable Step Timing

Plan for making the plotter stop losing steps.

Fourth draft, 2026-09-15, after a Q&A that pinned down the hardware, the
symptoms and the intent. Earlier drafts designed DMA and microcontroller
backends first; those are now the escalation path at the end of this
document, kept because the facts in them were hard-won. The file name is
historical: nothing here runs in the kernel.

## Decisions and facts from the Q&A

- **Machine:** Raspberry Pi 3 Model B Rev 1.2, Raspbian 12 Bookworm 32-bit,
  kernel 6.6.74 `rpi-v7` without `CONFIG_NO_HZ_FULL` (checked
  2026-09-15), four cores, does nothing but plot and serve the API.
  Drivers are A4988 on a home-made PCB; the two Y drivers have their own
  STEP/DIR pins. Microstep pins stay as they are. No endstops. The driver
  current limits were set once by feel so the motors move reliably; the
  sense resistor values are not recorded.
- **No hardware change for now.** Adding a microcontroller is acceptable
  only if nothing else works, and there is no wish to pick up embedded
  Rust for its own sake.
- **Symptoms, confirmed:** audible and visible hiccups, and the plotter
  physically losing position on X, Y and probably the head, seen
  repeatedly, including on small plots with the current (process-pinned)
  build. Unattended plots are not safe today.
- **Benchmark:** a two-hour plot at max speed returns to origin within a
  step, with no audible hiccups. Small plots must obviously pass first.
- **Speed means pen speed along the path.** Absolute calibration does not
  matter; relative changes in the settings must map to pen speed.
- **Abort:** wanted, need not be instant, no ramp. After an abort the plot
  is considered unrecoverable. Instant stop is the power switch.
- **Pause/continue:** nice to have, as before: tasks that write the rest of
  the plot to disk and continue from it. Used for leaving the house and
  for pen changes. On pause the head lifts, the machine travels to the
  origin, then the motors are disabled.
- **Live position in the Remote page:** nice to have if cheap.
- **Notes:** future, for audible feedback. Not now.
- **Headless plotting from the Pi is the point.** Hardcoded configuration
  for this one plotter is fine. Building with `cargo` on the Pi stays.
- **Effort:** open-ended, but it should not take a giant amount of work.
  Phase 0 only for now; hardware paths get reconsidered only if it fails.

## The problem

Steps are lost physically: the driver receives a pulse the motor does not
execute. The software counts every issued step, so the dead-reckoned
position drifts silently and the plotter can drive into its own frame.
There is no feedback to detect it.

Two families of causes produce this, and they look identical from the
outside:

1. **Software stalls.** The step loop is preempted for longer than a step
   period, the motor stops, then restarts at full speed without
   acceleration and slips. This is what the earlier drafts were about.
2. **Hardware.** An A4988 in thermal shutdown cuts the motor for a few
   hundred milliseconds and resumes: an audible hiccup and lost steps, with
   the software none the wiser. A current limit set too low, a tired power
   supply, a loose coil connection, a slack belt or a dry rod do the same
   more gradually. The machine sat unused for a long time and its
   potentiometers were mentioned as a possible culprit.

The plan therefore starts by instrumenting the loop so the two can be
told apart, then fixes the software causes, then retunes. Everything
else waits until the benchmark passes.

## What the current code actually does

Facts from `server_lib/src/hardware.rs`, `server/src/gpio_executor.rs` and
`server/src/util/system.rs`.

- **Per-axis timing, not path timing.** `Hardware::move_steps` applies one
  delay independently per axis. A 45° move runs 1.41 times faster than
  the setting, a 30° move 1.15 times. Step counts are exact; only speed is
  wrong. A "shape must close" test cannot catch it. A March 2025 attempt
  scaled delays by each axis's Manhattan share, which makes a 45° move run
  at 0.71 times the setting instead; it was reverted. Neither is path
  speed.
- **A hidden 50 to 100 µs per step.** `GpioExecutor::step` calls
  `thread::sleep(200 ns)`. A non-realtime thread gets 50 µs of timer slack
  on every `nanosleep`, so the pulse is really 50 to 100 µs wide, and the
  timestamp is taken after it. The true step period is `delay + about
  60 µs`, the settings were tuned against that, and the estimator ignores
  it. A `SCHED_FIFO` thread has no slack, so any realtime change silently
  speeds the machine up.
- **The last step was already at the torque limit.** Removing the hidden
  sleep at today's settings would take the head-up move from about 290 rpm
  to about 525 rpm on its lead screw. That alone could explain a head that
  loses position.
- **`set_realtime_priority` only sets I/O priority.** `ioprio` does not
  affect CPU scheduling. The June `SCHED_FIFO` attempt was reverted because
  a spinning thread exhausts the realtime budget (95 % of every second) and
  is throttled for the remaining 50 ms: a stall once per second, which is
  exactly a "regular hiccup". The throttle can be switched off.
- **The whole process is pinned to core 2.** `sched_setaffinity(0, ...)`
  pins the calling *thread* (the code comment says process); called from
  `main` before tokio starts, every later thread inherits it. Rocket's
  `/state` handler therefore runs on the plot core and preempts the loop
  under CFS wakeup rules. The plot thread should be pinned alone.
- **`Task::Abort` cannot work as routed.** It goes through the channel the
  plot thread only reads between tasks; today it hits `todo!()`, panics
  the plot thread, and every later `/task` request fails with a 500. No
  client sends it. `Note`, `NoteFreq`, `Move` and `PlotShape` have no
  client either.
- **Position is reported only between tasks**, so the Remote page shows a
  stale location during a plot.
- **Speeds live per project** in `latest_editor.plots`, and the server
  uses `PlotSettings::default()` for `SetHead` and the Remote page's
  moves. A change in what "speed" means reaches every project.
- **`rayon` is declared in `server/Cargo.toml` and unused.**

## Hardware facts and timing budget

From `pins.rs` and `PlotSettings::default()`. The A4988 steps on the rising
edge and needs at least 1 µs high, 1 µs low, and DIR stable 200 ns before
the edge. Columns marked "today" include the hidden sleep at 60 µs.

| Motion      | µm per microstep | Setting  | Nominal rate | Actual today | Motor rpm today |
|-------------|------------------|----------|--------------|--------------|-----------------|
| X, Y draw   | 8.75             | 4.5 cm/s | 5.1 kHz      | 3.4 cm/s     | 75              |
| X, Y travel | 8.75             | 7.0 cm/s | 8.0 kHz      | 4.7 cm/s     | 100             |
| Head down   | 5.0              | 4.0 cm/s | 8.0 kHz      | 2.7 cm/s     | 200             |
| Head up     | 5.0              | 7.0 cm/s | 14 kHz       | 3.8 cm/s     | 290             |

Head and XY never move at the same time. Peak acceleration on the head-up
ramp (1.5 to 7 cm/s over 0.6 mm) is over 600 cm/s²; on XY travel it is
about 45 cm/s². The head is the axis most likely to be over its limit.

## Phase 0: measure, isolate, fix path speed, retune

Order matters: instrument first, so every later step is judged by
numbers rather than by ear.

### 0.1 Instrument the existing loop

In `GpioExecutor` (and mirrored in the estimator so it stays exact):

- In `wait_axis`, record lateness = `now - delay_until` when the wait
  exits. Keep max, a count over 500 µs, a count over 5 ms, and the sum.
- Record the actual period between consecutive steps on each axis and the
  commanded delay, so the mean overhead per step (the hidden sleep) is a
  measured number.
- At the end of every task print one line to the journal:
  `steps, mean_overhead_us, max_late_us, late_over_500us, late_over_5ms`.
- Optional, one evening: expose the same numbers in `ServerState` so the
  Remote page shows them.

Cost: an afternoon. This is the tool that separates cause 1 from cause 2.

### 0.2 Reproduce and attribute

Run each of these with the instrumented build and note the log line and
what the machine did:

- **Square, 20 cm sides, 20 laps, pen up**, at today's settings. Then the
  same pen down. Mark the start; check the return.
- **Head only:** 50 up/down cycles. Check that the pen still touches the
  paper the same way (a head that drifts either stops reaching the paper
  or presses harder).
- **The same square at half speed** (halve `max` in a copy of the
  settings). If the machine still drifts at half speed with a clean log,
  the cause is hardware.
- **Load:** the square with the editor Remote page open and `stress -c 4`
  running on the Pi.

Read the results:

- Position lost **and** the log shows stalls over a few hundred µs at the
  time: software. Continue with 0.3.
- Position lost **and** the log is clean: hardware. Do the hardware checks
  before anything else in software.
- Log shows stalls but position holds: the machine tolerates them; still
  do 0.3, but the retune matters less.

Hardware checks, in order of likelihood and cost:

- **Driver temperature.** Touch or IR-thermometer each A4988 after a few
  minutes of plotting; run the square again with a fan pointed at the
  board. If the fan fixes it, it was thermal shutdown.
- **Current limit.** Read the sense resistor marking on the PCB (`R100`
  is 0.1 Ω, `R050` is 0.05 Ω), measure Vref at each potentiometer wiper,
  then `I_limit = Vref / (8 × R_sense)`. Compare with the motors' rated
  current. The potentiometers were set by feel once and can drift or be
  knocked; too low loses torque, too high overheats. Record the four
  values in this file.
- **Power supply** voltage under load, at the drivers, not at the plug.
- **Coil connections.** Wiggle each motor connector while the motor holds;
  a loose pin makes an unmistakable noise and loses steps instantly.
- **Mechanics** with motors disabled: move each axis by hand through its
  range and feel for binding; check belt tension; clean and oil the rods.
- **Microstep pins** actually at the configured levels (a multimeter).

### 0.3 Isolate the plot thread

The stall sources are other threads on the plot core, IRQs, and the
kernel tick. Remove the first two; the tick stays, because this kernel
has no `CONFIG_NO_HZ_FULL`, and it costs a few µs per tick against a
125 µs step period.

- Kernel cmdline (`/boot/firmware/cmdline.txt`):
  `isolcpus=2 irqaffinity=0,1,3`. Nothing else: this kernel has neither
  `CONFIG_NO_HZ_FULL` nor `CONFIG_RCU_NOCB_CPU` (checked 2026-09-15), so
  `nohz_full` and `rcu_nocbs` would be ignored. `isolcpus` removes every
  other task from core 2. USB and network on a Pi 3 interrupt on core 0,
  which is why the plot core is 2.
- `/etc/sysctl.d/99-plottery.conf`: `kernel.sched_rt_runtime_us = -1`.
- Frequency scaling: the server writes `performance` to the cpufreq
  governor at startup (root is available and the policy covers all cores).
- In the plot thread, at start, in this order: pin *this thread* to
  core 2; verify `/sys/devices/system/cpu/isolated` lists core 2; only if
  it does, set `SCHED_FIFO` priority 50; otherwise log a warning and stay
  on the normal scheduler. Never set `SCHED_FIFO` on a shared core: a
  spinning realtime thread there freezes the machine.
- Remove `pin_process_to_plot_core` from `main`. `isolcpus` keeps tokio and
  Rocket off core 2 by itself. Drop the `ioprio` call; it does nothing
  useful.
- Replace both `thread::sleep(200 ns)` calls with a 2 µs busy-wait on
  `Instant`. This removes the hidden overhead, which is why 0.5 follows.

Expected result: stalls in the tens of µs, well under a step period. The
instrumented log confirms or refutes it.

### 0.4 Path speed

Make "speed" mean pen speed along the path, as intended, with the
smallest change that does it. In `Hardware::move_steps` the Bresenham-style
interleave already spaces the Manhattan steps uniformly along the segment,
so the k-th step of N lies at path distance `L × k / N`. The fix:

- Scale the delay by `L / N = sqrt(dx² + dy²) / (|dx| + |dy|)` for the
  segment (between 0.707 and 1).
- Wait relative to the last step on *either* axis, not per axis. Add
  `wait_xy(delay_nanos)` to `HardwareExecutor`, which waits from
  `max(last_step[X], last_step[Y])`; `wait_axis` stays for the head and
  notes. Both the GPIO executor and the estimator implement it in three
  lines.

Path speed is then the setting at every angle, and each motor's rate is at
or below the axis-aligned rate. The estimator, sharing `move_steps`, stays
exact. A `hardware_test.rs` with the estimator executor checks that a 45°
move and an axis-aligned move of equal length take equal time, and that
step counts are unchanged.

### 0.5 Retune speeds

After 0.3 and 0.4 the same numbers mean faster physical motion. Start
from the "actual today" column, which is what the machine was doing,
then find the real limits:

- Set the defaults to the measured actual speeds from 0.1 so the first run
  matches today's physical behaviour (about draw 3.4, travel 4.7, head
  down 2.7, head up 3.8 cm/s if the overhead is 60 µs; use the measured
  value).
- Per axis, pen up: repeat 20 cm back-and-forth moves twenty times at a
  speed, return to origin, check the mark. Raise the speed until it fails;
  set `max` to 70 % of that.
- Pen down, same procedure for the draw speed (pen friction lowers it).
- Head: 50 cycles at a speed; check contact; raise until it fails; 70 %.
- Keep the acceleration distances unless corners skip; then lengthen them.
- Write the results into `PlotSettings::default()`. Projects that saved
  their own settings keep their numbers; since many use the defaults, a
  note in the release is enough. Speed presets in the editor can come
  later.

### 0.6 Abort

Simple, because after an abort the plot is unrecoverable by definition:

- `ManagedState` gets `abort: Arc<AtomicBool>`. The `/task` handler stores
  `true` for `Task::Abort` and does not enqueue it.
- The task loop clears the flag when a task starts. `Hardware::move_steps`
  and `set_head` check it once per step and return early; `plot_layer`,
  `plot_shape` and `travel_to` check `hardware.aborted()` at their loop
  heads and return.
- After an aborted task the loop lifts the head, disables motors, drains
  the channel with `try_recv`, and reports idle. Position is left as
  dead-reckoned; the user re-origins.
- The Remote page gets an Abort button.

### 0.7 Crash safety

- `plottery.service`: `ExecStopPost=` running `pinctrl set <pin> op dh` for
  the four enable pins (14, 2, 8, 21). It runs even when the process was
  killed, so a crash no longer leaves motors energised. `raspi-gpio` if
  `pinctrl` is missing.
- Root stays. rppal itself only needs the `gpio` group, but `SCHED_FIFO`
  without root needs an rlimit in the unit and the governor write needs
  root; not worth changing.

### Acceptance

- The 0.2 test set passes with a clean log and no drift.
- A full-page plot with the Remote page polling and `stress -c 4` running
  shows no stall over 500 µs.
- The two-hour benchmark plot returns to origin within a step.

## Phase 1: after the benchmark passes

Cheap additions on the existing architecture, each independent:

- **Live position.** Call `sync_server_state` between shapes in
  `plot_layer` (the machine is at minimum speed there, so the mutex is
  harmless). Long single shapes update at their end; good enough.
- **Pause / continue.** `Task::Pause` sets a flag checked between shapes.
  On pause: serialise the remaining shapes plus sample and plot settings
  to a fixed path, lift the head, travel to the origin, disable motors,
  report paused. `Task::Continue` loads the file, deletes it, enables
  motors and plots it; the travel to the first remaining shape is the
  normal one. Parking at the origin is what makes this robust: a server
  restart in between resets the position to zero, which is where the
  machine already is, so nothing needs re-origining unless the carriage
  was moved by hand.
- **Progress and ETA.** With the estimator exact after 0.4, estimate the
  remaining shapes on pause/progress boundaries and report a percentage.
- **Speed presets** in the editor's plot settings.
- **Notes** when they are wanted: the current `play_freq` already works
  through the same loop.
- Remove `rayon` from `server/Cargo.toml`.

## If Phase 0 is not enough: escalation path

Kept from earlier drafts, condensed. Only read if the log shows stalls
that isolation cannot remove, or if lost steps persist with a clean log
after the hardware checks (in which case this section does not help
either).

### Why a hardware backend needs a refactor first

Any hardware-timed backend consumes an explicit schedule. That means:

- A `StepEvent { t_ns, axis, forward }` stream produced lazily by the
  planner, sorted by time, never collected whole.
- A closed-form time parameterisation per segment. With speed linear in
  distance from `v0` to `v1` over length `L` (what `AccellerationPath`
  assumes today, `v_min > 0`): `t(s) = s / v0` if `v1 == v0`, else
  `L / (v1 - v0) × ln(v(s) / v0)`. X step `k` fires at
  `s = L × (k - ½) / |dx|`, Y likewise, merged by time; step sums are exact.
- A pull-model `Stepper::run(events, progress)` trait with `SpinStepper`,
  `PigpioStepper` or `McuStepper` behind it. Heavy planning (sampling,
  `AccellerationPath`) must happen where the machine is at `v_min`, which
  today is every shape boundary.
- Estimation becomes the sum of closed-form segment durations.
- Abort becomes a generation counter set by the HTTP handler; the planner
  ends its iterator when it changes. For a wave-based backend the stop
  lands at a wave boundary, which keeps position exact.

Constant acceleration (`v² = v0² + 2as`, `t = (v - v0) / a`) would be a
better profile: today's linear-in-distance ramp peaks at about twice the
acceleration of a trapezoid with the same ramp distance, right where
torque is lowest. It needs `AccellerationPath` to plan in `v²` space and
is its own change.

### pigpio DMA waves (works on the Pi 3, no hardware change)

`pigpiod` plays lists of `(gpio_on, gpio_off, delay_µs)` pulses from RAM
through the SoC's DMA engine, clocked by the PWM peripheral. Facts that
shape the design:

- Sample rate defaults to 5 µs; use `pigpiod -l -s 2`. Delays are rounded
  to the grid; round absolute times, then difference, to avoid drift.
- Capacity is a shared pool of DMA control blocks (25 016 on current
  pigpio; query `wave_get_max_cbs`). A step costs two pulses (rising and
  falling edge) at about 2 CBs each. Waves hold their CBs until
  `wave_delete`.
- One wave transmits and one can be queued behind it with
  `wave_send_using_mode(ONE_SHOT_SYNC)`; a queued wave cannot be safely
  un-queued, so abort latency is two waves. Size waves at 125 ms, never
  under 60 ms except at the end; two in existence at most.
- The last delay of a wave reaches to the first event of the next; the
  next wave starts with no leading delay. Each `wave_add_generic` call
  starts at the wave's time zero; a second call needs a leading
  delay-only pulse.
- DIR changes go into the pulse before the step edge. X and Head DIR are
  inverted relative to Y, as in `GpioExecutor::set_dir` today.
- No pulse-level position query: an emergency `wave_tx_stop` loses
  position within the current wave. Underrun (producer too slow) pauses
  motion but loses no pulses; count it, zero is the acceptance criterion.
- rppal and pigpiod can share the SET/CLR registers safely but not mode
  changes; route every pin through pigpiod and drop rppal in that build.
- Service: `Requires=pigpiod.service`, `After=pigpiod.service`, and
  `ExecStopPost=/usr/bin/pigs wvhlt w 14 1 w 2 1 w 8 1 w 21 1`.
- The producer thread sleeps between polls, so it can be `SCHED_FIFO`
  without a throttling concern. Talk to the daemon over its socket
  protocol (16-byte little-endian header plus extension; `pigpio.py` is the
  reference); the crates.io bindings are unmaintained. Linking `libpigpio`
  in-process was rejected: a crashed process leaves DMA reading reused
  memory, the daemon finishes the wave cleanly.

### RP2040 / Pico over USB (hardware change; last resort)

The host plans, the Pico pulses from a 1 µs timer alarm. A 4-byte record
per event (`delay_us: u16, step_mask: u8, dir_mask: u8`) over USB CDC,
credit-based flow control from 10 ms status frames carrying consumed
count, buffer space and executed step counts. Position stays exact even
on an emergency stop, underrun is a flagged pause, and endstops or an
e-stop would be MCU inputs. Firmware in Rust with `embassy-rp` is about
500 lines; a shared `no_std` crate holds the wire types. All driver
signals move to the Pico, the Pi no longer needs GPIO or root, and the
server could also run on a laptop. This path only makes sense if pigpio
proves insufficient, and it was explicitly deferred.

## Testing

### Unit tests

- `hardware_test.rs` (estimator executor): equal-length moves at 0°, 30°,
  45° and 60° take equal time after 0.4; step counts match `(dx, dy)`;
  zero-length move takes zero time; the abort flag stops a move within one
  step and leaves counts consistent with the steps issued.
- `estimation_test.rs`: the estimate for a fixture layer equals the sum of
  its segment times to within one step period, and shrinks by the
  measured overhead after 0.3.
- `task_handler_test.rs`: an abort during a task drains queued tasks; an
  abort while idle does not affect the next task.
- Phase 1: pause writes the remaining shapes and continue plots exactly
  those; the file is deleted afterwards.

### Hardware checklist

Run the 0.2 set after every change and keep the log lines in this file:

| Date | Change | Pattern | Machine | Log line (key fields) |
|------|--------|---------|---------|-----------------------|
| 2026-09-15 | baseline (0.1), run 1 | square-down 20 cm, 2 laps | X motor unpowered during lap 1, then fine | steps=186556 span_s=48.2 pulse_mean_us=61.9 pulse_max_us=11676 stalls_over_500us=1 stalls_over_5ms=0 spin_preempts_over_500us=1 |
| 2026-09-15 | baseline (0.1), run 2 | square-down 20 cm, 2 laps | all motors unpowered for seconds mid-plot, then resumed | steps=186556 span_s=48.1 pulse_mean_us=61.8 pulse_max_us=712 stalls_over_500us=0 stalls_over_5ms=0 spin_preempts_over_500us=0 |
| 2026-10-17 | ioprio call removed | square-up 20 cm, 2 laps (8 MoveTo) | smooth, back on the mark | per side steps=22870 span_s=4.8 pulse_mean_us=61.9; side 1: spin_preempts_over_500us=3 (Rocket still receiving tasks); sides 6-8 gap_max_us 63/167/1089, stalls_over_500us=1 total |
| 2026-10-17 | ioprio call removed | square-down 20 cm, 2 laps | smooth, minute ticks audible, back on the mark, single line | steps=186556 span_s=48.2 pulse_mean_us=61.8 pulse_max_us=1044 gap_max_us=493 stalls_over_500us=0 stalls_over_5ms=0 spin_preempts_over_500us=1 |
| 2026-10-17 | ioprio call removed | squares-down 10 cm ×20, shifted 0.25 cm | VMOT 8.27 V idle, 7.8 V motors holding (A4988 minimum is 8 V); run result see below | steps=1008552 span_s=260.9 pulse_mean_us=61.9 pulse_max_us=1314 gap_max_us=1180 stalls_over_500us=8 stalls_over_5ms=0 spin_preempts_over_500us=9 |
| 2026-10-17 | 0.3 (isolcpus, FIFO plot thread, busy-wait pulse, rescaled defaults) | square-up 20 cm, 2 laps | see text | per side steps=22870 span_s=4.9 pulse_mean_us=2.6 pulse_max_us≤11 gap_max_us≤11 stalls_over_500us=0 spin_preempt_max_us≤32 spin_preempts_over_500us=0 |

Reading of 2026-09-15: run 2 froze visibly with a clean log and the same
48 s span as run 1, so the software stepped continuously while the motors
had no power. That is the motor supply (protection trip or a bad
connection), not timing. The 62 µs hidden sleep and the 3.3 cm/s actual
speed match the predictions above. The 11.7 ms pulse outlier in run 1 is
the `sleep(200 ns)` misbehaving and is what 0.3 removes.

Reading of 2026-10-17: same patterns, same step counts, position held both
times at full speed and the log is clean apart from sub-millisecond gaps on
the shared core. The September power dropouts did not reproduce, so they
are intermittent: still hardware, still unfound. The Pi was cool (52 °C)
and idle; all server threads sat on core 2 as described above. Sense
resistors on the A4988 modules read `R100`, so `I_limit = Vref × 1.25 A`.

Motor supply measured 2026-10-17: the brick is rated **8 V 3 A**. VMOT reads
8.27 V idle, 7.8 V with the motors holding, 7.82 to 7.9 V during the
20-square plot; the plot itself was perfect. The A4988 is specified for
8 to 35 V, so the drivers run below their minimum supply the whole time,
and four drivers on 3 A leave no margin at this voltage. A switching
supply in overload goes into hiccup mode (off for about a second, then
retry), which is exactly run 2 of 2026-09-15. Fix: a 24 V supply with at
least 4 A (12 V as a minimum); check the VMOT capacitor rating on the PCB
first (needs 35 V or more for 24 V). The higher voltage also raises the
torque available at speed, which the retune in 0.5 depends on; measure
Vref and set the current limits afterwards.

Done on the Pi 2026-10-17 (0.3 system side): `isolcpus=2 irqaffinity=0,1,3`
appended to `/boot/firmware/cmdline.txt` (backup at
`cmdline.txt.bak-2026-10-17`), `/etc/sysctl.d/99-plottery.conf` with
`kernel.sched_rt_runtime_us = -1`, rebooted. The server sets the
`performance` governor itself, pins only the plot thread to core 2, and
switches it to `SCHED_FIFO 50` only when `/sys/devices/system/cpu/isolated`
lists core 2 and the realtime budget is unlimited; otherwise it logs a
warning and stays on the normal scheduler. The `sleep(200 ns)` calls are
replaced by a 2 µs busy-wait, and `PlotSettings::default()` now holds the
speeds the machine physically ran at before (draw 0.39-3.41, travel
0.58-4.68, head down 0.47-2.68, head up 1.27-3.75 cm/s).

Then the full-page plot with load, then the two-hour benchmark.

## References

- A4988 datasheet, timing and current-limit sections:
  <https://www.pololu.com/file/0J450/A4988.pdf>
- Linux CPU isolation: `Documentation/admin-guide/kernel-parameters.txt`
  (`isolcpus`, `irqaffinity`) and
  `Documentation/scheduler/sched-rt-group.rst` (`sched_rt_runtime_us`)
- `pinctrl` / `raspi-gpio`: <https://github.com/raspberrypi/utils>
- systemd `ExecStopPost`: `man systemd.service`
- pigpio waves: <https://abyz.me.uk/rpi/pigpio/cif.html#gpioWaveAddGeneric>;
  socket protocol: <https://abyz.me.uk/rpi/pigpio/sif.html>;
  daemon options: <https://abyz.me.uk/rpi/pigpio/pigpiod.html>
- `embassy-rp`: <https://embassy.dev>
- Klipper architecture (host planner, MCU stepper):
  <https://www.klipper3d.org/Overview.html>
