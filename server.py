import logging
import re
import subprocess

from fastapi import FastAPI
from fastapi.responses import HTMLResponse, RedirectResponse

app = FastAPI()
logger = logging.getLogger("api")
game_pattern = re.compile(r"^[ox\.;]*$")


class Validate:
    """Check GET queries"""

    @staticmethod
    def game(g: str):
        """Game Board"""
        assert game_pattern.match(g)

    @staticmethod
    def next_player(n: str):
        """Next Player is o or x"""
        assert n == "o" or n == "x"


class Shell:
    """Shell Script Runner"""

    @staticmethod
    def run(cmd: str) -> str:
        """Returns stdout"""
        logger.debug("Running: %s", cmd)
        stdout = subprocess.check_output(cmd, shell=True).decode().strip()
        logger.debug("stdout: %s", stdout)
        return stdout


@app.get("/", response_class=RedirectResponse)
async def index():
    """index page is /othello"""
    return RedirectResponse("/othello")


@app.get("/othello", response_class=HTMLResponse)
async def othello():
    """Othello Game Page"""
    return open("./server-files/index.html", "rt").read()


@app.get("/othello/solve/{game}")
async def solve(game: str, next_player: str, num_try: int = 200):
    """AI Solver"""
    Validate.game(game)
    Validate.next_player(next_player)

    result = Shell.run(
        f"echo '{game}' | tr ';' '\n'| "
        f"cargo run --release -- solve {next_player} --num-try {num_try}"
    )
    result = ";".join(result.split("\n"))
    return {"stdout": result}


@app.get("/othello/move/{game}")
async def move(game: str, next_player: str, x: int, y: int):
    """Game Simulation"""
    Validate.game(game)
    Validate.next_player(next_player)

    result = Shell.run(
        f"echo '{game}' | tr ';' '\n'| "
        f"cargo run --release -- move {next_player} {x} {y}"
    )
    result = ";".join(result.split("\n"))
    return {"stdout": result}
