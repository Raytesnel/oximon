import os
from pathlib import Path

ASSETS_PATH = Path(os.path.dirname(__file__)) / "assets"
POKEMON_SPRITES_PATH = ASSETS_PATH / "sprites" / "pokemon"
PLAYER_PATH = ASSETS_PATH / "sprites" / "player"
SMASH_MAP_PATH = ASSETS_PATH / "map" / "smash.tmx"
SCREEN_WIDTH = 1280
SCREEN_HEIGHT = 960
SCREEN_TITLE = "PokeSmash"

ROOT_DIR = Path(__file__).resolve().parent
