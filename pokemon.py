import os
import random
from pathlib import Path

from loguru import logger

from schemas.monster_schema import Monster
from schemas.schemas import (
    AttackClass,
    Attacks,
    FighterSprites,
    SpriteSheet,
    EncounterMonster,
    QuestLineMove,
    Moves,
    DescriptionMonster,
    OverWorldMonsterSprites,
)
from smash_stage.attacks import (
    BlueFireBreath,
    SimpleMelee,
    FireBreath,
    ALL_ATTACKS,
)

ASSETS_PATH = Path(os.path.dirname(__file__)) / "assets"

Monsters = [
    Monster(
        name="Bulbasaur",
        description=DescriptionMonster(
            description="een monster met een groene puist", type="Life"
        ),
        fighter_sprites=FighterSprites(
            attack_melee=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Attack_2.png",
                texture_columns=4,
                size=128,
            ),
            attack_range=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Light_ball.png",
                texture_columns=7,
                size=128,
            ),
            attack_range_heavy=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "charge_attack.png",
                texture_columns=13,
                size=128,
            ),
            attack_heavy_melee=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Attack_1.png",
                texture_columns=10,
                size=128,
            ),
            dead=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Dead.png",
                texture_columns=5,
                size=128,
            ),
            hurt=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Hurt.png",
                texture_columns=3,
                size=128,
            ),
            idle=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Idle.png",
                texture_columns=7,
                size=128,
            ),
            jump=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Jump.png",
                texture_columns=8,
                size=128,
            ),
            run=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Run.png",
                texture_columns=8,
                size=128,
            ),
            walk=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Walk.png",
                texture_columns=7,
                size=128,
            ),
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
            attack_melee=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Attack_2.png",
                texture_columns=4,
                size=128,
            ),
            attack_range=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Light_ball.png",
                texture_columns=7,
                size=128,
            ),
            attack_range_heavy=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "charge_attack.png",
                texture_columns=13,
                size=128,
            ),
            attack_heavy_melee=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Attack_1.png",
                texture_columns=10,
                size=128,
            ),
            dead=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Dead.png",
                texture_columns=5,
                size=128,
            ),
            hurt=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Hurt.png",
                texture_columns=3,
                size=128,
            ),
            idle=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Idle.png",
                texture_columns=7,
                size=128,
            ),
            jump=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Jump.png",
                texture_columns=8,
                size=128,
            ),
            run=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Run.png",
                texture_columns=8,
                size=128,
            ),
            walk=SpriteSheet(
                spritesheet_path=ASSETS_PATH
                / "sprites"
                / "pokemon"
                / "Lightning Mage"
                / "Walk.png",
                texture_columns=7,
                size=128,
            ),
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
                name="fire breath",
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


def assign_attacks(monster: Monster) -> Attacks:
    attacks = [ALL_ATTACKS[move.name] for move in monster.moves] + [
        None
    ] * random.randint(1, 5)
    logger.debug(f"attacks are: {attacks}")
    logger.debug(f"attack type: {type(attacks[0])}")
    return Attacks(
        neutral=AttackClass(
            up=random.choice(attacks),
            down=random.choice(attacks),
            side=random.choice(attacks),
            neutral=random.choice(attacks),
        ),
        special=AttackClass(
            up=random.choice(attacks),
            down=random.choice(attacks),
            side=random.choice(attacks),
            neutral=random.choice(attacks),
        ),
    )


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
