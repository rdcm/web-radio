from dataclasses import dataclass
import tomllib


@dataclass(frozen=True)
class Station:
    freq: int
    name: str


@dataclass(frozen=True)
class Config:
    sample_rate: int
    stations: list[Station]

    @property
    def center_freq(self) -> int:
        freqs = [s.freq for s in self.stations]
        return (min(freqs) + max(freqs)) // 2

    @classmethod
    def load(cls, path: str = "stations.toml") -> "Config":
        with open(path, "rb") as f:
            raw = tomllib.load(f)
        return cls(
            sample_rate=raw["sample_rate"],
            stations=[Station(**s) for s in raw["stations"]],
        )