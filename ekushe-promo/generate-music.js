const fs = require('fs');
const path = require('path');

const SAMPLE_RATE = 44100;
const DURATION = 45; // 45 seconds
const TOTAL_SAMPLES = SAMPLE_RATE * DURATION;

// Buffers for Left and Right channels (32-bit float for high dynamic range mixing)
const leftChannel = new Float32Array(TOTAL_SAMPLES);
const rightChannel = new Float32Array(TOTAL_SAMPLES);

// Frequency table for notes
const noteFreq = {
  'C2': 65.41, 'D2': 73.42, 'E2': 82.41, 'F2': 87.31, 'G2': 98.00, 'A2': 110.00, 'Bb2': 116.54, 'C3': 130.81,
  'D3': 146.83, 'E3': 164.81, 'F3': 174.61, 'G3': 196.00, 'A3': 220.00, 'Bb3': 233.08, 'C4': 261.63,
  'D4': 293.66, 'E4': 329.63, 'F4': 349.23, 'G4': 392.00, 'A4': 440.00, 'Bb4': 466.16, 'C5': 523.25,
  'D5': 587.33, 'E5': 659.25, 'F5': 698.46, 'G5': 783.99, 'A5': 880.00, 'Bb5': 932.33, 'C6': 1046.50
};

// Helper: Synthesize Piano-like note (harmonics with exponential decay and slight detune)
function addPianoNote(noteName, startTime, duration, velocity = 0.5, pan = 0.0) {
  const freq = noteFreq[noteName] || 440;
  const startSample = Math.floor(startTime * SAMPLE_RATE);
  const noteSamples = Math.floor(duration * SAMPLE_RATE);

  for (let s = 0; s < noteSamples; s++) {
    const idx = startSample + s;
    if (idx >= TOTAL_SAMPLES) break;
    const t = s / SAMPLE_RATE;

    // Multi-harmonic additive synthesis mimicking acoustic piano hammer strike
    const env = Math.exp(-t * 2.5) * (1 - Math.exp(-t * 250)); // Fast attack, natural decay
    const h1 = Math.sin(2 * Math.PI * freq * t);
    const h2 = 0.5 * Math.sin(2 * Math.PI * (freq * 2.001) * t) * Math.exp(-t * 3.5);
    const h3 = 0.25 * Math.sin(2 * Math.PI * (freq * 3.002) * t) * Math.exp(-t * 5.0);
    const h4 = 0.12 * Math.sin(2 * Math.PI * (freq * 4.004) * t) * Math.exp(-t * 7.0);
    const h5 = 0.06 * Math.sin(2 * Math.PI * (freq * 5.006) * t) * Math.exp(-t * 9.0);

    const sampleVal = (h1 + h2 + h3 + h4 + h5) * env * velocity * 0.4;

    const leftGain = Math.cos((pan + 1) * Math.PI / 4);
    const rightGain = Math.sin((pan + 1) * Math.PI / 4);

    leftChannel[idx] += sampleVal * leftGain;
    rightChannel[idx] += sampleVal * rightGain;
  }
}

// Helper: Warm ambient pad chord
function addAmbientPad(notes, startTime, duration, volume = 0.2) {
  const startSample = Math.floor(startTime * SAMPLE_RATE);
  const padSamples = Math.floor(duration * SAMPLE_RATE);

  for (let s = 0; s < padSamples; s++) {
    const idx = startSample + s;
    if (idx >= TOTAL_SAMPLES) break;
    const t = s / SAMPLE_RATE;

    // Smooth envelope: 2s attack, sustained, 2s release
    let env = 1.0;
    if (t < 2.0) env = t / 2.0;
    else if (t > duration - 2.0) env = Math.max(0, (duration - t) / 2.0);

    let chordVal = 0;
    notes.forEach((n, nIdx) => {
      const f = noteFreq[n] || 220;
      // Slight chorus/detune
      const wave1 = Math.sin(2 * Math.PI * (f * 0.999) * t);
      const wave2 = Math.sin(2 * Math.PI * (f * 1.001) * t);
      const wave3 = 0.4 * Math.sin(2 * Math.PI * (f * 2) * t);
      chordVal += (wave1 + wave2 + wave3) / (notes.length * 1.5);
    });

    // Gentle stereo panning flutter
    const panOffset = Math.sin(t * 0.5) * 0.2;
    leftChannel[idx] += chordVal * env * volume * (0.8 - panOffset);
    rightChannel[idx] += chordVal * env * volume * (0.8 + panOffset);
  }
}

// Helper: Modern Sub Bass & Kick
function addBassNote(noteName, startTime, duration, volume = 0.4) {
  const freq = noteFreq[noteName] || 73.42;
  const startSample = Math.floor(startTime * SAMPLE_RATE);
  const bassSamples = Math.floor(duration * SAMPLE_RATE);

  for (let s = 0; s < bassSamples; s++) {
    const idx = startSample + s;
    if (idx >= TOTAL_SAMPLES) break;
    const t = s / SAMPLE_RATE;
    const env = Math.exp(-t * 1.2) * (1 - Math.exp(-t * 60));
    const sub = Math.sin(2 * Math.PI * freq * t) + 0.3 * Math.sin(2 * Math.PI * freq * 2 * t);
    const val = sub * env * volume * 0.5;
    leftChannel[idx] += val * 0.7;
    rightChannel[idx] += val * 0.7;
  }
}

function addKick(startTime, volume = 0.6) {
  const startSample = Math.floor(startTime * SAMPLE_RATE);
  const kickSamples = Math.floor(0.4 * SAMPLE_RATE);

  for (let s = 0; s < kickSamples; s++) {
    const idx = startSample + s;
    if (idx >= TOTAL_SAMPLES) break;
    const t = s / SAMPLE_RATE;
    const f = 150 * Math.exp(-t * 30) + 40; // Pitch drop
    const env = Math.exp(-t * 9);
    const val = Math.sin(2 * Math.PI * f * t) * env * volume;
    leftChannel[idx] += val * 0.8;
    rightChannel[idx] += val * 0.8;
  }
}

function addHiHat(startTime, volume = 0.2, pan = 0.3) {
  const startSample = Math.floor(startTime * SAMPLE_RATE);
  const hatSamples = Math.floor(0.08 * SAMPLE_RATE);

  for (let s = 0; s < hatSamples; s++) {
    const idx = startSample + s;
    if (idx >= TOTAL_SAMPLES) break;
    const t = s / SAMPLE_RATE;
    const noise = (Math.random() * 2 - 1) * Math.exp(-t * 65);
    const val = noise * volume * 0.3;
    leftChannel[idx] += val * (1 - pan);
    rightChannel[idx] += val * (1 + pan);
  }
}

console.log("Synthesizing Cinematic Soundtrack for Ekushe Promo...");

// 1. Ambient Pads throughout the 45 seconds (Progression: Dm -> Gm -> Bb -> A -> Dm -> F -> C -> Bb -> Dm)
addAmbientPad(['D2', 'A2', 'F3', 'A3', 'D4'], 0.0, 10.5, 0.28);
addAmbientPad(['Bb2', 'F3', 'D4', 'F4'], 10.0, 8.5, 0.25);
addAmbientPad(['C3', 'G3', 'E4', 'G4'], 18.0, 6.0, 0.25);
addAmbientPad(['D3', 'A3', 'F4', 'A4'], 23.5, 6.5, 0.28);
addAmbientPad(['Bb2', 'D3', 'F3', 'Bb3', 'D4'], 29.5, 6.0, 0.30);
addAmbientPad(['C3', 'G3', 'C4', 'E4', 'G4'], 35.0, 5.0, 0.30);
addAmbientPad(['D2', 'A2', 'D3', 'F#3', 'A3', 'D4'], 39.5, 5.5, 0.32); // Picardy Major ending swell

// 2. Piano Theme — Part 1: Solemn Remembrance of 1952 (0s - 12s)
// "আমার ভাইয়ের রক্তে রাঙানো" inspired evocative motifs
const introMelody = [
  { n: 'D4', t: 1.0, d: 2.0, v: 0.7, p: -0.2 },
  { n: 'F4', t: 2.5, d: 1.5, v: 0.65, p: -0.1 },
  { n: 'E4', t: 4.0, d: 1.2, v: 0.6, p: 0.0 },
  { n: 'D4', t: 5.2, d: 2.2, v: 0.7, p: 0.1 },
  { n: 'A3', t: 7.2, d: 1.8, v: 0.55, p: -0.2 },
  { n: 'C4', t: 8.8, d: 1.2, v: 0.6, p: 0.0 },
  { n: 'D4', t: 9.8, d: 2.5, v: 0.75, p: 0.2 },
];
introMelody.forEach(m => addPianoNote(m.n, m.t, m.d, m.v, m.p));

// 3. Piano Theme — Part 2: Intricate Bengali Letters & Conjuncts (12s - 22s)
const middleMelody = [
  { n: 'D4', t: 12.0, d: 1.0, v: 0.65 },
  { n: 'F4', t: 13.0, d: 1.0, v: 0.7 },
  { n: 'A4', t: 14.0, d: 1.8, v: 0.75 },
  { n: 'Bb4', t: 15.5, d: 1.0, v: 0.7 },
  { n: 'A4', t: 16.5, d: 1.2, v: 0.65 },
  { n: 'G4', t: 17.5, d: 1.2, v: 0.65 },
  { n: 'F4', t: 18.5, d: 1.0, v: 0.6 },
  { n: 'E4', t: 19.5, d: 1.8, v: 0.7 },
  { n: 'D4', t: 21.0, d: 2.5, v: 0.8 },
];
middleMelody.forEach(m => addPianoNote(m.n, m.t, m.d, m.v, m.p || 0.0));

// Accompaniment arpeggio in mid section
[12.5, 14.5, 16.5, 18.5, 20.5].forEach((t, i) => {
  addPianoNote('D3', t, 2.0, 0.45, -0.3);
  addPianoNote('A3', t + 0.3, 1.8, 0.4, -0.1);
  addPianoNote('F4', t + 0.6, 1.5, 0.4, 0.2);
});

// 4. Part 3: The Rebirth (Ekushe Modern App Entrance) (22s - 38s)
// Modern rhythmic kick & percussion pulse
for (let t = 22.0; t < 38.0; t += 1.0) {
  addKick(t, 0.7);
  addHiHat(t + 0.5, 0.25, 0.2);
  addHiHat(t + 0.25, 0.15, -0.2);
  addHiHat(t + 0.75, 0.15, 0.2);
}

// Bassline
const bassGroove = [
  { n: 'D2', t: 22.0, d: 1.8 },
  { n: 'D2', t: 24.0, d: 1.8 },
  { n: 'Bb2', t: 26.0, d: 1.8 },
  { n: 'C3', t: 28.0, d: 1.8 },
  { n: 'D2', t: 30.0, d: 1.8 },
  { n: 'F2', t: 32.0, d: 1.8 },
  { n: 'G2', t: 34.0, d: 1.8 },
  { n: 'A2', t: 36.0, d: 1.8 },
];
bassGroove.forEach(b => addBassNote(b.n, b.t, b.d, 0.7));

// Uplifting high energy piano & synth bells
const techTheme = [
  { n: 'D5', t: 22.0, d: 0.6, v: 0.7 },
  { n: 'F5', t: 22.6, d: 0.6, v: 0.75 },
  { n: 'A5', t: 23.2, d: 1.2, v: 0.85 },
  { n: 'G5', t: 24.4, d: 0.8, v: 0.7 },
  { n: 'F5', t: 25.2, d: 0.8, v: 0.7 },
  { n: 'D5', t: 26.0, d: 1.5, v: 0.8 },

  { n: 'Bb4', t: 27.5, d: 0.6, v: 0.7 },
  { n: 'D5', t: 28.1, d: 0.6, v: 0.75 },
  { n: 'F5', t: 28.7, d: 1.2, v: 0.85 },
  { n: 'E5', t: 29.9, d: 1.0, v: 0.7 },

  { n: 'D5', t: 31.0, d: 0.6, v: 0.75 },
  { n: 'F5', t: 31.6, d: 0.6, v: 0.8 },
  { n: 'A5', t: 32.2, d: 1.5, v: 0.9 },
  { n: 'Bb5', t: 33.7, d: 1.0, v: 0.85 },
  { n: 'A5', t: 34.7, d: 1.0, v: 0.8 },
  { n: 'F5', t: 35.7, d: 1.0, v: 0.75 },
  { n: 'E5', t: 36.7, d: 1.5, v: 0.8 },
];
techTheme.forEach(m => addPianoNote(m.n, m.t, m.d, m.v, 0.1));

// 5. Part 4: Grand Finale / Outro (38s - 45s)
const outroChords = [
  { n: 'D4', t: 38.5, d: 4.5, v: 0.85, p: -0.2 },
  { n: 'F#4', t: 38.6, d: 4.5, v: 0.85, p: 0.0 },
  { n: 'A4', t: 38.7, d: 5.0, v: 0.9, p: 0.2 },
  { n: 'D5', t: 38.8, d: 5.5, v: 0.95, p: 0.0 },
];
outroChords.forEach(m => addPianoNote(m.n, m.t, m.d, m.v, m.p));
addBassNote('D2', 38.5, 5.5, 0.8);

// Apply Simple Stereo Reverb / Convolution-like Delay Effect
console.log("Applying stereo acoustic reverb...");
const delaySamples = Math.floor(0.28 * SAMPLE_RATE);
const feedback = 0.35;
for (let i = delaySamples; i < TOTAL_SAMPLES; i++) {
  leftChannel[i] += rightChannel[i - delaySamples] * feedback;
  rightChannel[i] += leftChannel[i - delaySamples] * feedback;
}

// Normalize and Master Audio (Soft Limiter)
let maxPeak = 0;
for (let i = 0; i < TOTAL_SAMPLES; i++) {
  const l = Math.abs(leftChannel[i]);
  const r = Math.abs(rightChannel[i]);
  if (l > maxPeak) maxPeak = l;
  if (r > maxPeak) maxPeak = r;
}

console.log(`Peak audio amplitude: ${maxPeak.toFixed(3)}. Normalizing to -1.0 dB...`);
const targetPeak = 0.89; // -1 dBFS
const gain = maxPeak > 0.001 ? targetPeak / maxPeak : 1.0;

// Fade in (first 1.5s) and fade out (last 2s)
for (let i = 0; i < TOTAL_SAMPLES; i++) {
  const t = i / SAMPLE_RATE;
  let envelope = 1.0;
  if (t < 1.0) envelope = t / 1.0;
  else if (t > DURATION - 2.0) envelope = Math.max(0, (DURATION - t) / 2.0);

  leftChannel[i] = Math.tanh(leftChannel[i] * gain * envelope);
  rightChannel[i] = Math.tanh(rightChannel[i] * gain * envelope);
}

// Write 16-bit PCM WAV File
const wavHeader = Buffer.alloc(44);
const dataSize = TOTAL_SAMPLES * 2 * 2; // 2 channels * 2 bytes (16-bit)
const fileSize = 36 + dataSize;

// RIFF chunk descriptor
wavHeader.write('RIFF', 0);
wavHeader.writeUInt32LE(fileSize, 4);
wavHeader.write('WAVE', 8);

// fmt sub-chunk
wavHeader.write('fmt ', 12);
wavHeader.writeUInt32LE(16, 16); // Subchunk1Size (16 for PCM)
wavHeader.writeUInt16LE(1, 20); // AudioFormat (1 = PCM)
wavHeader.writeUInt16LE(2, 22); // NumChannels (2 = Stereo)
wavHeader.writeUInt32LE(SAMPLE_RATE, 24); // SampleRate
wavHeader.writeUInt32LE(SAMPLE_RATE * 2 * 2, 28); // ByteRate
wavHeader.writeUInt16LE(4, 32); // BlockAlign
wavHeader.writeUInt16LE(16, 34); // BitsPerSample (16-bit)

// data sub-chunk
wavHeader.write('data', 36);
wavHeader.writeUInt32LE(dataSize, 40);

const pcmBuffer = Buffer.alloc(dataSize);
let offset = 0;
for (let i = 0; i < TOTAL_SAMPLES; i++) {
  // Convert [-1.0, 1.0] float to 16-bit signed integer [-32768, 32767]
  const lInt = Math.max(-32768, Math.min(32767, Math.floor(leftChannel[i] * 32767)));
  const rInt = Math.max(-32768, Math.min(32767, Math.floor(rightChannel[i] * 32767)));

  pcmBuffer.writeInt16LE(lInt, offset);
  offset += 2;
  pcmBuffer.writeInt16LE(rInt, offset);
  offset += 2;
}

const outPath = path.join(__dirname, 'public', 'ekushe-music.wav');
fs.writeFileSync(outPath, Buffer.concat([wavHeader, pcmBuffer]));
console.log(`✓ Audio successfully created at: ${outPath} (${(fs.statSync(outPath).size / (1024 * 1024)).toFixed(2)} MB)`);
