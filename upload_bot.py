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

    if res.returncode != 0:
        exit(1)

    res = subprocess.run(
        [
            "docker",
            "tag",
            f"ripleybot:latest",
            REGISTRY + "/ripleybot:latest",
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
            f"{REGISTRY}/ripleybot:latest",
        ],
    )

    if res.returncode != 0:
        exit(1)

if __name__ == "__main__":
    main()
