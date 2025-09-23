import random
from pathlib import Path
from typing import Optional

import arcade
from pydantic import BaseModel

from smash_stage.attacks import (
    FireBreath,
    Attack,
)


class KnockBackDamage(BaseModel):
    x_position: float
    y_position: float
    knockback: float
    damage: int


class Attacks(BaseModel):
    up: type[Attack] | None
    down: type[Attack] | None
    left: type[Attack] | None
    right: type[Attack] | None
    base: type[Attack]


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
        self.stun_duration = 1 / 5
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
        self.reverse = False
        self.attacks = None

    def set_attacks(self, attacks: Attacks):
        self.attacks = attacks

    def take_hit(self, knockback_data: KnockBackDamage):
        """Trigger stun + knockback sequence."""
        self.animation_state = "hurt"
        self.frame_duration = 1 / 10
        self.current_frame = 0
        self.pending_knockback = knockback_data
        self.stun_counter = 0
        if not self.is_stunned:
            self.lives = max(0, self.lives - self.pending_knockback.damage)

        self.is_stunned = True

    def update_animation(self, delta_time: float = 1 / 60):
        self.frame_timer += delta_time
        if self.is_stunned:
            self.animation_state = "hurt"
        elif self.frame_timer > self.frame_duration:
            if (
                self.animation_state
                in [
                    "jump",
                    "hurt",
                    "attack_1",
                    "attack_heavy_1",
                    "attack_2",
                    "attack_heavy_2",
                ]
                and self.current_frame == len(self.animations[self.animation_state]) - 1
            ):
                self.animation_state = "idle"
                self.frame_duration = 1 / 10
            self.current_frame = (self.current_frame + 1) % len(
                self.animations[self.animation_state]
            )
            self.texture = self.animations[self.animation_state][self.current_frame]
            if self.reverse:
                self.texture = self.texture.flip_horizontally()
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

    def perform_attack(self, direction="neutral"):
        if not self.animation_state in ["run", "walk", "jump", "idle"]:
            raise ValueError("already a attack animation is working.")
        attack = None
        match direction:
            case "special_right":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.right:
                    attack = self.attacks.right()
                    attack.direction = (1, 0)
            case "special_left":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.left:
                    attack = self.attacks.left()
                    attack.angle = 180
                    attack.direction = (-1, 0)

            case "special_neutral":
                offset_hand_x = 40
                offset_hand_y = -15
                attack = FireBreath()
            case "special_up":
                offset_hand_x = 0
                offset_hand_y = +15
                if self.attacks.up:
                    attack = self.attacks.up()
                    attack.angle = 90
                    attack.direction = (0, 1)

            case "special_down":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.down:
                    attack = self.attacks.down()
                    attack.direction = (1, 0)

            case "normal_neutral":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.base:
                    attack = self.attacks.base()
                    attack.direction = (1, 0)
            case _:
                raise ValueError("unknown button for attack.")
        if not attack:
            raise ValueError("no attack is set on that button")
        attack.new_attack()
        attack.owner = self
        attack.center_y = self.center_y + offset_hand_y
        attack.center_x = self.center_x + offset_hand_x
        return attack


# TODO: while in recovery state,Character(PLAYER_PATH, "monster") movement in reduced ( terug lopen terwilj je weg wordt geschoten)
# TODO: attack hit flashy things.
# TODO: add velocity instead of channge, zo when hit it slowly stops and then you can return.


class EnemyAI:
    def __init__(self, character: Character, target: Character, stage):
        self.character = character
        self.target = target
        self.stage = stage
        self.attack_cooldown = 1.0  # seconds between attacks
        self.attack_timer = 0.0

    def update(self, delta_time: float):
        """Move toward target and attack when close."""
        self.attack_timer -= delta_time
        if self.attack_timer < 0:
            self.attack_timer = 0

        target_position = self.target.center_x
        if not self.character.is_stunned:
            # Movement toward target
            if self.character.center_x < target_position - 10:
                self.character.change_x = self.character.MOVE_SPEED
                self.character.animation_state = "walk"
                self.character.reverse = False
            elif self.character.center_x > target_position + 10:
                self.character.change_x = -self.character.MOVE_SPEED
                self.character.animation_state = "walk"
                self.character.reverse = True
            else:
                self.character.change_x = 0
                self.character.animation_state = "idle"

            # Attack if close enough
            distance_x = abs(self.character.center_x - self.target.center_x)
            distance_y = abs(self.character.center_y - self.target.center_y)

            if distance_x < 100 and distance_y < 60 and self.attack_timer == 0:
                try:
                    attack_names = [
                        f"special_{direction}"
                        for direction, attack in self.character.attacks
                        if attack is not None
                    ]
                    attack_names.append("normal_neutral")
                    attack = self.character.perform_attack(random.choice(attack_names))
                    self.stage.attack_hitboxes.append(attack)
                    self.character.animation_state = "attack_1"
                    self.attack_timer = self.attack_cooldown
                except ValueError:
                    pass  # no attack available
