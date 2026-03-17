/**
 * Ambient garden audio: procedural water spring sound.
 *
 * Uses Web Audio API to generate a gentle water trickle without any
 * external audio files. White noise → band-pass filter → gain modulation
 * creates a convincing continuous water sound.
 *
 * Starts muted and fades in on first user interaction (browser autoplay policy).
 */

let ctx: AudioContext | null = null;
let masterGain: GainNode | null = null;
let rainGain: GainNode | null = null;
let cricketGain: GainNode | null = null;
let windGain: GainNode | null = null;
let windFilter: BiquadFilterNode | null = null;
let rustleGain: GainNode | null = null;
let rustleFilter: BiquadFilterNode | null = null;
let pollinatorGain: GainNode | null = null;
let frogGain: GainNode | null = null;
let beetleGain: GainNode | null = null;
let waterSoundGain: GainNode | null = null;
let started = false;
let isRaining = false;
let isNight = false;
let frogsActive = false;

/** Create the procedural water sound */
function createWaterSound(audioCtx: AudioContext, output: GainNode): void {
  // White noise source
  const bufferSize = audioCtx.sampleRate * 2;
  const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < bufferSize; i++) {
    data[i] = Math.random() * 2 - 1;
  }

  const noise = audioCtx.createBufferSource();
  noise.buffer = buffer;
  noise.loop = true;

  // Band-pass filter: keep only water-like frequencies (200-800 Hz)
  const filter = audioCtx.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.value = 400;
  filter.Q.value = 0.5;

  // Second filter for more natural sound
  const filter2 = audioCtx.createBiquadFilter();
  filter2.type = 'lowpass';
  filter2.frequency.value = 1200;

  // Gentle volume modulation for natural variation
  const lfo = audioCtx.createOscillator();
  const lfoGain = audioCtx.createGain();
  lfo.frequency.value = 0.3; // slow modulation
  lfoGain.gain.value = 0.15; // subtle volume variation

  waterSoundGain = audioCtx.createGain();
  waterSoundGain.gain.value = 0.04; // starts quiet — scales with water volume

  const waterGain = waterSoundGain;

  // Connect: noise → filter → filter2 → waterGain → output
  noise.connect(filter);
  filter.connect(filter2);
  filter2.connect(waterGain);
  waterGain.connect(output);

  // LFO modulates the water gain for natural variation
  lfo.connect(lfoGain);
  lfoGain.connect(waterGain.gain);

  noise.start();
  lfo.start();
}

/** Create procedural rain patter sound — higher freq noise with irregular modulation */
function createRainSound(audioCtx: AudioContext, output: GainNode): void {
  const bufferSize = audioCtx.sampleRate * 2;
  const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < bufferSize; i++) {
    data[i] = Math.random() * 2 - 1;
  }

  const noise = audioCtx.createBufferSource();
  noise.buffer = buffer;
  noise.loop = true;

  // Higher band-pass for rain patter (800-3000 Hz)
  const filter = audioCtx.createBiquadFilter();
  filter.type = 'bandpass';
  filter.frequency.value = 1800;
  filter.Q.value = 0.3;

  // High-shelf to add brightness
  const shelf = audioCtx.createBiquadFilter();
  shelf.type = 'highshelf';
  shelf.frequency.value = 2500;
  shelf.gain.value = 3;

  // Irregular modulation for patter feel
  const lfo = audioCtx.createOscillator();
  const lfoGain = audioCtx.createGain();
  lfo.frequency.value = 2.5; // faster than water — patter rhythm
  lfoGain.gain.value = 0.3;

  // Rain-specific gain (controlled externally for fade in/out)
  rainGain = audioCtx.createGain();
  rainGain.gain.value = 0; // start silent

  noise.connect(filter);
  filter.connect(shelf);
  shelf.connect(rainGain);
  rainGain.connect(output);

  lfo.connect(lfoGain);
  lfoGain.connect(rainGain.gain);

  noise.start();
  lfo.start();
}

/** Create procedural cricket chirps — rhythmic high-freq oscillation, classic evening sound */
function createCricketSound(audioCtx: AudioContext, output: GainNode): void {
  // Cricket chirps: rapid oscillation between two tones at ~4000Hz
  // Modulated by a slower rhythm that creates the chirp pattern
  const osc = audioCtx.createOscillator();
  osc.type = 'sine';
  osc.frequency.value = 4200;

  // Rapid tremolo: gives the characteristic cricket "trill"
  const tremolo = audioCtx.createOscillator();
  const tremoloGain = audioCtx.createGain();
  tremolo.frequency.value = 30; // 30Hz = rapid trill
  tremoloGain.gain.value = 0.5;
  tremolo.connect(tremoloGain);
  tremoloGain.connect(osc.frequency); // FM creates the trill

  // Slow on/off: creates the chirp→silence→chirp pattern
  const rhythm = audioCtx.createOscillator();
  const rhythmGain = audioCtx.createGain();
  rhythm.frequency.value = 2.5; // chirps per second
  rhythmGain.gain.value = 1.0;

  // The rhythmGain modulates the signal gain to make chirps
  const chirpGate = audioCtx.createGain();
  chirpGate.gain.value = 0;
  rhythm.connect(rhythmGain);
  rhythmGain.connect(chirpGate.gain);

  // Second cricket at a slightly different frequency for natural chorus
  const osc2 = audioCtx.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.value = 3800;

  const tremolo2 = audioCtx.createOscillator();
  const tremoloGain2 = audioCtx.createGain();
  tremolo2.frequency.value = 28;
  tremoloGain2.gain.value = 0.5;
  tremolo2.connect(tremoloGain2);
  tremoloGain2.connect(osc2.frequency);

  const rhythm2 = audioCtx.createOscillator();
  const rhythmGain2 = audioCtx.createGain();
  rhythm2.frequency.value = 2.2; // slightly offset rhythm
  rhythmGain2.gain.value = 1.0;

  const chirpGate2 = audioCtx.createGain();
  chirpGate2.gain.value = 0;
  rhythm2.connect(rhythmGain2);
  rhythmGain2.connect(chirpGate2.gain);

  // Cricket-specific gain (controlled externally for day/night fade)
  cricketGain = audioCtx.createGain();
  cricketGain.gain.value = 0; // start silent

  osc.connect(chirpGate);
  chirpGate.connect(cricketGain);
  osc2.connect(chirpGate2);
  chirpGate2.connect(cricketGain);
  cricketGain.connect(output);

  osc.start();
  tremolo.start();
  rhythm.start();
  osc2.start();
  tremolo2.start();
  rhythm2.start();
}

/** Create gentle wind — low-frequency filtered noise, volume varies with weather */
function createWindSound(audioCtx: AudioContext, output: GainNode): void {
  const bufferSize = audioCtx.sampleRate * 2;
  const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < bufferSize; i++) {
    data[i] = Math.random() * 2 - 1;
  }

  const noise = audioCtx.createBufferSource();
  noise.buffer = buffer;
  noise.loop = true;

  // Low-pass filter — wind is low rumble, not hiss
  windFilter = audioCtx.createBiquadFilter();
  windFilter.type = 'lowpass';
  windFilter.frequency.value = 300; // base: gentle breeze
  windFilter.Q.value = 1.0;

  // Slow modulation for wind gusts
  const lfo = audioCtx.createOscillator();
  const lfoGain = audioCtx.createGain();
  lfo.frequency.value = 0.15; // very slow — breathe in, breathe out
  lfoGain.gain.value = 0.02;

  windGain = audioCtx.createGain();
  windGain.gain.value = 0.03; // very quiet base level

  noise.connect(windFilter);
  windFilter.connect(windGain);
  windGain.connect(output);

  lfo.connect(lfoGain);
  lfoGain.connect(windGain.gain);

  noise.start();
  lfo.start();
}

/** Create leaf rustle — higher-freq shimmer layered above wind rumble.
 *  Scales with foliage count + wind strength for a garden-responsive soundscape. */
function createLeafRustleSound(audioCtx: AudioContext, output: GainNode): void {
  const bufferSize = audioCtx.sampleRate * 2;
  const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
  const data = buffer.getChannelData(0);
  for (let i = 0; i < bufferSize; i++) {
    data[i] = Math.random() * 2 - 1;
  }

  const noise = audioCtx.createBufferSource();
  noise.buffer = buffer;
  noise.loop = true;

  // Bandpass filter: 2500Hz — the "shh" of leaves in wind
  rustleFilter = audioCtx.createBiquadFilter();
  rustleFilter.type = 'bandpass';
  rustleFilter.frequency.value = 2500;
  rustleFilter.Q.value = 0.5;

  // Gentle random modulation for organic shimmer
  const lfo = audioCtx.createOscillator();
  const lfoGain = audioCtx.createGain();
  lfo.frequency.value = 0.4; // faster than wind LFO — leaf flutter
  lfoGain.gain.value = 0.01;

  rustleGain = audioCtx.createGain();
  rustleGain.gain.value = 0; // starts silent — scales with foliage

  noise.connect(rustleFilter);
  rustleFilter.connect(rustleGain);
  rustleGain.connect(output);

  lfo.connect(lfoGain);
  lfoGain.connect(rustleGain.gain);

  noise.start();
  lfo.start();
}

/** Create ambient pollinator hum — warm oscillator buzz that scales with bee/butterfly count. */
function createPollinatorHum(audioCtx: AudioContext, output: GainNode): void {
  // Base bee buzz: ~230Hz sine with FM vibrato
  const osc = audioCtx.createOscillator();
  osc.type = 'sine';
  osc.frequency.value = 230;

  // Vibrato: slight frequency wobble for organic buzzy sound
  const vibrato = audioCtx.createOscillator();
  const vibratoGain = audioCtx.createGain();
  vibrato.frequency.value = 8; // fast tremor
  vibratoGain.gain.value = 12; // ±12Hz wobble
  vibrato.connect(vibratoGain);
  vibratoGain.connect(osc.frequency);

  // Second harmonic for richer buzz
  const osc2 = audioCtx.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.value = 345; // ~1.5x fundamental — not a perfect harmonic, more organic

  const osc2Gain = audioCtx.createGain();
  osc2Gain.gain.value = 0.3; // quieter overtone

  pollinatorGain = audioCtx.createGain();
  pollinatorGain.gain.value = 0; // starts silent

  osc.connect(pollinatorGain);
  osc2.connect(osc2Gain);
  osc2Gain.connect(pollinatorGain);
  pollinatorGain.connect(output);

  osc.start();
  vibrato.start();
  osc2.start();
}

/** Create ambient frog chorus — low croaks near water during dusk/night. */
function createFrogChorus(audioCtx: AudioContext, output: GainNode): void {
  // Frog 1: deep croak at ~120Hz
  const osc1 = audioCtx.createOscillator();
  osc1.type = 'sine';
  osc1.frequency.value = 120;

  // Slow FM wobble gives organic croak quality
  const wobble1 = audioCtx.createOscillator();
  const wobbleGain1 = audioCtx.createGain();
  wobble1.frequency.value = 5;
  wobbleGain1.gain.value = 20; // ±20Hz pitch wobble
  wobble1.connect(wobbleGain1);
  wobbleGain1.connect(osc1.frequency);

  // Rhythmic gating: croak pattern (~1.5Hz = one croak per ~0.7s)
  const gate1 = audioCtx.createGain();
  gate1.gain.value = 0;
  const rhythm1 = audioCtx.createOscillator();
  const rhythmGain1 = audioCtx.createGain();
  rhythm1.frequency.value = 1.5;
  rhythmGain1.gain.value = 1.0;
  rhythm1.connect(rhythmGain1);
  rhythmGain1.connect(gate1.gain);

  // Frog 2: slightly higher, offset rhythm
  const osc2 = audioCtx.createOscillator();
  osc2.type = 'sine';
  osc2.frequency.value = 145;

  const wobble2 = audioCtx.createOscillator();
  const wobbleGain2 = audioCtx.createGain();
  wobble2.frequency.value = 4;
  wobbleGain2.gain.value = 15;
  wobble2.connect(wobbleGain2);
  wobbleGain2.connect(osc2.frequency);

  const gate2 = audioCtx.createGain();
  gate2.gain.value = 0;
  const rhythm2 = audioCtx.createOscillator();
  const rhythmGain2 = audioCtx.createGain();
  rhythm2.frequency.value = 1.1; // offset from frog 1
  rhythmGain2.gain.value = 1.0;
  rhythm2.connect(rhythmGain2);
  rhythmGain2.connect(gate2.gain);

  frogGain = audioCtx.createGain();
  frogGain.gain.value = 0; // starts silent

  osc1.connect(gate1);
  gate1.connect(frogGain);
  osc2.connect(gate2);
  gate2.connect(frogGain);
  frogGain.connect(output);

  osc1.start();
  wobble1.start();
  rhythm1.start();
  osc2.start();
  wobble2.start();
  rhythm2.start();
}

/** Create ambient beetle clicking — soft rhythmic ticking that scales with beetle count. */
function createBeetleClick(audioCtx: AudioContext, output: GainNode): void {
  // Sharp clicks via short high-freq oscillator bursts
  const osc = audioCtx.createOscillator();
  osc.type = 'square';
  osc.frequency.value = 800;

  // Rapid rhythmic gating: ~6Hz gives quick tick-tick-tick
  const gate = audioCtx.createGain();
  gate.gain.value = 0;
  const rhythm = audioCtx.createOscillator();
  const rhythmGain = audioCtx.createGain();
  rhythm.frequency.value = 6;
  rhythmGain.gain.value = 1.0;
  rhythm.connect(rhythmGain);
  rhythmGain.connect(gate.gain);

  // Highpass filter to make it clicky, not tonal
  const hpf = audioCtx.createBiquadFilter();
  hpf.type = 'highpass';
  hpf.frequency.value = 2000;
  hpf.Q.value = 5;

  beetleGain = audioCtx.createGain();
  beetleGain.gain.value = 0; // starts silent

  osc.connect(hpf);
  hpf.connect(gate);
  gate.connect(beetleGain);
  beetleGain.connect(output);

  osc.start();
  rhythm.start();
}

/** Initialize ambient audio (call once). Starts silent, fades in on interaction. */
export function initAmbientAudio(): void {
  if (started) return;
  started = true;

  // Create audio context on user interaction (browser autoplay policy)
  const startAudio = () => {
    if (ctx) return;
    ctx = new AudioContext();
    masterGain = ctx.createGain();
    masterGain.gain.value = 0; // start silent
    masterGain.connect(ctx.destination);

    createWaterSound(ctx, masterGain);
    createRainSound(ctx, masterGain);
    createCricketSound(ctx, masterGain);
    createWindSound(ctx, masterGain);
    createLeafRustleSound(ctx, masterGain);
    createPollinatorHum(ctx, masterGain);
    createFrogChorus(ctx, masterGain);
    createBeetleClick(ctx, masterGain);

    // Fade in over 3 seconds
    masterGain.gain.linearRampToValueAtTime(1.0, ctx.currentTime + 3);

    // Remove listener after first interaction
    document.removeEventListener('click', startAudio);
    document.removeEventListener('touchstart', startAudio);
    document.removeEventListener('keydown', startAudio);
  };

  document.addEventListener('click', startAudio, { once: false });
  document.addEventListener('touchstart', startAudio, { once: false });
  document.addEventListener('keydown', startAudio, { once: false });
}

/** Set master volume (0-1) */
export function setAmbientVolume(vol: number): void {
  if (masterGain && ctx) {
    masterGain.gain.linearRampToValueAtTime(
      Math.max(0, Math.min(1, vol)),
      ctx.currentTime + 0.1,
    );
  }
}

/** Fade rain sound in or out (call when weather state changes) */
export function setRaining(raining: boolean): void {
  if (raining === isRaining) return;
  isRaining = raining;
  if (rainGain && ctx) {
    const target = raining ? 0.12 : 0; // rain is quieter than spring
    rainGain.gain.linearRampToValueAtTime(target, ctx.currentTime + 2);
  }
}

/** Set wind intensity (0–1). Maps to volume and filter frequency.
 *  Call when weather wind strength changes. */
export function setWindAmbient(strength: number): void {
  if (!ctx) return;
  // Volume: 0.02 (calm) to 0.08 (gusty)
  if (windGain) {
    const vol = 0.02 + strength * 0.06;
    windGain.gain.linearRampToValueAtTime(vol, ctx.currentTime + 1);
  }
  // Filter: higher wind = higher cutoff (more audible rush)
  if (windFilter) {
    const freq = 200 + strength * 600; // 200Hz (calm) to 800Hz (gusty)
    windFilter.frequency.linearRampToValueAtTime(freq, ctx.currentTime + 1);
  }
}

/** Set leaf rustle intensity based on foliage count and wind strength.
 *  More trees + more wind = louder shimmering rustle. */
export function setLeafRustle(foliageCount: number, windStrength: number): void {
  if (!ctx) return;
  // foliageCount: 0 → no rustle, 5000+ → full coverage
  const coverage = Math.min(1, foliageCount / 5000);
  // wind strength: 0 → barely audible, 1 → full rustle
  const windFactor = 0.3 + windStrength * 0.7;
  const vol = coverage * windFactor * 0.05; // max 0.05 — subtle background
  if (rustleGain) {
    rustleGain.gain.linearRampToValueAtTime(vol, ctx.currentTime + 0.5);
  }
  // Higher wind pushes filter frequency up — more bright shimmer
  if (rustleFilter) {
    const freq = 2000 + windStrength * 2000; // 2000Hz calm → 4000Hz gusty
    rustleFilter.frequency.linearRampToValueAtTime(freq, ctx.currentTime + 0.5);
  }
}

/** Set pollinator hum intensity based on active bee/butterfly count.
 *  More pollinators = louder background buzz. */
export function setPollinatorHum(pollinatorCount: number): void {
  if (!ctx || !pollinatorGain) return;
  // 0 pollinators = silent, 5+ = full hum (very quiet — background texture)
  const vol = Math.min(1, pollinatorCount / 5) * 0.015;
  pollinatorGain.gain.linearRampToValueAtTime(vol, ctx.currentTime + 1);
}

/** Set water babble intensity based on total water voxel count.
 *  More water channels = stronger spring babble. */
export function setWaterBabble(waterCount: number): void {
  if (!ctx || !waterSoundGain) return;
  // 0 water = base 0.04, 200+ = full 0.12
  const vol = 0.04 + Math.min(1, waterCount / 200) * 0.08;
  waterSoundGain.gain.linearRampToValueAtTime(vol, ctx.currentTime + 1);
}

/** Set beetle clicking intensity based on active beetle count.
 *  More beetles = louder clicking. Daytime only (beetles are diurnal). */
export function setBeetleClick(beetleCount: number, dayTime: number): void {
  if (!ctx || !beetleGain) return;
  const isDaytime = dayTime >= 0.25 && dayTime <= 0.75;
  const vol = isDaytime ? Math.min(1, beetleCount / 3) * 0.008 : 0;
  beetleGain.gain.linearRampToValueAtTime(vol, ctx.currentTime + 1);
}

/** Set frog chorus intensity based on water count and time of day.
 *  Frogs croak at dusk/night when water is present. */
export function setFrogChorus(waterCount: number, dayTime: number): void {
  if (!ctx || !frogGain) return;
  const isDusk = dayTime >= 0.60 || dayTime < 0.08;
  const hasWater = waterCount > 20;
  const shouldCroak = isDusk && hasWater;
  if (shouldCroak === frogsActive) return;
  frogsActive = shouldCroak;
  const target = shouldCroak ? 0.02 : 0; // very quiet — background texture
  frogGain.gain.linearRampToValueAtTime(target, ctx.currentTime + 3);
}

/** Fade cricket sounds in for dusk/night, out for day.
 *  Call each frame with the day cycle time (0–1). */
export function setNightAmbient(dayTime: number): void {
  const shouldChirp = dayTime >= 0.65 || dayTime < 0.05;
  if (shouldChirp === isNight) return;
  isNight = shouldChirp;
  if (cricketGain && ctx) {
    const target = shouldChirp ? 0.03 : 0; // very quiet — background texture
    cricketGain.gain.linearRampToValueAtTime(target, ctx.currentTime + 3);
  }
}
