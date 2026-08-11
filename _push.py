import subprocess
from pathlib import Path

dej = Path(r"E:/Spark 游戏引擎/dejavu-engine")


def git(*args, check=True):
    r = subprocess.run(
        ["git", "-c", f"safe.directory={dej}", *args],
        cwd=dej,
        capture_output=True,
        text=True,
    )
    print(">", " ".join(args[:8]), r.returncode)
    if r.stdout.strip():
        print(r.stdout[-1000:])
    if r.stderr.strip():
        print(r.stderr[-1000:])
    if check and r.returncode:
        raise SystemExit(r.returncode)
    return r


for p in ["_fail.py", "_get_log.py", "_job.log"]:
    fp = dej / p
    if fp.exists():
        fp.unlink()

git("add", "-A")
# unstage dist if tracked
git("reset", "HEAD", "--", "projects/dejavu.ts/dejavu/dist", check=False)
git(
    "status",
    "-sb",
    check=False,
)
git(
    "commit",
    "-m",
    "Fix dejavu release-npm prepare: use tsup publish config + tsconfig paths.\n\nAvoid broken --no-external CLI; pin facade build:publish for OIDC smoke at 0.0.0.",
)
git("push", "doki-land", "HEAD:dev")
git("tag", "-f", "v0.0.0")
git("push", "doki-land", "refs/tags/v0.0.0", "--force")
print("pushed")
