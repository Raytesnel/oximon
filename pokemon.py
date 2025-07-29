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
    areas: list[str]


poke_data: dict[int, Pokemon] = {
    1: Pokemon(
        name="Pikachu",
        level=5,
        sprite=PokemonSprite(
            left=[Path("sprites/pikachu/left1.png"), Path("sprites/pikachu/left2.png")],
            right=[
                Path("sprites/pikachu/right1.png"),
                Path("sprites/pikachu/right2.png"),
            ],
            up=[Path("sprites/pikachu/up1.png"), Path("sprites/pikachu/up2.png")],
            down=[Path("sprites/pikachu/down1.png"), Path("sprites/pikachu/down2.png")],
        ),
        areas=["forest", "plains"],
    ),
    2: Pokemon(
        name="Charmander",
        level=5,
        sprite=PokemonSprite(
            left=[
                Path("sprites/charmander/left1.png"),
                Path("sprites/charmander/left2.png"),
            ],
            right=[
                Path("sprites/charmander/right1.png"),
                Path("sprites/charmander/right2.png"),
            ],
            up=[Path("sprites/charmander/up1.png"), Path("sprites/charmander/up2.png")],
            down=[
                Path("sprites/charmander/down1.png"),
                Path("sprites/charmander/down2.png"),
            ],
        ),
        areas=["mountain", "cave"],
    ),
}

Pokemons = {
    1: Pokemon(
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
        areas=["1", "2"],
    ),
    2: Pokemon(
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
        areas=["2", "3"],
    ),
}
