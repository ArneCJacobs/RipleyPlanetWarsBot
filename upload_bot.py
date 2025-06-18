import subprocess
import sys

REGISTRY = "pwregistry.zeus.gent"
# REGISTRY = "http://127.0.0.1:9001"
def main():
    # get botname from command line argument 
    if len(sys.argv) < 2:
        print("Usage: python upload_bot.py <botname>")
        exit(1)

    botname = sys.argv[1]

    res = subprocess.run(
        [
            "docker",
            "build",
            "-t",
            f"{botname}:latest",
            "--platform=linux/amd64",
            ".",
        ],
    )

    if res.returncode != 0:
        exit(1)

    res = subprocess.run(
        [
            "docker",
            "tag",
            f"{botname}:latest",
            REGISTRY + f"/{botname}:latest",
        ],
    )
    if res.returncode != 0:
        exit(1)

    # res = subprocess.run(
    #     ["docker", "push", f"{REGISTRY}/ripleybot:{tag}"],
    # )

    res = subprocess.run(
        [
            "docker",
            "push",
            f"{REGISTRY}/{botname}:latest",
        ],
    )

    if res.returncode != 0:
        exit(1)

if __name__ == "__main__":
    main()
