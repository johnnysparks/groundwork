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
let started = false;
let isRaining = false;
let isNight = false;

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

  const waterGain = audioCtx.createGain();
  waterGain.gain.value = 0.08; // quiet — background ambiance

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
