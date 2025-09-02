import os
from pathlib import Path

from pydantic import BaseModel

from lifeforms.pokemons import WildPokemon
from smash_stage.fighter import Character

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


# TODO make the fighter and overworld pokemon a sub class of a Monsterclass


class FighterSprites(BaseModel):
    light_attack_melee: Path
    light_attack_range: Path
    heavy_attack_range: Path
    heavy_attack_melee: Path
    dead: Path
    hurt: Path
    idle: Path
    jump: Path
    run: Path
    walk: Path


class OverWorldMonsterSprites(BaseModel):
    up: Path
    down: Path
    left: Path
    right: Path
    dead: Path


class EncounterMonster(BaseModel):
    field: int
    chance: float


class Monster(BaseModel):
    name: str
    fighter_sprites: FighterSprites
    over_world_sprites: OverWorldMonsterSprites
    encounter_fields: list[EncounterMonster]
    speed: int
    health: int
    defense: int
    attack: int

    @property
    def over_world(self) -> WildPokemon:
        return WildPokemon(
            name=self.name, maggots_bounds=None, image_path=self.over_world_sprites
        )

    @property
    def fighter(self) -> Character:
        return Character(
            name=self.name, asset_path=ASSETS_PATH / "sprites/pokemon/Lightning Mage"
        )


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
