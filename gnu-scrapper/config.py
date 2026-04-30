from dataclasses import dataclass
import tomllib


@dataclass(frozen=True)
class Station:
    freq: int
    name: str


@dataclass(frozen=True)
class Config:
    sample_rate: int
    channel_rate: int
    audio_rate: int
    spectrum_fft_size: int
    spectrum_fps: int
    api_base: str
    stations: list[Station]
    _center_freq: int | None = None

    @property
    def center_freq(self) -> int:
        if self._center_freq is not None:
            return self._center_freq
        freqs = [s.freq for s in self.stations]
        return (min(freqs) + max(freqs)) // 2

    @property
    def ws_base(self) -> str:
        return self.api_base.replace("http://", "ws://").replace("https://", "wss://")

    @classmethod
    def load(cls, config_path: str = "config-fm.toml", stations_path: str = "stations-fm.toml") -> "Config":
        with open(config_path, "rb") as f:
            cfg = tomllib.load(f)
        with open(stations_path, "rb") as f:
            sta = tomllib.load(f)
        return cls(
            sample_rate=cfg["sample_rate"],
            channel_rate=cfg["channel_rate"],
            audio_rate=cfg["audio_rate"],
            spectrum_fft_size=cfg["spectrum_fft_size"],
            spectrum_fps=cfg["spectrum_fps"],
            api_base=cfg["api_base"],
            stations=[Station(**s) for s in sta["stations"]],
            _center_freq=cfg.get("center_freq"),
        )
