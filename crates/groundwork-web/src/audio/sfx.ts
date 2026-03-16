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

/** Play a gentle "splash" for watering */
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

/** Play a fauna arrival sound based on type */
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
