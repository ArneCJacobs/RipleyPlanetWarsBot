# TODOs

- [ ] the way the final rankings are calculated aren't completely accurate, ripleybot has a lower rating then some of the bots it has won against.
- [ ] recalculate the score very time a new move is picked
- [ ] don't send all you can in one move, see how much a planet needs to send so a single planet can possibly sound out multiple expiditions
- [ ] add a score heuristic to prefer planets that are clustered together
- [ ] generate multiple options, and pick the best one with a map scoring scoring function:
    - for each projected friendly planet go over all other planets the score of each planet takes into account the distance to the current planet and the amount of units and a factor for friendly, neutral or hostile. Possibly adding other heuristics along as wel later (such as that planets proximity to other planet)
    - don't forget to add an option that isn't constrained by forces needed to survive (sometimes it might be better to take another enemy planet then to defent and lose it anyway)
    - also take into account that the score depents on the projected state, which in terms depends on the longest expidition, should this be made independent?

# Relevant commands
cargo build && planetwars-client --grpc-server-url  http://planetwars.zeus.gent:7000 ripleybot.toml simplebot --map hex
ipython --pdb -c "%run tournament.py"

https://git.zeus.gent/midgard/planetwars_client_python/
`planetwars_client ./target/debug/RipleyPlanetWarsBot silviator hunger_games`


# setup

## python
all python files in this repo have to be run by `uv run PYTHONSCRIPT`
first time running you must run `uv venv` to initialize the virtual env then `uv sync` to update dependencies


# relevant links

https://github.com/iasoon/planetwars.dev
https://github.com/ZeusWPI/planetwars-starterpack/tree/main?tab=readme-ov-file
https://planetwars.zeus.gent/docs/local-development
https://planetwars.zeus.gent/matches?before=2025-05-18T15%3A24%3A04.761857

https://mattermost.zeus.gent/zeus/channels/planetwars


