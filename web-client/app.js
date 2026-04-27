const GROUPS = [
    {
        label: "FM",
        stations: [
            { name: "Radio 1", freq: 101_200_000 },
            { name: "Radio 2", freq: 101_800_000 },
            { name: "Radio 3", freq: 102_000_000 },
        ],
    },
    {
        label: "Aviation",
        stations: [
            { name: "North-1", freq: 122_700_000 },
            { name: "West-1",  freq: 123_400_000 },
            { name: "South-1", freq: 125_300_000 },
            { name: "North-2", freq: 127_200_000 },
            { name: "West-2",  freq: 128_000_000 },
            { name: "East-1",  freq: 129_800_000 },
            { name: "East-2",  freq: 131_200_000 },
            { name: "South-2", freq: 134_000_000 },
        ],
    },
];

const SAMPLE_RATE = 48_000;
let ws = null;
let actx = null;
let nextTime = 0;
let activeFreq = null;

const container = document.getElementById("groups");

GROUPS.forEach(group => {
    const groupEl = document.createElement("div");
    groupEl.className = "group";

    const labelEl = document.createElement("div");
    labelEl.className = "group-label";
    labelEl.textContent = group.label;
    groupEl.appendChild(labelEl);

    const stationsEl = document.createElement("div");
    stationsEl.className = "stations";

    group.stations.forEach(s => {
        const el = document.createElement("div");
        el.className = "station";
        el.dataset.freq = s.freq;
        el.innerHTML = `<span class="name">${s.name}</span>
                        <span class="freq">${(s.freq / 1e6).toFixed(1)} MHz</span>`;
        el.onclick = () => tune(s.freq, el);
        stationsEl.appendChild(el);
    });

    groupEl.appendChild(stationsEl);
    container.appendChild(groupEl);
});

function tune(freq, el) {
    if (freq === activeFreq) return;

    ws?.close();
    actx?.close();

    document.querySelectorAll(".station").forEach(e => e.classList.remove("active"));
    el.classList.add("active");
    activeFreq = freq;

    actx = new AudioContext({ sampleRate: SAMPLE_RATE });
    nextTime = actx.currentTime + 0.1;

    ws = new WebSocket("ws://localhost:8020/ws/listen");
    ws.binaryType = "arraybuffer";
    ws.onopen = () => {
        const sub = msgpack.encode({ freq: freq });
        ws.send(sub);
        setStatus(`On Air — ${(freq / 1e6).toFixed(1)} MHz 🔴`);
    };

    ws.onmessage = ({ data }) => {
        const msg = msgpack.decode(new Uint8Array(data));
        const pcmBytes = new Uint8Array(msg.pcm);
        const pcm = new Int16Array(pcmBytes.buffer);
        const f32 = new Float32Array(pcm.length);
        for (let i = 0; i < pcm.length; i++) f32[i] = pcm[i] / 32768;

        const buf = actx.createBuffer(1, f32.length, SAMPLE_RATE);
        buf.copyToChannel(f32, 0);

        const src = actx.createBufferSource();
        src.buffer = buf;
        src.connect(actx.destination);

        if (nextTime < actx.currentTime) nextTime = actx.currentTime + 0.05;
        src.start(nextTime);
        nextTime += buf.duration;
    };

    ws.onclose = () => setStatus("Disconnected");
}

function setStatus(t) {
    document.getElementById("status").textContent = t;
}

// Waterfall

const DB_MIN = -100;
const DB_MAX = -20;
const COLORMAP = [
    [  0,   0,   0],  // -100 dB: black
    [  0,   0, 128],  //          dark blue
    [  0,   0, 255],  //          blue
    [  0, 255, 255],  //          cyan
    [255, 255,   0],  //          yellow
    [255, 255, 255],  //  -20 dB: white
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

function initWaterfall() {
    const canvas = document.getElementById("waterfall");
    const ctx = canvas.getContext("2d");
    const width = canvas.width;
    const rowData = ctx.createImageData(width, 1);

    const ws = new WebSocket("ws://localhost:8020/ws/spectrum");
    ws.binaryType = "arraybuffer";
    ws.onopen = () => ws.send(msgpack.encode({}));
    ws.onmessage = ({ data }) => {
        const { bins } = msgpack.decode(new Uint8Array(data));

        ctx.drawImage(canvas, 0, 0, width, canvas.height - 1, 0, 1, width, canvas.height - 1);

        for (let x = 0; x < width; x++) {
            const binIdx = Math.round(x * bins.length / width);
            const [r, g, b] = dbToRgb(bins[binIdx]);
            const o = x * 4;
            rowData.data[o]     = r;
            rowData.data[o + 1] = g;
            rowData.data[o + 2] = b;
            rowData.data[o + 3] = 255;
        }
        ctx.putImageData(rowData, 0, 0);
    };
}

initWaterfall();
