import os
from pathlib import Path
from typing import Literal

from pydantic import BaseModel

from lifeforms.characters import FighterSprites
from lifeforms.pokemons import WildPokemon, OverWorldMonsterSprites
from smash_stage.attacks import Attack, BlueFireBreath, SimpleMelee, FireBreath
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


class QuestLineMove(BaseModel):
    quest: str
    finised: bool
    objective_count: int
    achieved_count: int


class Moves(BaseModel, arbitrary_types_allowed=True):
    name: str
    attack_mode: Literal["heavy", "light"]
    range: bool
    move: type[Attack]
    quest_line: QuestLineMove


class DescriptionMonster(BaseModel):
    description: str
    type: Literal["Fire", "Water", "Void", "Life", "Air", "Stone", "Tech"]


class Monster(BaseModel):
    name: str
    fighter_sprites: FighterSprites
    over_world_sprites: OverWorldMonsterSprites
    encounter_fields: list[EncounterMonster]
    description: DescriptionMonster
    speed: int
    health: int
    defense: int
    attack: int
    moves: list[Moves]

    @property
    def over_world(self) -> WildPokemon:
        return WildPokemon(name=self.name, image_path=self.over_world_sprites)

    @property
    def fighter(self) -> Character:
        return Character(
            name=self.name,
            asset_path=ASSETS_PATH
            / "sprites/pokemon/Lightning Mage",  # TODO: make path to FighterSprites
        )

    @property
    def fields(self) -> list[int]:
        return [field.field for field in self.encounter_fields]


Monsters = [
    Monster(
        name="Bulbasaur",
        description=DescriptionMonster(
            description="een monster met een groene puist", type="Life"
        ),
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
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Bulbasaur" / "dead.png",
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
        moves=[
            Moves(
                name="electro ball",
                attack_mode="heavy",
                range=True,
                move=BlueFireBreath,
                quest_line=QuestLineMove(
                    quest="kill 20 characters with fire",
                    finised=True,
                    objective_count=20,
                    achieved_count=20,
                ),
            ),
            Moves(
                name="tacle",
                attack_mode="light",
                range=False,
                move=SimpleMelee,
                quest_line=QuestLineMove(
                    quest="run 100 meter",
                    finised=True,
                    objective_count=100,
                    achieved_count=100,
                ),
            ),
        ],
    ),
    Monster(
        name="Charmander",
        description=DescriptionMonster(
            description="fire dino, bruhahaha",
            type="Fire",
        ),
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
            dead=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "dead.png",
            banner=ASSETS_PATH / "sprites" / "pokemon" / "Charmander" / "banner.png",
        ),
        encounter_fields=[
            EncounterMonster(field=2, chance=0.8),
            EncounterMonster(field=3, chance=0.3),
        ],
        speed=60,
        health=50,
        defense=20,
        attack=80,
        moves=[
            Moves(
                name="fire blast",
                attack_mode="heavy",
                range=True,
                move=FireBreath,
                quest_line=QuestLineMove(
                    quest="Survive 10 fire attacks",
                    finised=False,
                    objective_count=10,
                    achieved_count=4,
                ),
            ),
            Moves(
                name="blue fire",
                attack_mode="heavy",
                range=True,
                move=FireBreath,
                quest_line=QuestLineMove(
                    quest="Survive 10 fire attacks",
                    finised=False,
                    objective_count=10,
                    achieved_count=4,
                ),
            ),
            Moves(
                name="tacle",
                attack_mode="light",
                range=False,
                move=SimpleMelee,
                quest_line=QuestLineMove(
                    quest="run 100 meter",
                    finised=True,
                    objective_count=100,
                    achieved_count=100,
                ),
            ),
        ],
    ),
]

"""
Lucy lijstje:
   - achtergrond muziek
   - gevecht muziek
   - meer monsters rapidash
   - grot
   - monster in gevecht
   - poef of downloading animation when pokemon defeated
   - pokemon achter je lopen.
"""
