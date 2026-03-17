/**
 * Sound effects for garden actions.
 *
 * Procedural sounds via Web Audio API — no audio files.
 * Each tool has a distinct short sound:
 * - Plant: soft earthy "pat" (low thump + noise burst)
 * - Water: gentle "splash" (filtered noise sweep)
 * - Dig: satisfying "scrunch" (noise with resonance)
 * - Milestone: warm "chime" (sine wave harmonics)
 */

let ctx: AudioContext | null = null;

/** Get or create the shared AudioContext */
function getContext(): AudioContext | null {
  if (!ctx) {
    try { ctx = new AudioContext(); } catch { return null; }
  }
  if (ctx.state === 'suspended') ctx.resume();
  return ctx;
}

/** Play a soft earthy "pat" for seed planting */
export function playPlant(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Low thump
  const osc = c.createOscillator();
  osc.frequency.setValueAtTime(120, t);
  osc.frequency.exponentialRampToValueAtTime(60, t + 0.1);
  const gain = c.createGain();
  gain.gain.setValueAtTime(0.15, t);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.15);
  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.15);

  // Soft noise burst
  const buf = c.createBuffer(1, c.sampleRate * 0.05, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1) * 0.1;
  const noise = c.createBufferSource();
  noise.buffer = buf;
  const lpf = c.createBiquadFilter();
  lpf.type = 'lowpass';
  lpf.frequency.value = 800;
  const ng = c.createGain();
  ng.gain.setValueAtTime(0.1, t);
  ng.gain.exponentialRampToValueAtTime(0.001, t + 0.08);
  noise.connect(lpf).connect(ng).connect(c.destination);
  noise.start(t);
}

/** Play a gentle "splash" — used for water flow events (watering can tool removed; irrigation via digging) */
export function playWater(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  const buf = c.createBuffer(1, c.sampleRate * 0.15, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1);
  const noise = c.createBufferSource();
  noise.buffer = buf;

  const bpf = c.createBiquadFilter();
  bpf.type = 'bandpass';
  bpf.frequency.setValueAtTime(2000, t);
  bpf.frequency.exponentialRampToValueAtTime(400, t + 0.12);
  bpf.Q.value = 1;

  const gain = c.createGain();
  gain.gain.setValueAtTime(0.08, t);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.15);

  noise.connect(bpf).connect(gain).connect(c.destination);
  noise.start(t);
}

/** Play a "scrunch" for digging */
export function playDig(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  const buf = c.createBuffer(1, c.sampleRate * 0.1, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < data.length; i++) data[i] = (Math.random() * 2 - 1);
  const noise = c.createBufferSource();
  noise.buffer = buf;

  const bpf = c.createBiquadFilter();
  bpf.type = 'bandpass';
  bpf.frequency.value = 600;
  bpf.Q.value = 2;

  const gain = c.createGain();
  gain.gain.setValueAtTime(0.12, t);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.12);

  noise.connect(bpf).connect(gain).connect(c.destination);
  noise.start(t);
}

/** Play a warm "chime" for milestones */
export function playMilestone(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Two harmonics for warmth
  for (const freq of [523, 659]) {
    const osc = c.createOscillator();
    osc.type = 'sine';
    osc.frequency.value = freq;
    const gain = c.createGain();
    gain.gain.setValueAtTime(0.1, t);
    gain.gain.exponentialRampToValueAtTime(0.001, t + 0.8);
    osc.connect(gain).connect(c.destination);
    osc.start(t);
    osc.stop(t + 0.8);
  }
}

/** Play a cozy bird chirp — two quick descending sine sweeps */
export function playBirdCall(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Slight random variation for natural feel
  const base = 2400 + Math.random() * 800; // 2400-3200 Hz

  // First chirp: quick descending sweep
  const osc1 = c.createOscillator();
  osc1.type = 'sine';
  osc1.frequency.setValueAtTime(base, t);
  osc1.frequency.exponentialRampToValueAtTime(base * 0.6, t + 0.08);
  const g1 = c.createGain();
  g1.gain.setValueAtTime(0.06, t);
  g1.gain.setValueAtTime(0.06, t + 0.03);
  g1.gain.exponentialRampToValueAtTime(0.001, t + 0.1);
  osc1.connect(g1).connect(c.destination);
  osc1.start(t);
  osc1.stop(t + 0.1);

  // Second chirp: slightly higher, 120ms later
  const osc2 = c.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.setValueAtTime(base * 1.15, t + 0.12);
  osc2.frequency.exponentialRampToValueAtTime(base * 0.7, t + 0.2);
  const g2 = c.createGain();
  g2.gain.setValueAtTime(0.05, t + 0.12);
  g2.gain.setValueAtTime(0.05, t + 0.15);
  g2.gain.exponentialRampToValueAtTime(0.001, t + 0.22);
  osc2.connect(g2).connect(c.destination);
  osc2.start(t + 0.12);
  osc2.stop(t + 0.22);
}

/** Play a distant bird call — quieter, slightly delayed, varied pitch.
 *  Layering these with nearby calls creates a full dawn chorus soundscape. */
export function playDistantBird(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime + Math.random() * 0.3; // stagger start
  const base = 1800 + Math.random() * 1200; // wider pitch range for variety
  const volume = 0.01 + Math.random() * 0.02; // quieter = farther away

  const osc = c.createOscillator();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(base, t);
  osc.frequency.exponentialRampToValueAtTime(base * (0.5 + Math.random() * 0.5), t + 0.12);

  const gain = c.createGain();
  gain.gain.setValueAtTime(volume, t);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.15);

  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.16);
}

/** Play a soft wind chime — 2-3 high sine tones with long decay, garden feels cultivated */
export function playWindChime(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Pentatonic wind chime tones (C6, E6, G6, A6) — pick 2-3 randomly
  const tones = [1047, 1319, 1568, 1760];
  const count = 2 + Math.floor(Math.random() * 2);
  const used = new Set<number>();

  for (let i = 0; i < count; i++) {
    let idx: number;
    do { idx = Math.floor(Math.random() * tones.length); } while (used.has(idx));
    used.add(idx);

    const osc = c.createOscillator();
    osc.type = 'sine';
    const freq = tones[idx] * (0.98 + Math.random() * 0.04); // slight detuning
    osc.frequency.value = freq;

    const gain = c.createGain();
    const start = t + i * (0.08 + Math.random() * 0.12);
    gain.gain.setValueAtTime(0, start);
    gain.gain.linearRampToValueAtTime(0.025, start + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 1.2);
    osc.connect(gain).connect(c.destination);
    osc.start(start);
    osc.stop(start + 1.3);
  }
}

/** Play a warbling trill — rapid 4-note ascending pattern, like a wren */
export function playBirdWarble(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Base pitch with random variation
  const base = 1800 + Math.random() * 600;
  const notes = [base, base * 1.12, base * 1.25, base * 1.33];

  for (let i = 0; i < notes.length; i++) {
    const osc = c.createOscillator();
    osc.type = 'sine';
    const start = t + i * 0.06;
    osc.frequency.setValueAtTime(notes[i], start);
    osc.frequency.exponentialRampToValueAtTime(notes[i] * 0.85, start + 0.05);
    const gain = c.createGain();
    gain.gain.setValueAtTime(0.04, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.06);
    osc.connect(gain).connect(c.destination);
    osc.start(start);
    osc.stop(start + 0.07);
  }
}

/** Play a robin-like song — melodic 5-note phrase that rises and falls */
export function playRobinSong(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  const base = 1400 + Math.random() * 400;
  // Rise-and-fall melodic contour
  const intervals = [1.0, 1.2, 1.35, 1.25, 1.05];
  const durations = [0.09, 0.07, 0.1, 0.08, 0.12];

  let offset = 0;
  for (let i = 0; i < intervals.length; i++) {
    const osc = c.createOscillator();
    osc.type = 'sine';
    const start = t + offset;
    const freq = base * intervals[i];
    osc.frequency.setValueAtTime(freq, start);
    // Slight downward sweep on each note for natural feel
    osc.frequency.exponentialRampToValueAtTime(freq * 0.9, start + durations[i]);
    const gain = c.createGain();
    gain.gain.setValueAtTime(0.05, start);
    gain.gain.setValueAtTime(0.05, start + durations[i] * 0.5);
    gain.gain.exponentialRampToValueAtTime(0.001, start + durations[i] + 0.02);
    osc.connect(gain).connect(c.destination);
    osc.start(start);
    osc.stop(start + durations[i] + 0.03);
    offset += durations[i] + 0.02;
  }
}

/** Play a brief sparkle for a shooting star — descending high-freq shimmer */
export function playShootingStar(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Quick descending shimmer: high sine sweep down
  const osc = c.createOscillator();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(4000, t);
  osc.frequency.exponentialRampToValueAtTime(1500, t + 0.4);
  const gain = c.createGain();
  gain.gain.setValueAtTime(0, t);
  gain.gain.linearRampToValueAtTime(0.03, t + 0.05);
  gain.gain.setValueAtTime(0.03, t + 0.15);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.5);
  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.5);
}

/** Play a soft owl hoot — two low descending tones, classic "hoo-hoo" */
export function playOwlHoot(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // First hoo: descending sine with vibrato
  const osc1 = c.createOscillator();
  osc1.type = 'sine';
  osc1.frequency.setValueAtTime(380, t);
  osc1.frequency.exponentialRampToValueAtTime(320, t + 0.3);
  const vib1 = c.createOscillator();
  vib1.frequency.value = 4;
  const vibGain1 = c.createGain();
  vibGain1.gain.value = 5;
  vib1.connect(vibGain1);
  vibGain1.connect(osc1.frequency);
  const g1 = c.createGain();
  g1.gain.setValueAtTime(0, t);
  g1.gain.linearRampToValueAtTime(0.04, t + 0.05);
  g1.gain.setValueAtTime(0.04, t + 0.2);
  g1.gain.exponentialRampToValueAtTime(0.001, t + 0.35);
  osc1.connect(g1).connect(c.destination);
  osc1.start(t);
  vib1.start(t);
  osc1.stop(t + 0.35);
  vib1.stop(t + 0.35);

  // Second hoo: slightly lower, 0.4s later
  const osc2 = c.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.setValueAtTime(340, t + 0.4);
  osc2.frequency.exponentialRampToValueAtTime(280, t + 0.75);
  const vib2 = c.createOscillator();
  vib2.frequency.value = 4;
  const vibGain2 = c.createGain();
  vibGain2.gain.value = 5;
  vib2.connect(vibGain2);
  vibGain2.connect(osc2.frequency);
  const g2 = c.createGain();
  g2.gain.setValueAtTime(0, t + 0.4);
  g2.gain.linearRampToValueAtTime(0.035, t + 0.45);
  g2.gain.setValueAtTime(0.035, t + 0.6);
  g2.gain.exponentialRampToValueAtTime(0.001, t + 0.8);
  osc2.connect(g2).connect(c.destination);
  osc2.start(t + 0.4);
  vib2.start(t + 0.4);
  osc2.stop(t + 0.8);
  vib2.stop(t + 0.8);
}

/** Play a soft buzz for bee/butterfly arrival */
export function playBuzz(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Low oscillator modulated by a higher frequency for buzz texture
  const osc = c.createOscillator();
  osc.type = 'sawtooth';
  osc.frequency.value = 180 + Math.random() * 60;
  const bpf = c.createBiquadFilter();
  bpf.type = 'bandpass';
  bpf.frequency.value = 400;
  bpf.Q.value = 3;
  const gain = c.createGain();
  gain.gain.setValueAtTime(0.04, t);
  gain.gain.setValueAtTime(0.04, t + 0.05);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.15);
  osc.connect(bpf).connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.15);
}

/** Play a rapid squirrel chitter — 5-6 quick high-pitched clicks */
export function playSquirrelChitter(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  const count = 5 + Math.floor(Math.random() * 2);
  const base = 1200 + Math.random() * 400;

  for (let i = 0; i < count; i++) {
    const osc = c.createOscillator();
    osc.type = 'sine';
    const start = t + i * 0.04;
    const freq = base + (Math.random() - 0.5) * 200;
    osc.frequency.setValueAtTime(freq, start);
    osc.frequency.exponentialRampToValueAtTime(freq * 0.7, start + 0.025);
    const gain = c.createGain();
    gain.gain.setValueAtTime(0.03, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.03);
    osc.connect(gain).connect(c.destination);
    osc.start(start);
    osc.stop(start + 0.04);
  }
}

/** Play a quiet wooden creak — the sound of a tree stretching as it grows */
export function playTreeCreak(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Low filtered noise with resonance — like bending wood
  const bufSize = c.sampleRate * 0.6;
  const buf = c.createBuffer(1, bufSize, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;

  const src = c.createBufferSource();
  src.buffer = buf;

  const bpf = c.createBiquadFilter();
  bpf.type = 'bandpass';
  bpf.frequency.setValueAtTime(200 + Math.random() * 100, t);
  bpf.frequency.exponentialRampToValueAtTime(80, t + 0.5);
  bpf.Q.value = 4; // resonant — gives the woody character

  const gain = c.createGain();
  gain.gain.setValueAtTime(0, t);
  gain.gain.linearRampToValueAtTime(0.03, t + 0.05);
  gain.gain.setValueAtTime(0.03, t + 0.2);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.6);

  src.connect(bpf).connect(gain).connect(c.destination);
  src.start(t);
  src.stop(t + 0.6);
}

/** Play a tiny water drip tinkle — accompanies dawn dew sparkles */
export function playDewDrop(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // High, delicate sine with quick attack and fast decay — like a droplet
  const freq = 1800 + Math.random() * 600;
  const osc = c.createOscillator();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(freq, t);
  osc.frequency.exponentialRampToValueAtTime(freq * 0.7, t + 0.15);
  const gain = c.createGain();
  gain.gain.setValueAtTime(0.02, t);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.2);
  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.2);
}

/** Play a soft ascending "growth" shimmer — gentle reward for plant growth */
export function playGrowth(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Gentle ascending sine — like something unfurling
  const osc = c.createOscillator();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(300, t);
  osc.frequency.exponentialRampToValueAtTime(600, t + 0.3);
  const gain = c.createGain();
  gain.gain.setValueAtTime(0.04, t);
  gain.gain.setValueAtTime(0.04, t + 0.15);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.4);
  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 0.4);

  // Soft shimmer overtone
  const osc2 = c.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.setValueAtTime(900, t + 0.05);
  osc2.frequency.exponentialRampToValueAtTime(1200, t + 0.25);
  const g2 = c.createGain();
  g2.gain.setValueAtTime(0.02, t + 0.05);
  g2.gain.exponentialRampToValueAtTime(0.001, t + 0.3);
  osc2.connect(g2).connect(c.destination);
  osc2.start(t + 0.05);
  osc2.stop(t + 0.3);
}

/** Play a gentle "wonder" chime for ecological discoveries.
 *  Three ascending notes (major triad) — quieter and more delicate
 *  than the milestone chime. Used for wild plant appearances,
 *  first-time ecological interactions, and surprise events. */
export function playDiscovery(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Ascending major triad: C5 → E5 → G5, each slightly delayed
  const notes = [523, 659, 784];
  for (let i = 0; i < notes.length; i++) {
    const osc = c.createOscillator();
    osc.type = 'sine';
    osc.frequency.value = notes[i];
    const gain = c.createGain();
    const start = t + i * 0.12;
    gain.gain.setValueAtTime(0, start);
    gain.gain.linearRampToValueAtTime(0.06, start + 0.03);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.5);
    osc.connect(gain).connect(c.destination);
    osc.start(start);
    osc.stop(start + 0.5);
  }
}

/** Play a gentle rain onset — descending filtered noise, like first drops */
export function playRainStart(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Band-pass filtered noise — descending center frequency
  const bufSize = c.sampleRate * 0.8;
  const buf = c.createBuffer(1, bufSize, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;

  const src = c.createBufferSource();
  src.buffer = buf;
  const filter = c.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.setValueAtTime(2000, t);
  filter.frequency.exponentialRampToValueAtTime(400, t + 0.7);
  filter.Q.value = 2;
  const gain = c.createGain();
  gain.gain.setValueAtTime(0, t);
  gain.gain.linearRampToValueAtTime(0.06, t + 0.1);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 0.8);
  src.connect(filter).connect(gain).connect(c.destination);
  src.start(t);
  src.stop(t + 0.8);
}

/** Play a dry wind whistle for drought onset */
export function playDroughtStart(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Gentle wind — sine with slow vibrato
  const osc = c.createOscillator();
  osc.type = 'sine';
  osc.frequency.setValueAtTime(200, t);
  osc.frequency.linearRampToValueAtTime(150, t + 1.0);

  const lfo = c.createOscillator();
  lfo.frequency.value = 3;
  const lfoGain = c.createGain();
  lfoGain.gain.value = 15;
  lfo.connect(lfoGain).connect(osc.frequency);
  lfo.start(t);
  lfo.stop(t + 1.0);

  const gain = c.createGain();
  gain.gain.setValueAtTime(0, t);
  gain.gain.linearRampToValueAtTime(0.04, t + 0.2);
  gain.gain.linearRampToValueAtTime(0.04, t + 0.6);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 1.0);
  osc.connect(gain).connect(c.destination);
  osc.start(t);
  osc.stop(t + 1.0);
}

/** Play a soft wind gust whoosh — filtered noise sweep */
export function playWindGust(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  // Short burst of filtered noise that sweeps up then fades
  const bufSize = c.sampleRate;
  const buf = c.createBuffer(1, bufSize, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;

  const noise = c.createBufferSource();
  noise.buffer = buf;

  const filter = c.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.setValueAtTime(200, t);
  filter.frequency.linearRampToValueAtTime(600, t + 0.4);
  filter.frequency.linearRampToValueAtTime(250, t + 1.2);
  filter.Q.value = 0.8;

  const gain = c.createGain();
  gain.gain.setValueAtTime(0, t);
  gain.gain.linearRampToValueAtTime(0.06, t + 0.3);
  gain.gain.linearRampToValueAtTime(0.04, t + 0.6);
  gain.gain.exponentialRampToValueAtTime(0.001, t + 1.2);

  noise.connect(filter).connect(gain).connect(c.destination);
  noise.start(t);
  noise.stop(t + 1.2);
}

/** Play a gnome state-change sound. Call on state transitions only. */
export function playGnomeSound(state: number): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  switch (state) {
    case 2: { // Working: small effort grunt — low tone + noise
      const osc = c.createOscillator();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(100, t);
      osc.frequency.exponentialRampToValueAtTime(80, t + 0.1);
      const gain = c.createGain();
      gain.gain.setValueAtTime(0.04, t);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.12);
      osc.connect(gain).connect(c.destination);
      osc.start(t);
      osc.stop(t + 0.12);
      break;
    }
    case 6: { // Inspecting: curious "hmm" — ascending tone
      const osc = c.createOscillator();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(180, t);
      osc.frequency.linearRampToValueAtTime(240, t + 0.2);
      osc.frequency.linearRampToValueAtTime(220, t + 0.3);
      const gain = c.createGain();
      gain.gain.setValueAtTime(0.03, t);
      gain.gain.setValueAtTime(0.03, t + 0.2);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.35);
      osc.connect(gain).connect(c.destination);
      osc.start(t);
      osc.stop(t + 0.35);
      break;
    }
    case 4: { // Resting: soft sigh — descending filtered noise
      const bufSize = Math.floor(c.sampleRate * 0.3);
      const buf = c.createBuffer(1, bufSize, c.sampleRate);
      const data = buf.getChannelData(0);
      for (let i = 0; i < bufSize; i++) data[i] = (Math.random() * 2 - 1);
      const noise = c.createBufferSource();
      noise.buffer = buf;
      const filter = c.createBiquadFilter();
      filter.type = 'lowpass';
      filter.frequency.setValueAtTime(400, t);
      filter.frequency.exponentialRampToValueAtTime(150, t + 0.3);
      const gain = c.createGain();
      gain.gain.setValueAtTime(0.025, t);
      gain.gain.exponentialRampToValueAtTime(0.001, t + 0.3);
      noise.connect(filter).connect(gain).connect(c.destination);
      noise.start(t);
      noise.stop(t + 0.3);
      break;
    }
    case 3: { // Eating: soft munch — two quick low clicks
      for (let i = 0; i < 2; i++) {
        const start = t + i * 0.08;
        const osc = c.createOscillator();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(250, start);
        osc.frequency.exponentialRampToValueAtTime(100, start + 0.03);
        const gain = c.createGain();
        gain.gain.setValueAtTime(0.03, start);
        gain.gain.exponentialRampToValueAtTime(0.001, start + 0.05);
        osc.connect(gain).connect(c.destination);
        osc.start(start);
        osc.stop(start + 0.05);
      }
      break;
    }
  }
}

/** Play a fauna arrival sound based on type */
/** Play a subtle underground crackle when roots expand.
 *  3-4 rapid tiny snaps at low pitch — soil breaking apart. */
export function playRootCrackle(): void {
  const c = getContext();
  if (!c) return;
  const t = c.currentTime;

  const snaps = 3 + Math.floor(Math.random() * 2); // 3-4 snaps
  for (let i = 0; i < snaps; i++) {
    const start = t + i * 0.04 + Math.random() * 0.02;
    // Short noise burst — filtered low for earthy character
    const bufSize = Math.floor(c.sampleRate * 0.03);
    const buf = c.createBuffer(1, bufSize, c.sampleRate);
    const data = buf.getChannelData(0);
    for (let j = 0; j < bufSize; j++) data[j] = Math.random() * 2 - 1;

    const src = c.createBufferSource();
    src.buffer = buf;

    const lpf = c.createBiquadFilter();
    lpf.type = 'lowpass';
    lpf.frequency.value = 400 + Math.random() * 200;

    const gain = c.createGain();
    gain.gain.setValueAtTime(0.015 + Math.random() * 0.01, start);
    gain.gain.exponentialRampToValueAtTime(0.001, start + 0.03);

    src.connect(lpf).connect(gain).connect(c.destination);
    src.start(start);
    src.stop(start + 0.04);
  }
}

export function playFaunaArrival(faunaType: number): void {
  // FaunaType: 0=Bee, 1=Butterfly, 2=Bird, 3=Worm, 4=Beetle
  switch (faunaType) {
    case 2: // Bird
      playBirdCall();
      break;
    case 0: // Bee
    case 1: // Butterfly
      playBuzz();
      break;
    // Worm and beetle are silent (underground/subtle)
  }
}
