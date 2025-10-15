from pydantic import BaseModel

from lifeforms.pokemons import WildPokemon
from schemas.schemas import (
    FighterSprites,
    OverWorldMonsterSprites,
    EncounterMonster,
    DescriptionMonster,
    Moves,
)
from smash_stage.fighter import Character


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
        return Character(name=self.name, asset_path=self.fighter_sprites)

    @property
    def fields(self) -> list[int]:
        return [field.field for field in self.encounter_fields]
