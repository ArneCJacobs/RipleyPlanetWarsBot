def main():
    import os 
    import subprocess

    # get tag and message from command line 
    if len(os.sys.argv) < 3:
        print("Usage: python tag_new_version.py <tag> <message>")
        return 

    tag = os.sys.argv[1]
    message = os.sys.argv[2] 

    # check for uncommitted changes
    cmd = subprocess.run(
        ["git", "status", "--porcelain"],
        capture_output=True,
        text=True
    )
    if "Untracked files" in cmd.stdout.strip():
        print("Error: There are uncommitted changes in the repository.")
        return

    cmd = subprocess.run(
        ["git", "tag", "-a", tag, "-m", message],
        capture_output=True,
        text=True
    )
    if cmd.returncode != 0:
        print(f"Error tagging version: {cmd.stderr.strip()}")
        return 

    # append to CHANGELOG.md 
    with open("CHANGELOG.md", "a") as changelog:
        changelog.write(f"- [{tag}]:\t{message}\n")





