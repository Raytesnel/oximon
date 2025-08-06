from pathlib import Path
from typing import Literal

import arcade

from utils import ASSETS_PATH


class Attack(arcade.Sprite):
    def __init__(
        self,
        textures: list[arcade.Texture],
        damage: float,
        knockback_direction: tuple[float, float],
        knockback_strength: float,
        explosion_knock_back: float,
        frame_duration=1 / 30,
        lifetime=10,
    ):
        super().__init__()
        self.explosion_knock_back = explosion_knock_back
        self.knockback_strength = knockback_strength
        self.lifetime = lifetime
        self.textures = textures
        self.texture = textures[0]
        self.current_frame = 0
        self.animation_timer = 0
        self.frame_duration = frame_duration
        self.life_timer = lifetime * frame_duration
        self.damage = damage
        self.knockback_direction = knockback_direction

    def new_attack(self):
        self.current_frame = 0
        self.animation_timer = 0
        self.texture = self.textures[0]
        self.life_timer = self.lifetime * self.frame_duration

    def update(self, delta_time):
        self.life_timer -= delta_time
        if self.life_timer <= 0:
            self.remove_from_sprite_lists()
            return
        height_last_frame = self.texture.height

        self.animation_timer += delta_time
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


# TODO: calculate knockback from center of attack to character.
# TODO: add up_force per attack
# TODO: add parameter for each attack how long unvunable enemy is.
# TODO: add recovery time
# TODO: attack hit flashy things.

HADUKAN = Attack(
    textures= collect_attack_sprites(Path(ASSETS_PATH) / "sprites/pokemon/Charmander/hadukan_2"),
    damage=1.5,
    knockback_direction=(1, 0),
    knockback_strength=1,
    frame_duration=1 / 20,
    explosion_knock_back=-0.5,
)

HADUKAN_BLITZ = Attack(
    textures=collect_attack_sprites(
        Path(ASSETS_PATH) / "sprites/pokemon/Charmander/hadukan"
    ),
    damage=6,
    knockback_direction=(0.01, 1),
    knockback_strength=8,
    frame_duration=1 / 20,
    explosion_knock_back=2,
)
