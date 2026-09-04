# Hardware-Timed Step Generation

Plan for taking step-pulse timing away from the Linux scheduler.

Third draft, 2026-09-15. Changes from the second draft: abort is a
planner-level deceleration ramp driven by a generation counter; pigpio
abort latency and wave sizing corrected; the RP2040 backend is designed
rather than sketched and becomes the recommended hardware path; estimation
is O(segments); a constant-acceleration profile is captured as a later
change; Phase 0 owns the speed retune; crash disarm, a planning invariant
and measurement tooling added.

Nothing in this plan runs in the kernel. The file name is historical.

## TL;DR

1. **Phase 0 first.** Isolate a core, run the plot thread `SCHED_FIFO` with
   throttling off, replace the sleep-based pulse with a busy-wait, and
   measure with a logic analyzer. No new dependencies. It may be enough.
2. **Do the refactor regardless (Phase 1).** The planner emits `StepEvent`s
   and a `Stepper` pulls them. This fixes the diagonal-speed bug, makes
   abort work, makes estimation exact and cheap, and is what every hardware
   backend needs.
3. **If Phase 0 is not enough: RP2040 (Phase 2a).** A Pico over USB does
   the pulsing. It works on any Pi or on no Pi, keeps position exact even on
   an emergency stop, and is where endstops and e-stop will live.
4. **pigpio (Phase 2b) only if** the machine is a Pi 4 or older *and* a
   hardware change is unacceptable. It is a dead-end platform with a daemon,
   a shared DMA buffer to juggle, and up to two waves of abort latency.

## Motivation

The server generates step pulses from a userspace busy-wait loop
(`GpioExecutor::wait_axis` then `step`). It works as long as the plot
thread is never preempted for longer than one step period (125 µs at
travel speed). Any longer stall and the motor stops, then restarts at full
speed without acceleration. That is where steps get lost: the software
still counts the step as issued, the motor did not make it, and the
dead-reckoned position is silently wrong for the rest of the plot.

Two things follow:

- The problem is *stalls*, not microsecond jitter. Jitter of a few µs on a
  125 µs period is inaudible and harmless. Stalls of milliseconds are the
  enemy.
- The fix has to either (a) make stalls rare and short enough to never
  exceed the motor's tolerance, or (b) move pulse timing to hardware that
  cannot stall. Phase 0 is (a). The RP2040 and pigpio are (b).

## What the current code actually does

Facts from `server_lib/src/hardware.rs`, `server/src/gpio_executor.rs` and
`server/src/util/system.rs` that the first draft got wrong or missed.

- **Per-axis timing, not path timing.** `Hardware::move_steps` computes one
  delay from the speed setting and applies it independently per axis. On a
  45° segment each motor runs at the nominal rate, so the pen moves √2
  times faster than the setting; on a 30° segment 1.15 times. Step *counts*
  are exact, so shapes still close; only speed depends on angle. A "shape
  must close" test can never catch this.
- **A hidden 50 to 100 µs per step.** `GpioExecutor::step` calls
  `thread::sleep(200 ns)`. On a non-realtime thread Linux adds 50 µs of
  timer slack to every `nanosleep`, so the pulse is really 50 to 100 µs
  wide, and the timestamp is taken *after* it. The real step period is
  `delay + roughly 60 µs`. The speed values in `PlotSettings::default()`
  were therefore tuned against a slower machine than they describe, and the
  time estimator is optimistic by the same amount. A `SCHED_FIFO` thread
  gets zero timer slack, so **Phase 0 alone changes the machine's speed**;
  the retune belongs there.
- **`set_realtime_priority` sets I/O priority only.** `ioprio` does not
  touch CPU scheduling. The earlier `SCHED_FIFO` attempt was removed because
  a busy-wait thread burns the RT budget (`sched_rt_runtime_us`) and gets
  throttled. The fix is to disable the throttle, not to give up
  `SCHED_FIFO`.
- **The whole process is pinned to core 2**, including Rocket and every
  tokio worker. That puts the `/state` handler on the same core as the busy
  loop, where CFS wakeup preemption lets it interrupt the loop. Pin the plot
  thread alone and keep everything else *off* that core.
- **`Task::Abort` cannot work as routed.** It travels through the same mpsc
  channel the plot thread reads only between tasks, so it would be received
  after the plot ends. Today it hits `todo!()`, which panics the plot
  thread and drops the channel receiver; every later `/task` request then
  fails with a 500.
- **`estimation.rs` depends on `HardwareExecutor`.** The editor calls
  `estimate_plot_layer_duration`. Deleting the trait breaks the editor
  unless estimation is reimplemented.
- **Every motion goes through the same path.** `Plot`, `PlotShape`, `Move`,
  `MoveTo`, `SetHead`, `Note` and `NoteFreq` all end in `wait_axis` and
  `step`. All of them must become event streams; there is no "just the step
  pins" scope.
- **Position is reported only between tasks.** `sync_server_state` runs
  before and after a task, so the editor's Remote page shows a stale
  location during a plot.

## Hardware facts and timing budget

From `pins.rs` and `PlotSettings::default()`. The microstep table is the
A4988's (MS1 to MS3, 1/16 = HHH). The A4988 steps on the rising edge and
needs at least 1 µs high, 1 µs low, and DIR stable 200 ns before the edge.
Its inputs are 3.3 V tolerant, so a Pi and a Pico drive it the same way.

| Motion      | Distance per microstep | Max speed | Max step rate | Min period |
|-------------|------------------------|-----------|---------------|------------|
| X, Y draw   | 8.75 µm                | 4.5 cm/s  | 5.1 kHz       | 194 µs     |
| X, Y travel | 8.75 µm                | 7.0 cm/s  | 8.0 kHz       | 125 µs     |
| Head down   | 5.0 µm                 | 4.0 cm/s  | 8.0 kHz       | 125 µs     |
| Head up     | 5.0 µm                 | 7.0 cm/s  | 14 kHz        | 71 µs      |

Worst-case event rate is a head lift (14 k events/s) or a 45° travel
(2 × 5.7 k = 11.3 k events/s). Head and XY never move at the same time.
Every backend below has an order of magnitude of headroom.

## Options

### 0. Isolated core + SCHED_FIFO (software, cheap)

Keep the busy-wait stepper but stop the kernel from scheduling anything
else on its core.

- Kernel cmdline (`/boot/firmware/cmdline.txt` on Bookworm):
  `isolcpus=2 nohz_full=2 rcu_nocbs=2 irqaffinity=0,1,3`. Core 2 then gets
  no other tasks, no periodic tick while a single task runs, no RCU
  callbacks and no IRQs. `nohz_full` needs `CONFIG_NO_HZ_FULL`; if the Pi
  kernel lacks it, `isolcpus` alone still removes everything but a
  100 to 250 Hz tick of a few µs.
- `kernel.sched_rt_runtime_us=-1` in `/etc/sysctl.d/`, then `SCHED_FIFO` on
  the plot thread only, pinned to core 2 with `sched_setaffinity(tid)`. Do
  not pin the process; `isolcpus` already keeps every other thread off
  core 2.
- **Guard:** only switch to `SCHED_FIFO` after confirming core 2 is listed
  in `/sys/devices/system/cpu/isolated`. A spinning realtime thread on a
  shared core freezes the machine.
- CPU frequency governor `performance` (or `force_turbo=1`).
- Step pulse: 2 µs by busy-wait, not `thread::sleep`.
- Absolute schedule (`t_start + event.t`). If the loop falls behind by more
  than 500 µs, re-base `t_start` instead of bursting the overdue steps.
- Log per task: max and p99 lateness and the count of stalls over 500 µs.

Expected result: worst-case stalls in the tens of µs, well under a step
period. Needs a multi-core Pi (Pi 2 and up). Works on a Pi 5. A
`PREEMPT_RT` kernel would allow a sleeping loop instead of a spinning one,
but it is a kernel swap for the same result and is not needed.

### 1. RP2040 / Pico over USB (hardware-timed, any host)

The Klipper and GRBL model: the host plans, an MCU pulses. A Raspberry Pi
Pico (about 5 EUR) receives a compact record stream over USB CDC and steps
from a hardware timer alarm with 1 µs resolution and sub-µs jitter.

- Independent of the Pi model, and of the Pi: the server can run on the
  laptop with the Pico on USB, and the Pi becomes optional.
- Position is exact in every case, including an emergency stop, because the
  MCU counts what it actually emitted and reports it.
- Underrun is a pause with a flag, never a lost pulse.
- All driver signals move to the Pico, so the server needs no root and no
  GPIO library; the `raspi` feature disappears from the hardware path.
- Endstops and a hardware e-stop are MCU inputs, not a Pi redesign.
- Cost: a board, a cable, a small firmware (Rust, `embassy-rp`), and a
  protocol with flow control. Designed in [RP2040 backend](#rp2040-backend).

### 2. pigpio DMA waveforms (hardware-timed, Pi 4 and older)

The BCM2835/6/7 and BCM2711 can DMA GPIO set/clear masks from RAM at a
fixed sample clock. `pigpiod` owns that machinery and exposes "waves":
lists of `(gpio_on_mask, gpio_off_mask, delay_µs)` pulses that the DMA
engine plays out with no CPU involvement.

- **No Pi 5.** Gate on `/proc/device-tree/model`.
- **Sample rate defaults to 5 µs**, selectable 1/2/4/5/8/10 µs with
  `pigpiod -s`. Finer costs CPU. Use 2 µs.
- **Capacity is a shared DMA control-block pool.** Every created wave holds
  its control blocks (CBs) until `wave_delete`. Query `wave_get_max_cbs`
  at startup (25 016 on current pigpio: 4 blocks × 53 pages × 118 CBs);
  each pulse costs 2 CBs (mask write plus delay), so a step event costs 4.
- **Queue depth is one.** One wave transmitting, one queued behind it.
- **No pulse-level position query**, so an emergency stop loses position to
  within one wave.
- **Abort latency is two waves**, because a queued wave cannot be safely
  un-queued.
- **Root and a daemon** on localhost TCP 8888; hand-rolled socket client
  (about 200 lines, no native dependency).

Designed in [pigpio backend](#pigpio-backend). Linking `libpigpio`
in-process instead of using the daemon was considered and rejected: a
crashed process leaves the DMA engine reading memory the kernel is reusing,
while the daemon finishes the wave cleanly.

### 3. RP1 PIO (Pi 5 only)

The Pi 5's RP1 has PIO state machines reachable through `piolib`. Same
guarantees as pigpio in principle, no Rust bindings, little prior art.
Strictly dominated by option 1 for this project; listed so it is not
re-researched.

### Decision rule

- Phase 0 and Phase 1 happen in every case.
- If the Phase 0 numbers show stalls within a factor of two of a step
  period under the load test, add a hardware backend.
- Hardware backend: RP2040. pigpio only if the machine is a Pi 4 or older
  and a hardware change is off the table.
- The backend is chosen at runtime, never at compile time, so one binary
  can be switched back.

## Design (backend-independent)

### Two positions

The planner runs ahead of the machine. Keep the two positions distinct.

- **Planned position**: integer step counters owned by the planner while it
  generates events. Segment targets are rounded onto the step grid from the
  planned position, as `Hardware::move_to` does today, so there is no
  accumulation error.
- **Executed position**: integer step counters advanced only for events the
  backend has actually emitted (spin: per step; RP2040: from the MCU's
  status frames; pigpio: per fully transmitted wave). This is what
  `ServerState.location` reports, and it is updated *during* a plot.

### `StepEvent`

```rust
pub struct StepEvent {
    /// Nanoseconds since the start of the current run.
    pub t_ns: u64,
    pub axis: Axis, // X, Y, Head
    pub forward: bool,
}
```

Streams are lazy iterators sorted by `t_ns`. A full-page plot is millions
of events; never collect a whole run into a `Vec`.

### Planner: time-parameterised segments (closed form)

`AccellerationPath` stays as it is: it turns a polyline into waypoints with
a speed fraction at each. What changes is how a segment between two
waypoints becomes steps.

For a segment of length `L` with speed linear in distance from `v0` to
`v1` (`v = v_min + (v_max - v_min) * fraction`; `v_min > 0`, so `v > 0`):

```
v(s) = v0 + (v1 - v0) * s / L
t(s) = s / v0                              if v1 == v0
     = L / (v1 - v0) * ln(v(s) / v0)       otherwise
```

Given integer step deltas `(dx, dy)`, X step `k` (1 ≤ k ≤ |dx|) fires at
`s = L * (k - 0.5) / |dx|`, Y likewise; merge the two sorted streams by
time. Speed is now along the path, so a 45° segment runs at the set speed
and each motor at or below its axis-aligned rate. Summing the events
reproduces `(dx, dy)` exactly. Linear-in-distance speed is kept in Phase 1
because it is what the acceleration planner assumes today; see
[Constant-acceleration profile](#constant-acceleration-profile) for the
better profile and why it is a separate change.

The same function covers the head (one axis, three linear pieces for the
accelerate/cruise/decelerate profile `set_head` uses today) and notes
(constant period, direction flipping every event). `SpeedDelayHandler`
goes away; `SpeedRange` gains `speed_at(fraction) -> cm/s`.

`plan_layer`, `plan_shape`, `plan_travel`, `plan_head` and `plan_note`
return `impl Iterator<Item = StepEvent>` and advance a `PlannerState`
(planned position, planned head state, running time offset, current
velocity). `plot_execution.rs` becomes these functions and nothing else.

### Invariant: heavy planning happens at standstill

The stepper pulls events, so planning cost lands inline with stepping.
Per-event cost is well under a µs against 71 µs or more between events;
that is fine. Sampling a shape and building its `AccellerationPath` is
not, and must only happen where speed is zero. Today that holds because
every path starts and ends at speed zero and travel moves do too. Any
future optimisation that overlaps travel with the next shape's planning,
or blends corners across shapes, must keep this invariant or buffer ahead.
The RP2040 and pigpio backends buffer 60 ms or more and do not care; the
spin backend does.

### `Stepper` trait (pull model)

```rust
pub trait Stepper {
    /// Drive the machine through `events` until they run out. Calls
    /// `progress` with the executed position at least a few times per
    /// second. Returns what was actually executed.
    fn run(
        &mut self,
        events: &mut dyn Iterator<Item = StepEvent>,
        progress: &mut dyn FnMut(Progress),
    ) -> Result<RunReport, StepperError>;

    fn set_enabled(&mut self, enabled: bool);

    /// Emergency stop: stop output now. Reports executed steps if the
    /// backend knows them (spin and RP2040: exact; pigpio: unknown within
    /// the current wave).
    fn halt(&mut self) -> HaltReport;
}

pub struct RunReport {
    pub executed_steps: [i64; 3],
    pub underruns: u32,
    pub max_lateness_ns: Option<u64>, // spin only
}
```

Pull, not push: the stepper drains the iterator at its own pace, so there
is no second thread, no bounded channel, and back-pressure is free. The
trait has no notion of abort; see below.

Implementations: `SpinStepper` (today's loop with the Phase 0 changes),
`McuStepper`, `PigpioStepper`, and a `LogStepper` for development off the
Pi.

### Abort: a generation counter and a deceleration ramp

Abort is a planner concern, not a stepper concern.

- `ManagedState` holds an `abort_generation: AtomicU64`. The `/task`
  handler treats `Task::Abort` specially: it increments the counter and
  never enqueues it. Every other task is enqueued as
  `QueuedTask { generation: counter.load(), task }`.
- The task loop skips any queued task whose generation is older than the
  current counter. Tasks sent *after* an abort therefore run; tasks sent
  before it are dropped; an abort received while idle harms nothing. No
  flag to clear, no drain window.
- The running planner remembers the generation it started under and checks
  the counter once per event (a relaxed atomic load). When it changes, the
  planner stops following the path and emits a **deceleration ramp**: from
  the current position along the current segment direction, from the
  current speed down to `v_min`, over at most the segment's remaining
  length and at most the profile's acceleration distance, rounded onto the
  step grid. Then the iterator ends. A head move in progress completes
  (they are under 150 ms); a note stops at the next event.
- `Stepper::run` returns normally. The task loop sees the generation
  change, plans and runs a head lift, disables motors, reports `Idle`.
- Position stays exact on every backend, and the motor never stops from
  full speed. The pen leaves a tail of at most the acceleration distance
  (1.4 mm when drawing).
- Latency: spin, one event; RP2040, the buffered lookahead (about 200 ms)
  plus the ramp; pigpio, up to two waves (250 ms) plus the ramp.
- `halt()` is not reachable from a task. It is called by `Drop`, by the
  plot thread's panic hook, and by a future hardware e-stop.

### Progress, state and position trust

- Progress goes into a lock-free snapshot (packed atomics or `arc-swap`),
  never a mutex shared with Rocket. A realtime thread must not block on a
  handler that may itself be preempted while holding the lock.
- `ServerState` gains:

  ```rust
  pub enum MotionState { Idle, Running(TaskKind), Aborting }
  pub position_trusted: bool
  ```

  `MotionState` replaces `plotting`. `position_trusted` is cleared by a
  pigpio `halt()` and by server start, and set by `SetOrigin` (and by
  homing, once it exists). The editor warns and offers `SetOrigin` when it
  is false.
- `ServerState` is bincode and shared with the editor. The workspace ships
  one version, so this is a coordinated change, not a wire-compatibility
  problem.

### Estimation in O(segments)

The duration of a segment is `t(L)` from the closed form; a head move and
a note have closed forms too. `estimate_plot_layer_duration` runs the
waypoint planner (`AccellerationPath`) exactly as the real planner does,
sums segment durations, and never generates a single event. It is exact
for the event stream by construction, and it is fast enough to run on
every parameter change in the editor.

## RP2040 backend

### Hardware

- Raspberry Pi Pico (RP2040) or Pico 2 (RP2350); either works, the
  firmware is the same. USB to the host.
- The Pico drives STEP, DIR and EN for the four drivers: 12 GPIOs. The two
  Y drivers can share one STEP and one DIR pin (they are always driven
  identically today) or keep separate pins; the Pico has 26.
- Microstep pins never change at runtime. Hard-wire them with jumpers to
  3.3 V / GND per `micstep_vals`, or drive them from the Pico if pins are
  spare. Either way they leave the Pi.
- Optional later: endstop inputs and an e-stop input on the Pico.
- Once wired, the Pi's GPIO header is unused. The server no longer needs
  root; the service user needs the `dialout` group and a udev rule for a
  stable device name.

### Wire format

A shared `no_std` crate (`stepper_proto`) used by `server_lib` and the
firmware, `serde` + `postcard`, COBS-framed with a CRC-16.

```rust
/// One step event. 4 bytes. Streamed host -> MCU.
pub struct StepRecord {
    /// µs since the previous record. Gaps over 65 ms are split into
    /// filler records with empty masks.
    pub delay_us: u16,
    /// Bit 0 X, bit 1 Y, bit 2 Head: which STEP pins pulse.
    pub step_mask: u8,
    /// Absolute DIR state for all axes at this record (idempotent).
    pub dir_mask: u8,
}

pub enum HostToMcu {
    Ping,
    Enable(bool),
    Start,                    // begin executing buffered records
    Records(Vec<StepRecord>), // append to the ring buffer
    Halt,                     // stop now, clear buffer, report
    Status,                   // request a status frame
}

pub struct McuStatus {
    pub state: McuState,      // Idle, Running, Underrun, Halted
    pub records_consumed: u64,
    pub buffer_free: u16,
    pub executed_steps: [i32; 3],
    pub inputs: u8,           // endstops / e-stop, future
}
```

At 14 k events/s the stream is 56 KB/s, well inside USB CDC. The wave
encoder's absolute-time quantisation rule applies here too: quantise
absolute times to 1 µs, then take differences.

### Firmware

Rust, `embassy-rp`, about 500 lines.

- USB CDC task fills a ring buffer of 8 192 records (32 KB of the 264 KB
  SRAM; about 0.6 s at the worst-case rate, seconds at normal rates).
- A hardware timer alarm ISR pops one record per firing: write DIR pins
  from `dir_mask`, wait 1 µs, raise the STEP pins in `step_mask`, hold
  2 µs, lower them, arm the next alarm at `previous_alarm + delay_us`
  (absolute, no drift). Every step edge is offset by the same 1 µs, so
  relative timing is preserved. The timer IRQ gets a higher NVIC priority
  than USB.
- Empty buffer while `Running` means underrun: stop pulsing, set the flag,
  keep counting. Position stays exact.
- Status frame every 10 ms and on request.
- No host frame for 2 s while `Idle` or `Underrun`: disable motors. This
  is the crash disarm for this backend.
- `Halt`: disable the alarm, clear the buffer, report `executed_steps`.
- Flashed as UF2 over USB (hold BOOTSEL); no debug probe needed.

### Host side (`McuStepper`)

- Opens the serial device, pings, enables.
- `run` pulls events, encodes records, and keeps between 200 ms and
  `buffer_free` worth of records in flight (credit from the status
  frames). Sleeps on the serial port between status frames; never spins.
- Progress from `executed_steps` in each status frame.
- Underrun flag from the MCU is counted and logged; zero is the acceptance
  criterion.
- `halt` sends `Halt` and takes the exact executed steps from the reply.

### Running without the Pi

With the Pico on USB, `plottery_server` runs on any machine. The editor
already talks to it over HTTP, so pointing `HOST_NAME` at `localhost` is
the only change. Headless plotting from the Pi keeps working exactly as
now.

## pigpio backend

### Encoding

For each wave, walk events in time order and emit pigpio pulses.

1. Quantise *absolute* times to the sample grid, then take differences.
   Rounding deltas would drift; rounding absolutes cannot.
2. Rising edge: an entry with `gpio_on` = the step pins of every event at
   this sample and `delay` = one sample. Y is two motors and two pins in
   one mask.
3. Falling edge: an entry with `gpio_off` = the same pins and `delay` =
   until the next event.
4. Direction: when an axis's `forward` differs from its previous event, put
   the DIR pin into the mask of the *preceding* entry (at least one sample
   before the edge, which covers the 200 ns setup). If there is no
   preceding entry in this wave, emit a dedicated one. DIR polarity per
   motor is what `GpioExecutor::set_dir` does today (X and Head are inverted
   relative to Y).
5. **Wave join:** the final entry's delay reaches to the first event of the
   next wave, and the next wave's first entry has no leading delay. Timing
   across the join is then continuous.
6. Every `wave_add_generic` call places its pulses starting at the wave's
   time zero. If one wave has to be sent in several calls, each call after
   the first starts with a delay-only pulse carrying the offset.
7. Two events on one axis never land in the same sample (min period 71 µs,
   sample 2 µs), so no merging rule is needed beyond OR-ing masks.

Pure function:
`encode_wave(events, sample_us, pins, dir_state) -> (Vec<Pulse>, dir_state)`.
Unit-testable without a daemon.

### Pipeline

```
start:   A = create(plan next chunk); send(A, ONE_SHOT)
         B = create(plan next chunk); send(B, ONE_SHOT_SYNC)
loop:    sleep until A is expected to end minus 10 ms, then poll
         wave_tx_at() every 2 ms
           == A          -> keep waiting
           == B          -> delete(A); executed_pos += A.steps; progress();
                            if events exhausted -> break
                            A = create(next chunk); send(A, ONE_SHOT_SYNC)
                            swap(A, B)
           == NO_TX_WAVE -> underrun (or normal end): count, log; if events
                            remain, restart with a fresh ONE_SHOT
end:     wait until wave_tx_busy() == false; delete remaining; progress()
```

Chunking rule: close a chunk at 125 ms, or when its CB count reaches 45 %
of `wave_get_max_cbs`, whichever comes first. Never close below 60 ms
unless events are exhausted; a shorter wave can end before the next one is
queued. Two waves at 125 ms bound abort latency at 250 ms plus the ramp.
At the worst-case event rate a 125 ms wave is 1 750 events, 3 500 pulses,
about 7 000 CBs, so the CB cap rarely binds.

Producer margin: after B starts, at least 60 ms to delete A, plan about
1 750 events, send about 42 KB over the socket and queue. Each socket round
trip is well under a millisecond. The producer thread sleeps between polls,
so it can run `SCHED_FIFO` without any throttling concern and is then
immune to load. Log the minimum observed margin.

### Underrun

Pulses are never lost on underrun; the machine pauses and restarts hard,
which can skip mechanically. Count underruns per run, log each with its gap
length, and surface the count. Zero is the acceptance criterion.

### GPIO ownership

rppal maps `/dev/gpiomem`, pigpiod maps `/dev/mem`. Both write the same
GPSET/GPCLR registers, which are write-1-to-set per bit with no
read-modify-write, so concurrent use on *different* pins is safe. Pin
*mode* changes (GPFSEL) are read-modify-write in both and could race.
Rule: the pigpio backend routes every pin (step, dir, enable, microstep)
through pigpiod and does not link rppal at all.

### Daemon and service setup

- `apt install pigpio`; drop-in
  `/etc/systemd/system/pigpiod.service.d/override.conf` with an empty
  `ExecStart=` line followed by `ExecStart=/usr/bin/pigpiod -l -s 2`.
- `plottery.service`: `Requires=pigpiod.service`, `After=pigpiod.service`.
- **Crash disarm:** `ExecStopPost=/usr/bin/pigs wvhlt w 14 1 w 2 1 w 8 1 w 21 1`
  (stop waves, raise the four enable pins from `pins.rs`). `ExecStopPost`
  runs even when the main process was killed. The spin backend gets the
  same line using `pinctrl set <pin> op dh`.
- Remove `pin_process_to_plot_core` and the I/O priority call from the
  pigpio path.

## Migration plan

Each phase ends with the hardware checklist in [Testing](#testing).

### Phase 0: measure and isolate (no refactor, speed retune)

1. Buy a logic analyzer (see Testing). Record the real step period and
   pulse width on a step pin today, including the hidden sleep.
2. Explicit 2 µs busy-wait pulse; absolute schedule with re-base;
   lateness logging.
3. Kernel cmdline isolation, `sched_rt_runtime_us=-1`, the isolated-core
   guard, `SCHED_FIFO` on the plot thread only, process *not* pinned,
   `performance` governor.
4. Retune `PlotSettings::default()`: first lower the `max` values so the
   machine reproduces the physical speeds measured in step 1, then raise
   deliberately while watching for skipped steps.
5. Hardware checklist. Keep the numbers; they are the baseline.

### Phase 1: event-stream refactor

6. `StepEvent`, `PlannerState` and the planners in `server_lib`, using the
   closed form. Delete `HardwareExecutor`, `Hardware::move_steps`,
   `SpeedDelayHandler` and `TimeEstimatorExecutor`. Estimation in
   O(segments).
7. `Stepper` trait, `SpinStepper` with the Phase 0 loop, `LogStepper`.
8. Abort: generation counter in the HTTP handler and task loop,
   deceleration ramp in the planner, head lift after abort.
9. Lock-free progress snapshot; `MotionState` and `position_trusted` in
   `ServerState`; editor Remote page shows both.
10. Hardware checklist. Speeds must match Phase 0 (same physical speed,
    now also on diagonals).

### Phase 2a: RP2040 backend (recommended)

11. `stepper_proto` crate; firmware crate under `server/firmware/` or a
    sibling workspace (it needs a `thumbv6m-none-eabi` target and must not
    be in the default `cargo build`).
12. Firmware: USB CDC, ring buffer, timer ISR, status, disarm timeout,
    `Halt`. Bench-test on a breadboard with the logic analyzer before
    touching the plotter.
13. `McuStepper` with credit-based streaming and progress from status
    frames; runtime backend selection (`PLOTTERY_STEPPER=spin|mcu|pigpio`).
14. Rewire drivers to the Pico; move or hard-wire microstep pins; udev
    rule; drop `User=root` from the service.
15. Hardware checklist, load test with `stress -c 4`, full-page plot with
    zero underruns.

### Phase 2b: pigpio backend (Pi 4 and older, no hardware change)

11. Socket client (`pigpio_client.rs`): `set_mode`, `write`, `wave_clear`,
    `wave_add_new`, `wave_add_generic`, `wave_create`, `wave_delete`,
    `wave_send_using_mode`, `wave_tx_at`, `wave_tx_busy`, `wave_tx_stop`,
    `wave_get_max_cbs`, `wave_get_max_pulses`, `wave_get_cbs`.
12. `encode_wave` with tests.
13. `PigpioStepper` with the pipeline, underrun counter, progress per wave,
    `SCHED_FIFO` producer thread.
14. Install script and service changes including `ExecStopPost`.
15. Hardware checklist, load test, full-page plot with zero underruns.

### Phase 3: hardening (either backend)

16. `Drop` for the stepper and a panic hook in the plot thread call
    `halt()` and disable motors, and log it.
17. `position_trusted` handling in the editor.
18. Keep the spin backend and its isolation code; it is the fallback and
    the development path.

## Testing

### Tooling

A sigrok-compatible USB logic analyzer (about 10 EUR, 8 channels, 24 MHz)
on STEP and DIR of one axis. It measures pulse width, period, jitter and
DIR setup time for every backend, and turns the checklist below from
impressions into numbers. Software-side lateness logs complement it for
the spin backend.

### Unit tests

- `plan_segment_test.rs`
  - Axis-aligned segment: events on one axis only, equal gaps at constant
    speed.
  - 45°: alternating axes, equal cadence, total duration equals `L / v`.
  - 30° and 60°: X to Y count ratio about √3 (±1), and total duration
    equals `L / v` regardless of angle.
  - Accelerating segment: strictly decreasing gaps; decelerating: strictly
    increasing; `v0 == v1` handled without division by zero.
  - Sum of signed events equals `(dx, dy)` for random segments (property
    test, a few thousand cases).
  - Zero-length segment: no events, no panic.
  - Multi-segment plan: `t_ns` non-decreasing across shape and travel
    boundaries; head events interleave correctly with axis events.
  - Abort mid-segment: the tail is a monotonic deceleration along the
    segment direction, ends at or before the segment end, and the summed
    events equal the planned position.
  - Estimation (segment sums) equals the last event time of the generated
    stream for a fixture layer, to within one step period.
- `task_loop_test.rs`
  - Tasks queued before an abort are skipped; tasks queued after it run;
    an abort while idle does not affect the next task.
- `spin_stepper_test.rs` (with a fake clock)
  - Emits steps on the absolute schedule; after an injected 5 ms stall it
    re-bases instead of bursting.
- `stepper_proto_test.rs`
  - Round-trip every frame; a gap over 65 ms becomes filler records with
    the exact total delay; CRC rejects a corrupted frame.
- `mcu_stepper_test.rs` (fake serial port)
  - Never exceeds `buffer_free`; keeps at least 200 ms in flight; progress
    follows `executed_steps`; underrun flag is counted.
- `encode_wave_test.rs`
  - Known events produce the expected pulse list, including merged
    simultaneous X+Y edges and the Y dual-pin mask.
  - Absolute-time quantisation: the sum of delays equals the quantised span
    for 10 000 random events.
  - A direction change lands at least one sample before the step edge; a
    note stream (flip every event) encodes correctly.
  - Chunking splits at 125 ms and at the CB budget, never below 60 ms
    except at the end; the join carries the exact inter-event delay.
- `pigpio_client_test.rs`
  - Request and response framing against a fake socket for each command.

### Hardware checklist (run after every phase, record numbers)

- **Logic analyzer capture** of a 10 cm line at max travel speed: pulse
  width, min/max/mean period, DIR setup. Compare against the previous
  phase.
- **Long straight line** at max travel speed, each axis separately.
  Lateness or underrun count must be zero.
- **Angle fan**: eight 10 cm lines at 0°, 15°, ..., 105° from one origin.
  Each must take the same time (from the logic analyzer or the log). This
  catches the per-axis speed bug; closure of a shape does not.
- **Concentric squares** and a **dense spiral**; the pen returns to the
  start mark. Drift here means mechanical skipping at the configured
  speeds.
- **Load test**: editor Remote page polling, plus `stress -c 4` on the Pi,
  during a full-page plot. Lateness (spin) or underruns (MCU, pigpio) must
  be zero.
- **Abort**: mid-plot, at max speed, pen down. Motion ramps down within the
  stated latency, the head lifts, `ServerState.location` is trusted, and a
  `MoveTo` origin lands on the origin mark.
- **Crash test**: `kill -9` the server mid-plot. Output stops (spin:
  immediately; MCU: when the buffer drains; pigpio: when the queued wave
  ends) and motors are disabled within a few seconds by the disarm path.

## Later changes

### Constant-acceleration profile

Speed linear in distance means acceleration grows with speed:
`a = (dv/ds) * v`. On the draw ramp (0.4 to 4.5 cm/s over 0.14 cm) peak
acceleration is about 130 cm/s² at top speed, exactly where stepper
torque is lowest. A trapezoidal profile with the same ramp distance holds
about 70 cm/s² throughout:

```
v(s)^2 = v0^2 + 2 a s,   a = (v_max^2 - v_min^2) / (2 * accel_dist)
t(s)   = (v(s) - v0) / a
```

The segment planner change is trivial, but `AccellerationPath` must then
plan reachability in velocity-squared space too, or corners get speeds the
ramp cannot honour. Its 343 lines of tests are the reason this is a
separate change after Phase 1, with its own angle-fan and spiral run.

### Pause / continue

With a lazy planner, pause is "stop pulling events at a safe point". Safe
point: between shapes (the planner checks a flag before each shape) or a
deceleration ramp with a head lift (leaves a tiny mark). The remainder is
the unconsumed rest of the layer iterator, held by the task loop. Add
`MotionState::Paused`.

### Endstops and homing

Switches wired to the Pico (or, with pigpio, to Pi inputs polled by the
producer). `Task::Home`, `MotionState::Homing`, sets `position_trusted`.
An endstop hit during normal motion is `halt()`; on the RP2040 that still
leaves position exact.

## Open questions

- **Which Pi is `otter.local`?** Decides whether pigpio is even possible.
  Not recorded anywhere in the repo; put it in `server/README.md`.
- **Are the drivers A4988?** The microstep table matches; confirm from the
  boards. A DRV8825 needs 1.9 µs pulses (still fine at 2 µs).
- **Pico or Pico 2?** Either works; the Pico 2 has more SRAM and a faster
  core, neither of which matters here.
- **Microstep pins: hard-wire or drive from the Pico?** Hard-wiring is
  less firmware and fewer wires; driving keeps `micstep_vals` in software.
- **pigpio sync-queue depth** (Phase 2b only). Verify in `pigpio.c` that a
  second `ONE_SHOT_SYNC` send while one is queued replaces rather than
  appends. The pipeline never has two queued and is safe either way.
- **pigpiod extension size** (Phase 2b only). A 3 500-pulse wave is 42 KB
  in one `wave_add_generic` call; confirm the daemon accepts it, otherwise
  split with offset pulses as described.

## References

- A4988 datasheet, timing section: <https://www.pololu.com/file/0J450/A4988.pdf>
- Linux CPU isolation: `Documentation/admin-guide/kernel-parameters.txt`
  (`isolcpus`, `nohz_full`, `rcu_nocbs`, `irqaffinity`) and
  `Documentation/scheduler/sched-rt-group.rst` (`sched_rt_runtime_us`)
- systemd `ExecStopPost` semantics: `man systemd.service`
- RP2040 datasheet, Timer chapter; `embassy-rp`: <https://embassy.dev>;
  `postcard`: <https://github.com/jamesmunns/postcard>
- sigrok / PulseView: <https://sigrok.org>
- pigpio C interface, waves:
  <https://abyz.me.uk/rpi/pigpio/cif.html#gpioWaveAddGeneric>
- pigpio daemon socket protocol: <https://abyz.me.uk/rpi/pigpio/sif.html>;
  reference client `pigpio.py` in <https://github.com/joan2937/pigpio>
- pigpiod options (`-s` sample rate, `-t` clock peripheral, `-l`):
  <https://abyz.me.uk/rpi/pigpio/pigpiod.html>
- Pi 5 / RP1 PIO: <https://github.com/raspberrypi/utils/tree/master/piolib>
- Klipper architecture: <https://www.klipper3d.org/Overview.html>
- rppal: <https://github.com/golemparts/rppal>
