import pandas as pd

def main():
    latest_rankings = pd.read_csv("latest_ratings.csv")
    opponents = latest_rankings["bot_name"].tolist()
    maps = ['hex', 'spiral', 'hunger_games']



if __name__ == "__main__":
    main()
