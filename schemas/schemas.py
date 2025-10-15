from pathlib import Path
from typing import Literal

from pydantic import BaseModel

from smash_stage.attacks import Attack


class KnockBackDamage(BaseModel):
    x_position: float
    y_position: float
    knockback: float
    damage: int


class CharacterAttack(BaseModel):
    attack: type[Attack]
    position: tuple[int, int]
    end_start_up_frame: int
    end_attack_frame: int
    animation: str


class AttackClass(BaseModel):
    up: type[Attack] | None = None
    down: type[Attack] | None = None
    side: type[Attack] | None = None
    neutral: type[Attack] | None = None


class Attacks(BaseModel):
    neutral: AttackClass
    special: AttackClass


class SpriteSheet(BaseModel):
    spritesheet_path: Path
    texture_columns: int
    size: int


class FighterSprites(BaseModel):
    walk: SpriteSheet
    run: SpriteSheet
    jump: SpriteSheet
    idle: SpriteSheet
    hurt: SpriteSheet
    dead: SpriteSheet
    attack_melee: SpriteSheet
    attack_heavy_melee: SpriteSheet
    attack_range: SpriteSheet
    attack_range_heavy: SpriteSheet


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


class OverWorldMonsterSprites(BaseModel):
    up: Path
    down: Path
    left: Path
    right: Path
    dead: Path
    banner: Path
