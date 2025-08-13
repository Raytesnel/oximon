from enum import Enum
from pathlib import Path
from loguru import logger
import arcade
from pydantic import BaseModel

from utils import ASSETS_PATH


class Position(BaseModel):
    x_direction: float
    y_direction: float


class AttackPattern(BaseModel, arbitrary_types_allowed=True):
    texture: arcade.Texture
    damage: float
    knockback_direction: Position
    knockback_strength: int
    explosion_knock_back: int


class Attack(arcade.Sprite):
    def __init__(
        self,
        attack_pattern: list[AttackPattern],
        frame_duration=1 / 30,
    ):
        super().__init__()
        self.attack_pattern = attack_pattern
        self.attack_flow = None
        self.explosion_knock_back = 0
        self.knockback_strength = 0
        self.textures = [pattern.texture for pattern in attack_pattern]
        self.texture = attack_pattern[0].texture
        self.current_frame = 0
        self.animation_timer = 0
        self.frame_duration = frame_duration
        self.damage = 0
        self.knockback_direction = Position(x_direction=0, y_direction=0)

    def new_attack(self):
        self.attack_flow = iter(self.attack_pattern)
        self.current_frame = 0
        self.explosion_knock_back = 0
        self.damage = 0
        self.knockback_direction = Position(x_direction=0, y_direction=0)
        self.knockback_strength = 0
        self.animation_timer = 0
        self.texture = self.attack_pattern[0].texture

    def update(self, delta_time):
        self.animation_timer += delta_time
        height_last_frame = self.texture.height
        if self.current_frame == len(self.textures) - 1:
            self.remove_from_sprite_lists()
        if self.animation_timer >= self.frame_duration:
            try:
                new_attack: AttackPattern = next(self.attack_flow)
            except StopIteration:
                logger.debug("eind of animation")
                self.remove_from_sprite_lists()
                return
            self.texture = new_attack.texture
            self.explosion_knock_back = new_attack.explosion_knock_back
            self.knockback_strength = new_attack.knockback_strength
            self.damage = new_attack.damage
            self.knockback_direction = new_attack.knockback_direction
            self.animation_timer = 0
            self.height = self.texture.height
            self.center_y = self.center_y - (height_last_frame - self.height) // 2


def collect_attack_sprites(attack_folder: Path) -> list[arcade.Texture]:
    return [
        arcade.load_texture(attack_folder / f"hadukan_{i}.png") for i in range(1, 11)
    ]


HADUKAN = Attack(
    attack_pattern=[
        AttackPattern(
            explosion_knock_back=-1,
            damage=1.5,
            knockback_direction=Position(x_direction=0, y_direction=1),
            knockback_strength=1,
            texture=arcade.load_texture(
                Path(ASSETS_PATH)
                / "sprites/pokemon/Charmander/hadukan_2"
                / f"hadukan_{i}.png"
            ),
        )
        for i in range(1, 8)
    ]
    + [
        AttackPattern(
            explosion_knock_back=5,
            damage=1.5,
            knockback_direction=Position(x_direction=1, y_direction=1),
            knockback_strength=1,
            texture=arcade.load_texture(
                Path(ASSETS_PATH)
                / "sprites/pokemon/Charmander/hadukan"
                / f"hadukan_{i}.png"
            ),
        )
        for i in range(4, 8)
    ],
    frame_duration=1 / 5,
)

HADUKAN_BLITZ = Attack(
    attack_pattern=[
        AttackPattern(
            damage=6,
            knockback_direction=Position(x_direction=1, y_direction=1),
            knockback_strength=8,
            explosion_knock_back=2,
            texture=arcade.load_texture(
                Path(ASSETS_PATH)
                / "sprites/pokemon/Charmander/hadukan"
                / f"hadukan_{i}.png",
            ),
        )
        for i in range(1, 11)
    ],
    frame_duration=1 / 20,
)
