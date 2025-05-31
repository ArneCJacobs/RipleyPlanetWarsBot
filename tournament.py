from concurrent.futures import ThreadPoolExecutor, as_completed
from time import sleep
from typing import Literal
import pandas as pd
from trueskill import TrueSkill, Rating
import requests 
import subprocess
import pydantic
import datetime
from tqdm import tqdm

GRPC_SERVER_URL = "http://planetwars.zeus.gent:7000"
BOT_CONFIG = "ripleybot.toml"
MATCH_API_ENDPOINT = "https://planetwars.zeus.gent/api/matches"

class Player(pydantic.BaseModel):
    bot_version_id: int | None
    bot_id: int | None
    bot_name: str | None
    owner_id: int | None
    had_errors: bool | None

    @property 
    def id(self):
        return f"{self.owner_id}-{self.bot_name}"


class Map(pydantic.BaseModel):
    name: str

class Match(pydantic.BaseModel):
    id: int
    timestamp: datetime.datetime 
    state: Literal["Playing"] | Literal["Finished"]
    players: list[Player]
    winner: int | None
    map: Map


def main():
    latest_rankings = pd.read_csv("latest_ratings.csv")
    opponents = latest_rankings["bot_name"].tolist()
    maps = ['hex', 'spiral', 'hunger_games']
    own_bot = "ripleybot"

    simplebot_index = latest_rankings[latest_rankings["bot_name"] == "simplebot"].index
    if simplebot_index.empty:
        print("No simplebot found in the latest ratings. Exiting.")
        return
    try:
        simplebot_index = int(simplebot_index[0])  # pyright: ignore[reportArgumentType]
    except ValueError:
        print("Invalid simplebot index found in the latest ratings. Exiting.")
        return

    simplebot_index += 1 # include simplebot 

    progress_bar = tqdm(
        total=simplebot_index * len(maps),
        desc="Playing matches",
    )
    matches = []
    tasks = {}

    with ThreadPoolExecutor(max_workers=10) as executor:
        for opponent in opponents[:simplebot_index]:
            for map in maps:
                task = executor.submit(play_match, opponent, map)
                tasks[task] = (opponent, map)

        for task in as_completed(tasks):
            opponent, map = tasks[task]
            match: Match = task.result()
            # print(f"{match_id=}, {own_bot=} vs {opponent=}, {map=}, {result=}")
            matches.append(match)
            progress_bar.update(1)

        progress_bar.close()

    ts_env = TrueSkill(draw_probability=0.20)
    opponents_ratings = {
        opponent["bot_name"]: Rating(mu=opponent["rating_mu"], sigma=opponent["rating_sigma"])
        for opponent in latest_rankings.to_dict(orient="records")
    }
    opponents_ratings[own_bot] = ts_env.create_rating()
    rankings = process_match_results(matches, ts_env, opponents_ratings, own_bot)

    with open("matches.jsonl", "w") as f:
        for match in matches:
            f.write(match.model_dump_json() + "\n")

    matches_df = pd.DataFrame(rankings)
    matches_df.sort_values(by='timestamp', inplace=True)
    matches_df.to_parquet("matches.parquet", index=False)

    latest_ratings = matches_df.sort_values(by='timestamp').groupby('bot_name').last().reset_index()
    # sort by ratings 
    latest_ratings = latest_ratings.sort_values(by='rating_mu', ascending=False)
    #pretty print 
    print(latest_ratings[['bot_name', 'rating_mu']].to_string(index=False))

    latest_ratings.to_csv("tournament_ratings.csv", index=False)




def process_match_results(
    matches: list[Match],
    ts_env: TrueSkill,
    opponents_ratings: dict[str, Rating],
    own_bot: str,
) -> list[dict]:
    matches.sort(key=lambda x: x.timestamp)
    rankings = []

    for match in matches:
        # for a local match, own bot is always none
        opponent = next(player.bot_name for player in match.players if player.bot_name != None)

        if match.winner is None:
            result_text = "draw"
        elif match.players[match.winner].bot_name == opponent:
            result_text = "loss"
        else:
            result_text = "win"

        own_rating, opponent_rating = adjust_ratings(
            match,
            ts_env,
            opponents_ratings,
            own_bot,
            opponent,
        )

        rankings.append({
            'match_id': match.id,
            "timestamp": match.timestamp,
            "bot_name": own_bot,
            "rating_mu" : own_rating.mu,
            "rating_sigma": own_rating.sigma,
            "opponent": opponent,
            "map": match.map.name,
            "result": result_text,
        })

        rankings.append({
            'match_id': match.id,
            "timestamp": match.timestamp,
            "bot_name": opponent,
            "rating_mu" : opponent_rating.mu,
            "rating_sigma": opponent_rating.sigma,
            "opponent": own_bot,
            "map": match.map.name,
            "result": "draw" if result_text == "draw" else "loss" if result_text == "win" else "win",
        })

    return rankings




def adjust_ratings(
    match: Match,
    ts_env: TrueSkill,
    opponents_ratings: dict[str, Rating],
    own_bot: str,
    opponent_bot: str,
) -> tuple[Rating, Rating]:
    own_rating = opponents_ratings[own_bot]
    opponent_rating = opponents_ratings[opponent_bot]
    ratings_groups = [
        (opponent_rating,) if player.bot_name == opponent_bot else (own_rating,) 
        for player in match.players
    ]

    ranks = [
        index == match.winner for index, _player in enumerate(match.players)
    ]

    new_own_rating, new_opponent_rating = ts_env.rate(
        ratings_groups,
        ranks=ranks,
    )

    opponents_ratings[own_bot] = new_own_rating[0]
    opponents_ratings[opponent_bot] = new_opponent_rating[0]

    return new_own_rating[0], new_opponent_rating[0]

def play_match(
    opponent_bot: str,
    map_name: str,
) -> Match:
    cmd = subprocess.run(
        [
            "planetwars-client",
            "--grpc-server-url", GRPC_SERVER_URL,
            "--map", map_name,
            BOT_CONFIG,
            opponent_bot,
        ],
        capture_output=True,
        text=True,
    )

    if cmd.returncode != 0:
        print(f"Error running match: {cmd.stderr}")
        exit(1)


    match_stats = cmd.stdout.strip().split("\n")[-1]
    match_url = match_stats.split(" ")[-1]
    match_id = match_url.split("/")[-1]
    match_stat_url = f"{MATCH_API_ENDPOINT}/{match_id}/"

    params = {
        'content-type': 'application/json',
    }

    request = requests.get(match_stat_url, params)
    if request.status_code != 200:
        print(f"Error fetching match results: {request.text}")
        exit(1)


    match = Match.model_validate(request.json())
    while match.state == "Playing":
        sleep(1)
        request = requests.get(match_stat_url, params)
        match = Match.model_validate(request.json())

    return match

def repreocess_maches():
    latest_rankings = pd.read_csv("latest_ratings.csv")
    own_bot = "ripleybot"
    matches = []
    with open("matches.jsonl", "r") as f:
        for line in f:
            match = Match.model_validate_json(line)
            matches.append(match)

    ts_env = TrueSkill(draw_probability=0.20)
    opponents_ratings = {
        opponent["bot_name"]: Rating(mu=opponent["rating_mu"], sigma=opponent["rating_sigma"])
        for opponent in latest_rankings.to_dict(orient="records")
    }
    opponents_ratings[own_bot] = ts_env.create_rating()
    rankings = process_match_results(matches, ts_env, opponents_ratings, own_bot)
    matches_df = pd.DataFrame(rankings)
    matches_df.sort_values(by='timestamp', inplace=True)
    matches_df.to_parquet("matches.parquet", index=False)
    print("==================== Match results ==================== ")
    print(
        matches_df[matches_df["bot_name"] == own_bot] # pyright: ignore[reportCallIssue]
        .sort_values(by='timestamp').to_string(index=False),
    ) 

    latest_ratings = matches_df.sort_values(by='timestamp').groupby('bot_name').last().reset_index()
    latest_ratings = latest_ratings.sort_values(by='rating_mu', ascending=False)
    latest_ratings.to_csv("tournament_ratings.csv", index=False)
    print("==================== Latest ratings ==================== ")
    print(latest_ratings[['bot_name', 'rating_mu']].to_string(index=False))




if __name__ == "__main__": main()
