import asyncio
import time
import numpy as np
import msgpack
import websockets
from gnuradio import gr, analog, filter as gr_filter
import osmosdr

from config import Config, Station

AUDIO_RATE = 48_000
SPECTRUM_FFT_SIZE = 1024
SPECTRUM_FPS = 20

send_queue: asyncio.Queue = asyncio.Queue(maxsize=512)
spectrum_queue: asyncio.Queue = asyncio.Queue(maxsize=64)


class StationSink(gr.sync_block):
    def __init__(self, station: Station, loop: asyncio.AbstractEventLoop):
        super().__init__(f"sink_{station.freq}", [np.float32], [])
        self.station = station
        self.loop = loop

    def work(self, input_items, _):
        samples = input_items[0].copy()
        pcm = (samples * 32767).clip(-32768, 32767).astype(np.int16).tobytes()
        msg = msgpack.packb({"freq": self.station.freq, "pcm": pcm})
        asyncio.run_coroutine_threadsafe(send_queue.put(msg), self.loop)
        return len(samples)


class SpectrumSink(gr.sync_block):
    def __init__(self, config: Config, loop: asyncio.AbstractEventLoop):
        super().__init__("spectrum_sink", [np.complex64], [])
        self.config = config
        self.loop = loop
        self.window = np.hanning(SPECTRUM_FFT_SIZE).astype(np.float32)
        self._last_send = 0.0
        self._interval = 1.0 / SPECTRUM_FPS

    def work(self, input_items, _):
        samples = input_items[0]
        now = time.monotonic()
        if now - self._last_send >= self._interval and len(samples) >= SPECTRUM_FFT_SIZE:
            chunk = samples[:SPECTRUM_FFT_SIZE] * self.window
            spectrum = np.fft.fftshift(np.abs(np.fft.fft(chunk))) / SPECTRUM_FFT_SIZE
            power_db = (20 * np.log10(spectrum + 1e-10)).tolist()
            msg = msgpack.packb({
                "center_freq": self.config.center_freq,
                "sample_rate": self.config.sample_rate,
                "bins": power_db,
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

    for station in config.stations:
        offset = station.freq - config.center_freq
        shifter = gr_filter.freq_xlating_fir_filter_ccf(
            1,
            gr_filter.firdes.low_pass(1.0, config.sample_rate, 100_000, 50_000),
            offset,
            config.sample_rate,
        )
        k = config.sample_rate / (2 * 3.14159 * 75000)
        quad = analog.quadrature_demod_cf(k)
        decim = config.sample_rate // AUDIO_RATE
        audio_taps = gr_filter.firdes.low_pass(1.0, config.sample_rate, 15_000, 8_000)
        lpf = gr_filter.fir_filter_fff(decim, audio_taps)
        sink = StationSink(station, loop)
        tb._blocks.extend([shifter, quad, lpf, sink])
        tb.connect(src, shifter, quad, lpf, sink)

    spectrum_sink = SpectrumSink(config, loop)
    tb._blocks.append(spectrum_sink)
    tb.connect(src, spectrum_sink)

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
    config = Config.load("stations-fm.toml")
    loop = asyncio.get_event_loop()

    async with (
        websockets.connect("ws://localhost:8020/ws/ingest") as audio_ws,
        websockets.connect("ws://localhost:8020/ws/ingest/spectrum") as spectrum_ws,
    ):
        print("building flowgraph...")
        tb = build_flowgraph(config, loop)
        print("starting...")
        tb.start()
        print("started!")
        try:
            await asyncio.gather(audio_sender(audio_ws), spectrum_sender(spectrum_ws))
        finally:
            tb.stop()
            tb.wait()


asyncio.run(main())
