import os
from pathlib import Path

from pydantic import BaseModel

from lifeforms.characters import FighterSprites
from lifeforms.pokemons import WildPokemon, OverWorldMonsterSprites
from smash_stage.fighter import Character
from utils import ASSETS_PATH

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
        return WildPokemon(name=self.name, image_path=self.over_world_sprites)

    @property
    def fighter(self) -> Character:
        return Character(
            name=self.name, asset_path=ASSETS_PATH / "sprites/pokemon/Lightning Mage"
        )

    @property
    def fields(self) -> list[int]:
        return [field.field for field in self.encounter_fields]


Monsters = [
    Monster(
        name="Bulbasaur",
        fighter_sprites=FighterSprites(
            light_attack_melee=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "Attack_2.png",
            light_attack_range=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "light_ball.png",
            heavy_attack_range=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "charge_attack.png",
            heavy_attack_melee=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "Attack_1.png",
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Dead.png",
            hurt=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Hurt.png",
            idle=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Idle.png",
            jump=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Jump.png",
            run=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Run.png",
            walk=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Walk.png",
        ),
        over_world_sprites=OverWorldMonsterSprites(
            left=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "left.png",
            right=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "right.png",
            up=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "up.png",
            down=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "down.png",
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "left.png",
            banner=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "banner.png",
        ),
        encounter_fields=[
            EncounterMonster(field=1, chance=0.8),
            EncounterMonster(field=2, chance=0.3),
        ],
        speed=60,
        health=50,
        defense=20,
        attack=80,
    ),
    Monster(
        name="Charmander",
        fighter_sprites=FighterSprites(
            light_attack_melee=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "Attack_2.png",
            light_attack_range=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "light_ball.png",
            heavy_attack_range=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "charge_attack.png",
            heavy_attack_melee=ASSETS_PATH
            / "sprites"
            / "pokemon"
            / "Lightning Mage"
            / "Attack_1.png",
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Dead.png",
            hurt=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Hurt.png",
            idle=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Idle.png",
            jump=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Jump.png",
            run=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Run.png",
            walk=ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage" / "Walk.png",
        ),
        over_world_sprites=OverWorldMonsterSprites(
            left=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "left.png",
            right=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "right.png",
            up=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "up.png",
            down=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "down.png",
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "left.png",
            banner=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "banner.png",
        ),
        encounter_fields=[
            EncounterMonster(field=1, chance=0.8),
            EncounterMonster(field=2, chance=0.3),
        ],
        speed=60,
        health=50,
        defense=20,
        attack=80,
    ),
]
