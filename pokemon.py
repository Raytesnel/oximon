import os
from enum import Enum
from pathlib import Path
from typing import Literal

from pydantic import BaseModel

ASSETS_PATH = Path(os.path.dirname(__file__)) / "assets"


class PokemonSprite(BaseModel):
    left: list[Path]
    right: list[Path]
    up: list[Path]
    down: list[Path]


class Pokemon(BaseModel):
    name: str
    level: int
    sprite: PokemonSprite
    areas: list[int]


class Pokemons(Enum):
    ONE = Pokemon(
        name="Bulbasaur",
        level=1,
        sprite=PokemonSprite(
            left=[
                ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / f"overworld_{i}"
                for i in range(6, 9)
            ],
            right=[
                ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / f"overworld_{i}"
                for i in range(9, 12)
            ],
            up=[
                ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / f"overworld_{i}"
                for i in range(3, 6)
            ],
            down=[
                ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / f"overworld_{i}"
                for i in range(0, 3)
            ],
        ),
        areas=[1, 2],
    )
    TWO = Pokemon(
        name="Charmander",
        level=1,
        sprite=PokemonSprite(
            left=[
                ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / f"overworld_{i}"
                for i in range(6, 9)
            ],
            right=[
                ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / f"overworld_{i}"
                for i in range(9, 12)
            ],
            up=[
                ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / f"overworld_{i}"
                for i in range(3, 6)
            ],
            down=[
                ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / f"overworld_{i}"
                for i in range(0, 3)
            ],
        ),
        areas=[2],
    )
