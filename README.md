cargo build && planetwars-client --grpc-server-url  http://planetwars.zeus.gent:7000 ripleybot.toml simplebot --map hex
ipython --pdb -c "%run tournament.py"

all python files in this repo have to be run by `uv run PYTHONSCRIPT`
first time running you must run `uv venv` to initialize the virtual env then `uv sync` to update dependencies

https://github.com/iasoon/planetwars.dev
https://github.com/ZeusWPI/planetwars-starterpack/tree/main?tab=readme-ov-file
https://planetwars.zeus.gent/docs/local-development
https://planetwars.zeus.gent/matches?before=2025-05-18T15%3A24%3A04.761857

https://mattermost.zeus.gent/zeus/channels/planetwars


