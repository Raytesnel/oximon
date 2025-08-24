import os
from pathlib import Path
import yaml

ASSETS_PATH = Path(os.path.dirname(__file__)) / "assets"
POKEMON_SPRITES_PATH = ASSETS_PATH / "sprites" / "pokemon"
PLAYER_PATH = ASSETS_PATH / "sprites" / "player"
SMASH_MAP_PATH = ASSETS_PATH / "map" / "smash.tmx"
SCREEN_WIDTH = 1280/1.5
SCREEN_HEIGHT = 960/1.5
SCREEN_TITLE = "PokeSmash"

ROOT_DIR = Path(__file__).resolve().parent

STATE_FILE = PLAYER_PATH / "state.yaml"

def load_state():
    if not STATE_FILE.exists():
        return {"player": {}, "pokemons": {}, "quests": {}}
    with open(STATE_FILE, "r", encoding="utf-8") as f:
        return yaml.safe_load(f)

def save_state(state: dict):
    with open(STATE_FILE, "w", encoding="utf-8") as f:
        yaml.safe_dump(state, f)
