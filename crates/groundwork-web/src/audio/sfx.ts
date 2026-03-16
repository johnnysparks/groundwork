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
