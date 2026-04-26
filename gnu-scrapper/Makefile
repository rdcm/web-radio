sync:
	uv venv --system-site-packages --python /usr/bin/python3
	uv sync --no-install-project

run:
	uv run fm-scrapper.py

install-deps:
	sudo dnf install gnuradio gr-osmosdr

hack-rf-info:
	hackrf_info

gdb:
	PYTHONFAULTHANDLER=1 uv run fm-scrapper.py