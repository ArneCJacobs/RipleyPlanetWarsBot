import subprocess

REGISTRY = "pwregistry.zeus.gent"
def main():
    res = subprocess.run(
        ["docker", "build", "-t", "ripleybot", "."],
    )
    res = subprocess.run(
        ["docker", "tag", "ripleybot", REGISTRY + "/ripleybot"],
    )
    if res.returncode != 0:
        print("Failed to tag the image.")
        return

    res = subprocess.run(
        ["docker", "push", REGISTRY + "/ripleybot:latest"],
    )



if __name__ == "__main__":
    main()
