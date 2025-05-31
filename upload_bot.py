import subprocess
import re

REGISTRY = "pwregistry.zeus.gent"
def main():
    with open("CHANGELOG.md", "r") as f:
        changelog = f.readlines()[-1].strip()

    tag = re.search(r"\[(.*)\]", changelog)
    if not tag:
        print("Version not found in CHANGELOG.md")
        return

    tag = tag.group(1).strip()
    
    res = subprocess.run(
        [
            "docker",
            "build",
            "-t",
            f"ripleybot:{tag}",
            ".",
        ],
    )
    res = subprocess.run(
        [
            "docker",
            "tag",
            f"ripleybot:{tag}",
            REGISTRY + "/ripleybot",
        ],
    )
    if res.returncode != 0:
        print("Failed to tag the image.")
        return

    # res = subprocess.run(
    #     ["docker", "push", f"{REGISTRY}/ripleybot:{tag}"],
    # )

    res = subprocess.run(
        ["docker", "push", f"{REGISTRY}/ripleybot:latest"],
    )



if __name__ == "__main__":
    main()
