import os
from pathlib import Path
from typing import Optional

import arcade
from loguru import logger
from pydantic import BaseModel

from smash_stage.attacks import (
    Attack,
    HADUKAN,
    HADUKAN_BLITZ,
    FireBreath,
    BlueFireBreath,
)
from utils import ASSETS_PATH


class KnockBackDamage(BaseModel):
    x_position: float
    y_position: float
    knockback: float
    damage: int


class Character(arcade.Sprite):
    def __init__(self, asset_path: Path, name: str):
        super().__init__(scale=1)
        self.animations = {
            "walk": arcade.load_spritesheet(asset_path / "Walk.png").get_texture_grid(
                size=(128, 128), columns=7, count=7
            ),
            "run": arcade.load_spritesheet(asset_path / "Run.png").get_texture_grid(
                size=(128, 128), columns=8, count=8
            ),
            "jump": arcade.load_spritesheet(asset_path / "Jump.png").get_texture_grid(
                size=(128, 128), columns=8, count=8
            ),
            "idle": arcade.load_spritesheet(asset_path / "Idle.png").get_texture_grid(
                size=(128, 128), columns=7, count=7
            ),
            "hurt": arcade.load_spritesheet(asset_path / "Hurt.png").get_texture_grid(
                size=(128, 128), columns=3, count=3
            ),
            "dead": arcade.load_spritesheet(asset_path / "Dead.png").get_texture_grid(
                size=(128, 128), columns=5, count=5
            ),
            "attack_1": arcade.load_spritesheet(
                asset_path / "Attack_1.png"
            ).get_texture_grid(size=(128, 128), columns=10, count=10),
            "attack_2": arcade.load_spritesheet(
                asset_path / "Attack_2.png"
            ).get_texture_grid(size=(128, 128), columns=4, count=4),
            "attack_heavy_1": arcade.load_spritesheet(
                asset_path / "Light_ball.png"
            ).get_texture_grid(size=(128, 128), columns=7, count=7),
            # "attack_heavy_2": arcade.load_spritesheet(asset_path/ "charge_attack.png").get_texture_grid(size=(128, 128), columns=13, count=13),
        }
        self.direction = "down"
        self.stun_duration = 1/10
        self.stun_counter = 0.0
        self.is_stunned = False
        self.pending_knockback: Optional[KnockBackDamage] = None
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 1 / 10
        self.texture = self.animations["idle"][0]
        self.jump_count = 0
        self.max_jumps = 2
        self.lives = 100
        self.MOVE_SPEED = 2
        self.JUMP_SPEED = 20
        self.name = name
        self.hit_timer = 1 / 10
        self.hit_counter = 0
        self.invincible_frame = 2 / 60
        self.hits_take: Optional[KnockBackDamage] = None
        self.held_keys = set()
        self.animation_state = "idle"
        self.attack = BlueFireBreath()
        self.attack_2 = FireBreath()

        # Attack animation
        hadouken_path = Path(ASSETS_PATH) / "sprites/pokemon/Charmander/hadukan"
        self.hadouken_frames = [
            arcade.load_texture(hadouken_path / f"hadukan_{i}.png")
            for i in range(1, 11)
        ]

    def take_hit(self, knockback_data: KnockBackDamage):
        """Trigger stun + knockback sequence."""
        self.animation_state = "hurt"
        self.frame_duration = 1/10
        self.current_frame = 0
        self.pending_knockback = knockback_data
        self.stun_counter = 0
        if not self.is_stunned:
            self.lives = max(0, self.lives - self.pending_knockback.damage)

        self.is_stunned = True

    def update_animation(self, delta_time: float = 1 / 60):
        self.frame_timer += delta_time
        if self.frame_timer > self.frame_duration:
            if (
                self.animation_state
                in ["hurt","attack_1", "attack_heavy_1", "attack_2", "attack_heavy_2"]
                and self.current_frame == len(self.animations[self.animation_state]) -1
            ):
                self.animation_state = "idle"
                self.frame_duration = 1 / 10
            self.current_frame = (self.current_frame + 1) % len(
                self.animations[self.animation_state]
            )
            self.texture = self.animations[self.animation_state][self.current_frame]
            self.frame_timer = 0

    def update(self, delta_time: float = 1 / 60):
        if self.is_stunned:
            self.stun_counter += delta_time
            if self.stun_counter >= self.stun_duration:
                self.is_stunned = False
                if self.pending_knockback:
                    self.change_x = (
                        self.pending_knockback.x_position
                        * self.pending_knockback.knockback
                    )
                    self.change_y = (
                        self.pending_knockback.y_position
                        * self.pending_knockback.knockback
                    )
                    self.pending_knockback = None
            else:
                self.change_y = 0
                self.change_x = 0
            return  # No movement/gravity during stun
        if abs(self.change_x) > 0.1:
            self.change_x *= 0.85
        else:
            self.change_x = 0

    def perform_attack(self, name, direction="neutral"):
        logger.debug("pew pew")
        offset_hand_x = 30
        offset_hand_y = -15
        attack = self.attack

        # if direction == "neutral":
        #     attack = self.attack_2
        if direction == "special_left":
            attack.facing = -1
        else:
            attack.facing = 1
        attack.new_attack()
        attack.owner = self
        attack.center_y = self.center_y + offset_hand_y
        attack.center_x = self.center_x + offset_hand_x
        return attack


# TODO: add recovery time
# TODO: while in recovery state,Character(PLAYER_PATH, "monster") movement in reduced ( terug lopen terwilj je weg wordt geschoten)
# TODO: build up force, combo's builds up a dicrectional force. before unleashing it.
# TODO: attack hit flashy things.
