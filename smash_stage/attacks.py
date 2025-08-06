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
        self.explosion_knock_back = attack_pattern[0].explosion_knock_back
        self.knockback_strength = attack_pattern[0].knockback_strength
        self.textures = [pattern.texture for pattern in attack_pattern]
        self.texture = attack_pattern[0].texture
        self.current_frame = 0
        self.animation_timer = 0
        self.frame_duration = frame_duration
        self.damage = attack_pattern[0].damage
        self.knockback_direction = attack_pattern[0].knockback_direction
    # TODO: nog ff de lijst gebruiken ipv alles op 1 value
    def new_attack(self):
        self.current_frame = 0
        self.animation_timer = 0
        self.texture = self.attack_pattern[0].texture

    def update(self, delta_time):
        self.animation_timer += delta_time
        height_last_frame = self.texture.height
        if self.current_frame == len(self.textures) - 1:
            self.remove_from_sprite_lists()
        if self.animation_timer >= self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.textures)
            self.texture = self.textures[self.current_frame]
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
        for i in range(1, 11)
    ],
    frame_duration=1 / 5,
)

HADUKAN_BLITZ = Attack(
    attack_pattern=[
        AttackPattern(
            damage=6,
            knockback_direction=Position(x_direction=1,y_direction=1),
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
