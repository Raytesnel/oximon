import random
from pathlib import Path
from typing import Optional

import arcade
from arcade import PhysicsEnginePlatformer
from loguru import logger
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


class CharacterAttack(BaseModel):
    attack: type[Attack]
    position: tuple[int, int]
    end_start_up_frame: int
    end_attack_frame: int
    animation: str


class Attacks(BaseModel):
    up: CharacterAttack | None
    down: CharacterAttack | None
    left: CharacterAttack | None
    right: CharacterAttack | None
    base: CharacterAttack


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
        self.direction = "down"
        self.stun_duration = 1 / 5
        self.stun_counter = 0.0
        self.is_stunned = False
        self.pending_knockback: Optional[KnockBackDamage] = None
        self.current_frame = 0
        self.frame_timer = 0
        self.frame_duration = 1 / 10
        self.texture = self.animations["idle"][0]
        # --- jumping power
        self.max_jumps = 2
        self.is_jumping = False
        self.jump_held = False
        self.jump_timer = 0.0
        self.max_jump_time = 0.5
        self.jump_power = 10
        self.jump_volocity = 0.85
        self.base_hold_jump_power = 4
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
                if not self.is_jumping:
                    self.animation_state = "idle"
                    self.is_attacking = False
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
                    attack = self.attacks.right.attack()
                    attack.direction = (1, 0)
            case "special_left":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.left:
                    attack = self.attacks.left.attack()
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
                    attack = self.attacks.up.attack()
                    attack.angle = 90
                    attack.direction = (0, 1)

            case "special_down":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.down:
                    attack = self.attacks.down.attack()
                    attack.direction = (1, 0)

            case "normal_neutral":
                offset_hand_x = 30
                offset_hand_y = -15
                if self.attacks.base:
                    attack = self.attacks.base.attack()
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

    def idle(self):
        if not self.is_attacking:
            self.animation_state = "idle"
            self.change_x = 0

    def walk(self, direction):
        if self.is_attacking:
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
        if self.is_attacking:
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
        """jumping like hollow knight, how longer you press how higher you jump,
        this is called from on_key_press"""
        if self.is_attacking:
            return
        if not self.physics_engine.can_jump():
            logger.debug(f"jumps:{self.physics_engine.allowed_jumps}")
            return
        else:
            logger.debug(f"we can jump:{self.physics_engine.jumps_since_ground}")
            logger.debug(f"we can jump:{self.physics_engine.allowed_jumps}")
            logger.debug(f"we can jump:{self.physics_engine.can_jump()}")
        self.current_frame = 0
        self.animation_state = "jump"
        logger.debug(f"before jumping!:{self.change_y}")
        self.physics_engine.increment_jump_counter()
        logger.debug(f"we can jump:{self.physics_engine.can_jump()}")
        self.change_y = 0
        self.change_y += self.jump_power
        logger.debug(f"after jumping!:{self.change_y}")
        self.jump_timer = 0.0

    def update_jump(self, delta_time: float):
        if self.is_attacking:
            return
        if not self.physics_engine.can_jump():
            return
        if self.jump_held:
            logger.debug(f"keep climbing!: {round(self.hold_jump_power,2)}")
            self.hold_jump_power *= self.jump_volocity
            self.change_y += self.hold_jump_power
            self.jump_timer += delta_time
        if self.jump_timer > self.max_jump_time:
            logger.debug("times up!")
            self.jump_timer = 0
        if not self.jump_held and self.hold_jump_power != self.base_hold_jump_power:
            logger.debug("resetting jumping power")
            self.hold_jump_power = self.base_hold_jump_power
        if not self.is_jumping and not self.jump_held:
            return

    def attack(self, attack: CharacterAttack | None) -> Attack:
        if not attack:
            raise NoAttackSet()
        if self.is_attacking:
            raise NoAttackSet("not the time to attack yet")
        offset_hand_x, offset_hand_y = attack.position
        new_attack = attack.attack()
        new_attack.new_attack()
        new_attack.owner = self
        new_attack.center_y = self.center_y + offset_hand_y
        new_attack.center_x = self.center_x + offset_hand_x
        self.current_frame = 0
        self.animation_state = attack.animation
        self.is_attacking = True
        return new_attack

    def is_hurt(self, time_knockback: int): ...


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
        self.MOVE_SPEED = 2

    def update(self, delta_time: float):
        """Move toward target and attack when close."""
        self.attack_timer -= delta_time
        if self.attack_timer < 0:
            self.attack_timer = 0

        target_position = self.target.center_x
        if not self.character.is_stunned:
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
