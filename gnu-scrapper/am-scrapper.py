import asyncio
import json
import time
import urllib.request
import numpy as np
import msgpack
import websockets
from gnuradio import gr, analog, filter as gr_filter
import osmosdr

from config import Config

send_queue:     asyncio.Queue = asyncio.Queue(maxsize=512)
spectrum_queue: asyncio.Queue = asyncio.Queue(maxsize=64)


class MultiSink(gr.sync_block):
    """Single Python block for all audio channels — avoids per-channel GIL overhead."""
    def __init__(self, freqs: list[int], gain: float, loop: asyncio.AbstractEventLoop):
        super().__init__("multi_sink", [np.float32] * len(freqs), [])
        self.freqs = freqs
        self.gain  = gain
        self.loop  = loop

    def work(self, input_items, _):
        n = len(input_items[0])
        for samples, freq in zip(input_items, self.freqs):
            pcm = np.clip(samples * self.gain * 32767, -32768, 32767).astype(np.int16).tobytes()
            asyncio.run_coroutine_threadsafe(
                send_queue.put(msgpack.packb({"freq": freq, "pcm": pcm})),
                self.loop,
            )
        return n


class SpectrumSink(gr.sync_block):
    def __init__(self, config: Config, loop: asyncio.AbstractEventLoop):
        super().__init__("spectrum_sink", [np.complex64], [])
        self.config = config
        self.loop   = loop
        self.window = np.hanning(config.spectrum_fft_size).astype(np.float32)
        self._last_send = 0.0
        self._interval  = 1.0 / config.spectrum_fps

    def work(self, input_items, _):
        samples  = input_items[0]
        now      = time.monotonic()
        fft_size = self.config.spectrum_fft_size
        if now - self._last_send >= self._interval and len(samples) >= fft_size:
            chunk    = samples[:fft_size] * self.window
            spectrum = np.fft.fftshift(np.abs(np.fft.fft(chunk))) / fft_size
            power_db = (20 * np.log10(spectrum + 1e-10)).tolist()
            msg = msgpack.packb({
                "center_freq": self.config.center_freq,
                "sample_rate": self.config.sample_rate,
                "bins":        power_db,
            })
            asyncio.run_coroutine_threadsafe(spectrum_queue.put(msg), self.loop)
            self._last_send = now
        return len(samples)


def build_flowgraph(config: Config, loop: asyncio.AbstractEventLoop) -> gr.top_block:
    tb = gr.top_block()
    tb._blocks = []

    src = osmosdr.source(args="hackrf=0")
    src.set_sample_rate(config.sample_rate)
    src.set_center_freq(config.center_freq)
    src.set_gain(40)
    src.set_if_gain(32)
    src.set_bb_gain(20)
    tb._blocks.append(src)

    chan_decim  = config.sample_rate // config.channel_rate
    audio_decim = config.channel_rate // config.audio_rate

    channel_taps = gr_filter.firdes.low_pass(1.0, config.sample_rate, 10_000, 5_000)

    sink = MultiSink([s.freq for s in config.stations], config.audio_gain, loop)
    tb._blocks.append(sink)

    for i, station in enumerate(config.stations):
        offset = station.freq - config.center_freq
        xlating = gr_filter.freq_xlating_fir_filter_ccf(
            chan_decim, channel_taps, offset, config.sample_rate,
        )
        demod = analog.am_demod_cf(
            channel_rate=config.channel_rate,
            audio_decim=audio_decim,
            audio_pass=3_500,
            audio_stop=4_000,
        )
        agc = analog.agc2_ff(1e-1, 1e-3, 0.5, 100.0)
        tb.connect(src, xlating, demod, agc, (sink, i))
        tb._blocks.append(agc)
        tb._blocks.extend([xlating, demod])
        print(f"[chain] {station.freq / 1e6:.3f} MHz  ({station.name})")

    spec_sink = SpectrumSink(config, loop)
    tb._blocks.append(spec_sink)
    tb.connect(src, spec_sink)

    return tb


async def audio_sender(ws):
    while True:
        msg = await send_queue.get()
        await ws.send(msg)


async def spectrum_sender(ws):
    while True:
        msg = await spectrum_queue.get()
        await ws.send(msg)


async def main():
    config = Config.load(config_path="config-am.toml", stations_path="stations-am.toml")

    stations_json = json.dumps([{"freq": s.freq, "name": s.name} for s in config.stations]).encode()
    urllib.request.urlopen(urllib.request.Request(
        f"{config.api_base}/stations/{config.modulation}",
        data=stations_json, method="POST",
        headers={"Content-Type": "application/json"},
    ))
    print(f"[stations] registered {len(config.stations)} stations")

    loop = asyncio.get_event_loop()

    async with (
        websockets.connect(f"{config.ws_base}/ws/ingest/{config.modulation}")          as audio_ws,
        websockets.connect(f"{config.ws_base}/ws/ingest/{config.modulation}/spectrum") as spectrum_ws,
    ):
        print("building flowgraph...")
        tb = build_flowgraph(config, loop)
        print("starting...")
        tb.start()
        print("started!")
        try:
            await asyncio.gather(
                audio_sender(audio_ws),
                spectrum_sender(spectrum_ws),
            )
        finally:
            tb.stop()
            tb.wait()


asyncio.run(main())
