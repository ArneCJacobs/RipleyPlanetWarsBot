import subprocess

REGISTRY = "pwregistry.zeus.gent"
# REGISTRY = "http://127.0.0.1:9001"
def main():
    res = subprocess.run(
        [
            "docker",
            "build",
            "-t",
            f"ripleybot:latest",
            "--platform=linux/amd64",
            ".",
        ],
    )
    res = subprocess.run(
        [
            "docker",
            "tag",
            f"ripleybot:latest",
            REGISTRY + "/ripleybot:latest",
        ],
    )
    if res.returncode != 0:
        print("Failed to tag the image.")
        return

    # res = subprocess.run(
    #     ["docker", "push", f"{REGISTRY}/ripleybot:{tag}"],
    # )

    res = subprocess.run(
        [
            "docker",
            "push",
            f"{REGISTRY}/ripleybot:latest",
        ],
    )

if __name__ == "__main__":
    main()
