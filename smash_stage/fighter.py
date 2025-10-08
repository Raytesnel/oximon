import random
from pathlib import Path
from typing import Optional

import arcade
from arcade import PhysicsEnginePlatformer
from loguru import logger
from pydantic import BaseModel

from smash_stage.attacks import (
    Attack,
)


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
    up: CharacterAttack | None = None
    down: CharacterAttack | None = None
    side: CharacterAttack | None = None
    neutral: CharacterAttack | None = None


class Attacks(BaseModel):
    neutral: AttackClass
    special: AttackClass


class NoAttackSet(Exception):
    pass


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
        self.animations["jump_start"] = self.animations["jump"][:3]
        self.animations["jump_up"] = [
            self.animations["jump"][3],
            self.animations["jump"][3],
        ]
        self.animations["jump_down"] = [
            self.animations["jump"][5],
            self.animations["jump"][5],
        ]
        self.stage = None
        self.stun_sec = 0
        self.animations["jump_landing"] = self.animations["jump"][5:8]
        self.direction = "down"
        self.stun_duration = 1 / 5
        self.pending_knockback: Optional[KnockBackDamage] = None
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 1 / 10
        self.texture = self.animations["idle"][0]
        # --- jumping power
        self.max_jumps = 2
        self.jump_count = 0
        self.is_jumping = False
        self.jump_held = False
        self.jump_timer = 0.0
        self.max_jump_time = 0.5
        self.jump_power = 4
        self.jump_volocity = 0.85
        self.base_hold_jump_power = 3
        self.hold_jump_power = 4
        self.physics_engine: PhysicsEnginePlatformer | None = None
        # --- jumping power
        # -- movement
        self.run_volocity = 1.1
        self.max_run_speed = 8
        self.walk_speed = 0.5
        # -- movement
        self.is_attacking = False
        self.lives = 100
        self.name = name
        self.hit_timer = 1 / 10
        self.hit_counter = 0
        self.invincible_frame = 2 / 60
        self.hits_take: Optional[KnockBackDamage] = None
        self.held_keys = set()
        self.animation_state = "idle"
        self.reverse = False
        self.attacks: Attacks | None = None

    def set_attacks(self, attacks: Attacks):
        self.attacks = attacks

    def update_animation(self, delta_time: float = 1 / 60):
        self.frame_timer += delta_time
        if self.stun_sec:
            self.animation_state = "hurt"
        if self.frame_timer > self.frame_duration:
            if (
                self.animation_state is not "idle"
                and self.current_frame == len(self.animations[self.animation_state]) - 1
            ):
                if not self.is_jumping and not self.stun_sec:
                    self.animation_state = "idle"
                    self.frame_duration = 1 / 10
                self.is_attacking = False
            self.current_frame = (self.current_frame + 1) % len(
                self.animations[self.animation_state]
            )
            self.texture = self.animations[self.animation_state][self.current_frame]
            if self.reverse:
                self.texture = self.texture.flip_horizontally()
            self.frame_timer = 0

    def update(self, delta_time: float = 1 / 60):
        self.update_jump(delta_time)
        self.handle_landing()
        if self.stun_sec:
            self.stun_sec -= delta_time
            if self.stun_sec < 0:
                self.stun_sec = 0
            self.change_y = 1
            self.change_x = 0
            return
        if self.pending_knockback:
            self.change_x = (
                self.pending_knockback.x_position * self.pending_knockback.knockback
            )
            self.change_y = (
                self.pending_knockback.y_position * self.pending_knockback.knockback
            )
            self.pending_knockback = None
        if abs(self.change_x) > 0.1:
            self.change_x *= 0.85
        else:
            self.change_x = 0

    def idle(self):
        if not self.is_attacking and not self.is_jumping:
            self.animation_state = "idle"
            self.change_x = 0

    def walk(self, direction):
        if self.is_attacking or self.stun_sec:
            return
        if not self.is_jumping:
            self.animation_state = "walk"
        if direction == "left":
            self.reverse = True
            self.change_x -= self.walk_speed
        elif direction == "right":
            self.reverse = False
            self.change_x += self.walk_speed

    def run(self, direction):
        if self.is_attacking or self.stun_sec:
            return
        if not self.is_jumping:
            self.animation_state = "run"
        if direction == "left":
            self.reverse = True if direction == "left" else False
        if self.change_x >= self.max_run_speed:
            self.change_x = self.max_run_speed
        else:
            self.change_x *= self.run_volocity

    def jump(self):
        """Triggered on key press. Hollow Knight-style jump."""
        if self.is_attacking:
            return
        if self.jump_count >= self.max_jumps:
            return
        self.jump_count += 1
        self.animation_state = "jump"
        self.current_frame = 0
        self.is_jumping = True
        self.jump_held = True
        self.jump_timer = 0.0
        self.hold_jump_power = 4
        self.change_y = 0
        self.change_y += self.jump_power
        logger.debug(f"Jump #{self.jump_count} started!")

    def update_jump(self, delta_time: float):
        """Handle held jump to extend height."""
        if self.is_attacking:
            self.change_y = 1
        elif self.change_y > 0:
            self.animation_state = "jump_up"
        elif self.change_y < 0:
            self.animation_state = "jump_down"
        if self.is_attacking or not self.is_jumping:
            return
        if not self.jump_held and self.jump_timer > 0:
            self.jump_timer = self.max_jump_time

        if self.jump_held and self.jump_timer < self.max_jump_time:
            self.jump_timer += delta_time
            self.hold_jump_power *= self.jump_volocity
            self.change_y += self.hold_jump_power
        else:
            self.jump_held = False
            self.hold_jump_power = self.base_hold_jump_power

    def handle_landing(self, offset: float = 5.0):
        """
        Reset jumps if character is on/just above a platform.
        offset: how far below the sprite to check for ground.
        """
        sensor = arcade.SpriteSolidColor(int(self.width), 2, arcade.color.WHITE)
        sensor.center_x = self.center_x
        sensor.center_y = self.bottom - offset

        hits = arcade.check_for_collision_with_list(sensor, self.stage)
        if hits and self.change_y <= 0 < self.jump_count and not self.is_attacking:
            if self.jump_count != 0:
                self.animation_state = "jump_landing"
                logger.debug("Landed! Jump count reset.")
                self.is_jumping = False
            self.jump_count = 0
            self.jump_held = False

    def attack(self, attack: CharacterAttack | None) -> Attack:
        if not attack:
            raise NoAttackSet()
        if self.is_attacking:
            raise NoAttackSet("not the time to attack yet")
        offset_hand_x, offset_hand_y = attack.position
        new_attack = attack.attack()
        if self.reverse:
            new_attack.center_x = self.center_x - offset_hand_x
            new_attack.direction = (-1, 0)
        else:
            new_attack.center_x = self.center_x + offset_hand_x
        new_attack.new_attack()
        new_attack.owner = self
        new_attack.center_y = self.center_y + offset_hand_y
        self.current_frame = 0
        self.animation_state = attack.animation
        self.is_attacking = True
        return new_attack

    def is_hurt(self, knockback_data: KnockBackDamage):
        self.center_x += 10
        self.center_y += 10
        self.animation_state = "hurt"
        self.frame_duration = 1 / 10
        self.current_frame = 0
        self.pending_knockback = knockback_data
        if not self.stun_sec:
            self.lives = max(0, self.lives - self.pending_knockback.damage)
        self.stun_sec = 0.5


# TODO: while in recovery state,Character(PLAYER_PATH, "monster") movement in reduced ( terug lopen terwilj je weg wordt geschoten)
# TODO: attack hit flashy things.


class EnemyAI:
    def __init__(self, character: Character, target: Character, stage):
        self.character = character
        self.target = target
        self.stage = stage
        self.attack_cooldown = 1.0  # seconds between attacks
        self.attack_timer = 0.0
        self.MOVE_SPEED = 2

    def update(self, delta_time: float):
        """Move toward target and attack when close."""
        self.attack_timer -= delta_time
        if self.attack_timer < 0:
            self.attack_timer = 0

        target_position = self.target.center_x
        if not self.character.stun_sec:
            if self.character.center_x < target_position - 10:
                self.character.change_x = self.MOVE_SPEED
                self.character.animation_state = "walk"
                self.character.reverse = False
            elif self.character.center_x > target_position + 10:
                self.character.change_x = -self.MOVE_SPEED
                self.character.animation_state = "walk"
                self.character.reverse = True
            else:
                self.character.change_x = 0
                self.character.animation_state = "idle"

            distance_x = abs(self.character.center_x - self.target.center_x)
            distance_y = abs(self.character.center_y - self.target.center_y)

            if distance_x < 100 and distance_y < 60 and self.attack_timer == 0:
                try:
                    attack_kind = random.choice(
                        [self.character.attacks.special, self.character.attacks.neutral]
                    )
                    attack_choice = random.choice(
                        [
                            attack_kind.up,
                            attack_kind.down,
                            attack_kind.side,
                            attack_kind.neutral,
                        ]
                    )
                    if not attack_choice:
                        return
                    try:
                        attack = self.character.attack(attack_choice)
                    except NoAttackSet:
                        pass
                    else:
                        self.stage.attack_hitboxes.append(attack)
                        self.character.animation_state = "attack_1"
                        self.attack_timer = self.attack_cooldown
                except ValueError:
                    pass  # no attack available
