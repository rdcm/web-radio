const stationNames = {};
let currentModulation = "fm";

function loadStations(modulation) {
    fetch(`${CONFIG.apiBase}/stations/${modulation}`)
        .then(r => r.json())
        .then(list => {
            for (const key of Object.keys(stationNames)) delete stationNames[key];
            for (const s of list) stationNames[s.freq] = s.name;
            drawOverlay();
        });
}

loadStations(currentModulation);

let ws = null;
let actx = null;
let nextTime = 0;
let activeFreq = null;
let spectrumWs = null;

function selectModulation(modulation) {
    if (modulation === currentModulation) return;
    currentModulation = modulation;

    document.querySelectorAll(".mod-btn").forEach(b => b.classList.remove("active"));
    document.getElementById(`btn-${modulation}`).classList.add("active");

    ws?.close();
    actx?.close();
    activeFreq = null;
    setStatus("—", null);
    document.getElementById("rds-ps").textContent = "";

    loadStations(modulation);
    initWaterfall();
}

function snapToStation(freq) {
    const freqs = Object.keys(stationNames).map(Number);
    if (freqs.length === 0) return null;
    return freqs.reduce((a, b) => Math.abs(b - freq) < Math.abs(a - freq) ? b : a);
}

function tune(freq, el = null) {
    if (freq === activeFreq) return;

    ws?.close();
    actx?.close();

    activeFreq = freq;
    drawOverlay();
    setStatus(`On Air — ${(freq / 1e6).toFixed(3)} MHz 🔴`, currentModulation);
    const configName = stationNames[freq];
    document.getElementById("rds-ps").textContent = configName || "";

    actx = new AudioContext({ sampleRate: CONFIG.sampleRate });
    nextTime = actx.currentTime + 0.3;

    const thisWs = ws = new WebSocket(`${CONFIG.wsBase}/ws/listen/${currentModulation}`);
    thisWs.binaryType = "arraybuffer";
    thisWs.onopen = () => {
        thisWs.send(msgpack.encode({ freq: freq }));
    };

    thisWs.onmessage = ({ data }) => {
        const msg = msgpack.decode(new Uint8Array(data));
        const pcmBytes = new Uint8Array(msg.pcm);
        const pcm = new Int16Array(pcmBytes.buffer);
        const f32 = new Float32Array(pcm.length);
        for (let i = 0; i < pcm.length; i++) f32[i] = pcm[i] / 32768;

        const buf = actx.createBuffer(1, f32.length, CONFIG.sampleRate);
        buf.copyToChannel(f32, 0);

        const src = actx.createBufferSource();
        src.buffer = buf;
        src.connect(actx.destination);

        if (nextTime < actx.currentTime) nextTime = actx.currentTime + 0.2;
        src.start(nextTime);
        nextTime += buf.duration;
    };

    thisWs.onclose = () => { if (ws === thisWs) { activeFreq = null; setStatus("—", null); } };
}


function setStatus(text, modulation) {
    document.getElementById("status").textContent = text;
    const badge = document.getElementById("modulation-badge");
    if (modulation) {
        badge.textContent = modulation;
        badge.classList.add("visible");
    } else {
        badge.classList.remove("visible");
    }
}

// Waterfall

const DB_MIN = CONFIG.dbMin;
const DB_MAX = CONFIG.dbMax;
const COLORMAP = [
    [  0,   0,  15],
    [  0,   0,  80],
    [  0,  30, 200],
    [  0, 160, 255],
    [  0, 255, 200],
    [  0, 220,  80],
];

function dbToRgb(db) {
    const t = Math.max(0, Math.min(1, (db - DB_MIN) / (DB_MAX - DB_MIN)));
    const pos = t * (COLORMAP.length - 1);
    const i = Math.min(Math.floor(pos), COLORMAP.length - 2);
    const f = pos - i;
    const a = COLORMAP[i], b = COLORMAP[i + 1];
    return [
        Math.round(a[0] + f * (b[0] - a[0])),
        Math.round(a[1] + f * (b[1] - a[1])),
        Math.round(a[2] + f * (b[2] - a[2])),
    ];
}

let spectrumMeta = null;

function canvasXToFreq(canvas, clientX) {
    if (!spectrumMeta) return null;
    const { center_freq, sample_rate } = spectrumMeta;
    const rect = canvas.getBoundingClientRect();
    const t = (clientX - rect.left) / rect.width;
    return center_freq - sample_rate / 2 + t * sample_rate;
}

function drawOverlay() {
    if (!spectrumMeta) return;
    const { center_freq, sample_rate } = spectrumMeta;
    const freqStart = center_freq - sample_rate / 2;

    const overlay = document.getElementById("waterfall-overlay");
    const dpr = window.devicePixelRatio || 1;
    overlay.width  = Math.round((overlay.offsetWidth  || overlay.parentElement.clientWidth) * dpr);
    overlay.height = Math.round((overlay.offsetHeight || 200) * dpr);
    const ctx = overlay.getContext("2d");
    ctx.clearRect(0, 0, overlay.width, overlay.height);

    if (!activeFreq) return;
    const t = (activeFreq - freqStart) / sample_rate;
    if (t < 0 || t > 1) return;
    const x = Math.round(t * overlay.width) + 0.5;
    ctx.strokeStyle = "rgba(255, 60, 60, 0.9)";
    ctx.setLineDash([]);
    ctx.beginPath();
    ctx.moveTo(x, 0);
    ctx.lineTo(x, overlay.height);
    ctx.stroke();
}

function drawFreqScale() {
    if (!spectrumMeta) return;
    const { center_freq, sample_rate } = spectrumMeta;

    const canvas = document.getElementById("freq-scale");
    const dpr = window.devicePixelRatio || 1;
    canvas.width  = Math.round((canvas.offsetWidth || canvas.parentElement.clientWidth) * dpr);
    canvas.height = Math.round(24 * dpr);
    const ctx = canvas.getContext("2d");
    const w = canvas.width;
    const h = canvas.height;

    ctx.clearRect(0, 0, w, h);

    const freqStart = center_freq - sample_rate / 2;
    const freqEnd   = center_freq + sample_rate / 2;

    // minor ticks every 500 kHz
    ctx.strokeStyle = "#2a2a2a";
    ctx.lineWidth = 1;
    const minorStep = 500_000;
    for (let f = Math.ceil(freqStart / minorStep) * minorStep; f <= freqEnd; f += minorStep) {
        const x = Math.round((f - freqStart) / sample_rate * w) + 0.5;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h * 0.3);
        ctx.stroke();
    }

    // major ticks + labels every 1 MHz
    ctx.strokeStyle = "#444";
    ctx.fillStyle = "#555";
    ctx.font = `${Math.round(9 * dpr)}px monospace`;
    ctx.textAlign = "center";
    const majorStep = 1_000_000;
    for (let f = Math.ceil(freqStart / majorStep) * majorStep; f <= freqEnd; f += majorStep) {
        const x = Math.round((f - freqStart) / sample_rate * w) + 0.5;
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h * 0.45);
        ctx.stroke();
        ctx.fillText(`${(f / 1e6).toFixed(0)}`, x, h * 0.95);
    }
}

let waterfallRowQueue = [];
let waterfallInitialized = false;
let waterfallAnimId = null;

function initWaterfall() {
    const canvas = document.getElementById("waterfall");
    const dpr = window.devicePixelRatio || 1;
    const cssWidth  = canvas.offsetWidth  || 900;
    const cssHeight = 200;
    canvas.width  = Math.round(cssWidth  * dpr);
    canvas.height = Math.round(cssHeight * dpr);
    canvas.style.width  = cssWidth  + "px";
    canvas.style.height = cssHeight + "px";
    const ctx = canvas.getContext("2d");
    const width  = canvas.width;
    const height = canvas.height;

    canvas.style.cursor = "crosshair";

    const hoverEl = document.getElementById("hover-freq");

    canvas.addEventListener("mousemove", (e) => {
        const freq = canvasXToFreq(canvas, e.clientX);
        hoverEl.textContent = freq ? `${(freq / 1e6).toFixed(3)} MHz` : "";
    });

    canvas.addEventListener("mouseleave", () => {
        hoverEl.textContent = "";
    });

    canvas.addEventListener("click", (e) => {
        const freq = canvasXToFreq(canvas, e.clientX);
        if (!freq) return;
        const station = snapToStation(freq);
        if (station !== null) tune(station, null);
    });

    if (waterfallAnimId !== null) cancelAnimationFrame(waterfallAnimId);
    spectrumMeta = null;
    waterfallRowQueue = [];
    waterfallInitialized = false;
    spectrumWs?.close();
    spectrumWs = new WebSocket(`${CONFIG.wsBase}/ws/spectrum/${currentModulation}`);
    spectrumWs.binaryType = "arraybuffer";
    spectrumWs.onopen = () => spectrumWs.send(msgpack.encode({}));

    const offscreen = document.createElement("canvas");
    offscreen.width  = width;
    offscreen.height = height;
    const offCtx = offscreen.getContext("2d");

    const rowData = ctx.createImageData(width, 1);

    const SCROLL_PPS = 20;
    let lastTs  = null;
    let accumPx = 0;

    (function renderLoop(ts) {
        waterfallAnimId = requestAnimationFrame(renderLoop);
        if (lastTs === null) { lastTs = ts; return; }

        const dt = Math.min((ts - lastTs) / 1000, 0.1);
        lastTs = ts;

        if (waterfallRowQueue.length === 0) return;

        accumPx += SCROLL_PPS * dt;
        const steps = Math.floor(accumPx);
        if (steps === 0) return;
        accumPx -= steps;

        const draw = Math.min(steps, waterfallRowQueue.length);

        ctx.drawImage(offscreen, 0, 0, width, height - draw, 0, draw, width, height - draw);

        for (let i = draw - 1; i >= 0; i--) {
            rowData.data.set(waterfallRowQueue.shift());
            ctx.putImageData(rowData, 0, i);
        }

        offCtx.drawImage(canvas, 0, 0);
    })();

    spectrumWs.onmessage = ({ data }) => {
        const { bins, center_freq, sample_rate } = msgpack.decode(new Uint8Array(data));

        if (!waterfallInitialized) {
            spectrumMeta = { center_freq, sample_rate };
            drawFreqScale();
            drawOverlay();
            waterfallInitialized = true;
        }

        const row = new Uint8ClampedArray(width * 4);
        for (let x = 0; x < width; x++) {
            const binIdx = Math.round(x * bins.length / width);
            const [r, g, b] = dbToRgb(bins[binIdx]);
            const o = x * 4;
            row[o]     = r;
            row[o + 1] = g;
            row[o + 2] = b;
            row[o + 3] = 255;
        }
        if (waterfallRowQueue.length < 60) waterfallRowQueue.push(row);
    };
}

initWaterfall();
